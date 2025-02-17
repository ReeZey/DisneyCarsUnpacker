use std::{ffi::CStr, fs::{self, File}, io::{Read, Seek, SeekFrom, Write}, path::PathBuf};

use crate::{utils::{self, FileEntry}, INPUT_PATH, UNPACKED_PATH, VERBOSE};
use byteorder::{LittleEndian, ReadBytesExt};

const BOX_SIZE: usize = 108;

pub fn all() {
    for input_file in fs::read_dir(INPUT_PATH).unwrap().filter_map(|f| f.ok()) {
        if input_file.path().is_dir() || input_file.path().extension().unwrap() != "pak" {
            continue;
        }

        println!("Parsing boxes from {:?}", input_file.path());
        let boxes = parse_boxes(&input_file.path());

        println!("Extracting files from boxes...");
        extract_boxes(boxes, &input_file.path());
    }
}

pub fn parse_boxes(file_path: &PathBuf) -> Vec<FileEntry> {
    let mut file_handle = std::fs::File::open(file_path).unwrap();
    let file_stem = file_path.file_stem().unwrap();

    let file_size = file_handle.read_u32::<LittleEndian>().unwrap();
    if VERBOSE {
        println!("File size: {}", file_size);
    }

    let mut files: Vec<FileEntry> = Vec::with_capacity(108 * file_size as usize);

    for _ in 0..file_size {
        let mut buffer = [0; BOX_SIZE];
        file_handle.read_exact(&mut buffer).unwrap();

        let file_name: PathBuf = CStr::from_bytes_until_nul(&buffer)
            .unwrap()
            .to_string_lossy()
            .to_string()
            .into();

        let mut last_bytes = &buffer[100..];
        let offset = last_bytes.read_u32::<LittleEndian>().unwrap();
        let size = last_bytes.read_u32::<LittleEndian>().unwrap();

        if VERBOSE {
            println!("Adding File: {:?}", file_name);
        }

        let file_name = PathBuf::from(UNPACKED_PATH).join(file_stem).join(file_name);
        fs::create_dir_all(file_name.clone().parent().unwrap()).unwrap();

        files.push(FileEntry {
            file_name,
            offset: offset + file_size as u32 * BOX_SIZE as u32 + 4,
            size,
        });
    }

    return files;
}

pub fn extract_boxes(files: Vec<FileEntry>, file_path: &PathBuf) {
    let mut file_handle = std::fs::File::open(file_path).unwrap();

    for file in files {
        file_handle
            .seek(SeekFrom::Start(file.offset as u64))
            .unwrap();
        let mut buffer = vec![0; file.size as usize];
        file_handle.read_exact(&mut buffer).unwrap();

        if VERBOSE {
            println!("Writing File: {:?}", file.file_name);
        }

        let mut file_handle = File::create(&file.file_name).unwrap();

        if let Some(ext) = file.file_name.extension() {
            match ext.to_str().unwrap() {
                /*
                "dxt" => {
                    let success = crate::utils::convert_image(&mut buffer, file.file_name.clone());
                    
                    if success {
                        continue;
                    }
                }
                */
                "wav" => {
                    if utils::convert_adpcm_to_wav(buffer.clone(), file.file_name.clone()).is_ok() {
                        continue;
                    }
                }
                _ => {}
            }
        }

        file_handle.write_all(&buffer).unwrap();
    }
} 