use crate::{utils::{self, FileEntry}, StartArguments};
use std::{fs::{self, File},io::{Read, Write}, path::PathBuf};
use byteorder::{LittleEndian, WriteBytesExt};
use walkdir::WalkDir;

pub fn all(args: StartArguments) {
    for unpacked_pak in fs::read_dir(args.unpacked_path).unwrap().filter_map(|f| f.ok()) {
        if !unpacked_pak.path().is_dir() {
            continue;
        }

        println!("Repacking files from {:?}", unpacked_pak.path());

        //I LOVE SORTING
        let files = WalkDir::new(&unpacked_pak.path())
            .sort_by(utils::windows_sort)
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
                .to_str()
                .unwrap()
                .split_once(unpacked_pak.path().file_name().unwrap().to_str().unwrap())
                .unwrap()
                .1
                .to_string();
            formatted_path.remove(0);
            if args.verbose {
                println!("Packing file: {}", formatted_path);
            }
            formatted_path.push('\0');

            let mut file = File::open(path).unwrap();
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();

            if let Some(ext) = path.extension() {
                if ext == "wav" {
                    if let Ok(new_buffer) = utils::convert_wav_to_adpcm(buffer.clone()) {
                        buffer = new_buffer;
                    }
                }
            }

            let size = buffer.len() as u32;

            file_entries.push(FileEntry {
                file_name: PathBuf::from(formatted_path),
                offset,
                size,
            });

            data.extend(buffer);
            offset += size;
        }

        println!("Packed {:#?} files", file_entries.len());

        let output_file_path = PathBuf::from(&args.repacked_path).join(unpacked_pak.file_name().into_string().unwrap() + ".pak");
        fs::create_dir_all(output_file_path.parent().unwrap()).unwrap();

        let mut output_file = File::create(&output_file_path).unwrap();

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

        let sha = sha256::try_digest(output_file_path).unwrap();
        let sha_correct = sha256::try_digest(
            PathBuf::from(&args.input_path)
                .join(unpacked_pak.file_name().into_string().unwrap() + ".pak"),
        );

        if let Ok(sha_correct) = sha_correct {
            if sha != sha_correct {
                println!("SHA256 mismatch");
            }
        } else {
            println!("Could not calculate SHA256 hash: {}", sha_correct.err().unwrap());
        }

        println!()
    }
}
