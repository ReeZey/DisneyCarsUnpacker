use std::{
    ffi::CStr,
    fs::{self, File},
    io::{Cursor, Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use walkdir::WalkDir;

use crate::VERBOSE;

const BOX_SIZE: usize = 108;

#[derive(Debug)]
pub struct FileEntry {
    file_name: PathBuf,
    offset: u32,
    size: u32,
}

pub fn extract_all(file_path: &PathBuf) {
    for input_file in fs::read_dir(file_path).unwrap().filter_map(|f| f.ok()) {
        if input_file.path().is_dir() || input_file.path().extension().unwrap() != "pak" {
            continue;
        }

        println!("Extracting files from {:?}", input_file.path());

        let files = unbox(&input_file.path());
        extract_files(files, &input_file.path());

        println!("done");
    }
}

pub fn unbox(file_path: &PathBuf) -> Vec<FileEntry> {
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
        let file_name = PathBuf::from("output").join(file_stem).join(file_name);
        fs::create_dir_all(file_name.clone().parent().unwrap()).unwrap();

        files.push(FileEntry {
            file_name,
            offset: offset + file_size as u32 * BOX_SIZE as u32 + 4,
            size,
        });
    }

    return files;
}

pub fn extract_files(files: Vec<FileEntry>, file_path: &PathBuf) {
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

        /*
        if let Some(ext) = file.file_name.extension() {
            if ext == "dxt" {
                let success = convert_image(&mut buffer, file.file_name.clone());

                if success {
                    continue;
                }
            }
        }
        */

        file_handle.write_all(&buffer).unwrap();
    }
}

pub fn repack_all(input_path: &PathBuf, output_path: &PathBuf) {
    for unpacked_pak in fs::read_dir(input_path).unwrap().filter_map(|f| f.ok()) {
        if !unpacked_pak.path().is_dir() {
            continue;
        }

        println!("Repacking files from {:?}", unpacked_pak.path());

        //I LOVE SORTING
        let files = WalkDir::new(&unpacked_pak.path())
            .sort_by(|a, b| {
                a.file_name().to_ascii_lowercase().cmp(&b.file_name().to_ascii_lowercase())
            })
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .collect::<Vec<_>>();

        let mut file_entries: Vec<FileEntry> = Vec::with_capacity(files.len());

        let mut offset = 0;
        let mut data = Vec::new();
        for file in files {
            let path = file.path();

            //im sorry
            let mut formatted_path = path
                .to_str().unwrap()
                .split_once(unpacked_pak.path().file_name().unwrap().to_str().unwrap())
                .unwrap().1
                .to_string();
            formatted_path.remove(0);
            formatted_path.push('\0');

            if VERBOSE {
                println!("Packing file: {}", formatted_path);
            }

            let size = file.metadata().unwrap().len() as u32;

            file_entries.push(FileEntry {
                file_name: PathBuf::from(formatted_path),
                offset,
                size,
            });

            let mut file = File::open(path).unwrap();
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();

            data.extend(buffer);
            offset += size;
        }

        println!("Packed {:#?} files", file_entries.len());

        let output_file =
            output_path.join(unpacked_pak.file_name().into_string().unwrap() + ".pak");
        fs::create_dir_all(output_file.parent().unwrap()).unwrap();

        let mut output_file = File::create(output_file).unwrap();

        output_file
            .write_u32::<LittleEndian>(file_entries.len() as u32)
            .unwrap();
        for file in file_entries {
            let file_name = file.file_name.to_str().unwrap();
            let offset = file.offset;
            let size = file.size;

            let mut buffer = file_name.as_bytes().to_vec();
            buffer.resize(100, 0xCC);
            buffer.extend_from_slice(&offset.to_le_bytes());
            buffer.extend_from_slice(&size.to_le_bytes());

            output_file.write_all(&buffer).unwrap();
        }
        output_file.write_all(&data).unwrap();
    }
}


fn convert_image(buffer: &mut Vec<u8>, file_name: PathBuf) -> bool {
    let mut file = Cursor::new(buffer);

    let _unknown = file.read_u32::<LittleEndian>().unwrap();
    let flags = file.read_u32::<LittleEndian>().unwrap();

    let mut skip = false;
    match flags {
        38 => {}
        54 => {}
        50 => {}
        unsupported => {
            println!(
                "Unsupported DXT format: {}. {}",
                unsupported,
                file_name.to_string_lossy()
            );
            skip = true
        }
    }
    if skip {
        return false;
    }

    //println!("Converting DXT image: {:?}", file_name);

    let _unknown3 = file.read_u32::<LittleEndian>().unwrap();
    let _unknown4 = file.read_u32::<LittleEndian>().unwrap();
    let width = file.read_u32::<LittleEndian>().unwrap();
    let height = file.read_u32::<LittleEndian>().unwrap();
    let _unknown5 = file.read_u32::<LittleEndian>().unwrap();
    let _unknown6 = file.read_u32::<LittleEndian>().unwrap();
    let size = file.read_u32::<LittleEndian>().unwrap();

    if width != _unknown5 || height != _unknown6 {
        println!(
            "Invalid DXT image: {}. {}",
            file_name.to_string_lossy(),
            size
        );
        return false;
    }

    let mut input = vec![0; size as usize];
    file.read_exact(&mut input).unwrap();

    let mut output = vec![0; (width * height * 4) as usize];

    match flags {
        38 => texpresso::Format::Bc1.decompress(
            &mut input,
            width as usize,
            height as usize,
            &mut output,
        ),
        54 => texpresso::Format::Bc3.decompress(
            &mut input,
            width as usize,
            height as usize,
            &mut output,
        ),
        50 => texpresso::Format::Bc3.decompress(
            &mut input,
            width as usize,
            height as usize,
            &mut output,
        ),
        _ => {
            return false;
        }
    }

    let mut image = image::ImageBuffer::new(width, height);

    image.enumerate_pixels_mut().for_each(|(x, y, pixel)| {
        let index = ((y * width + x) * 4) as usize;
        let r = output[index];
        let g = output[index + 1];
        let b = output[index + 2];
        let a = output[index + 3];
        *pixel = image::Rgba([r, g, b, a]);
    });

    image
        .save(format!("{}.png", file_name.to_string_lossy()))
        .unwrap();

    return false;
}
