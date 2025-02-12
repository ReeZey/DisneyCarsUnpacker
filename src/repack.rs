use crate::{utils::{self, FileEntry}, REPACKED_PATH, UNPACKED_PATH};
use std::{fs::{self, File},io::{Read, Write}, path::PathBuf};
use byteorder::{LittleEndian, WriteBytesExt};
use walkdir::WalkDir;

use crate::{INPUT_PATH, VERBOSE};

pub fn all() {
    for unpacked_pak in fs::read_dir(UNPACKED_PATH).unwrap().filter_map(|f| f.ok()) {
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

        let output_file_path = PathBuf::from(REPACKED_PATH).join(unpacked_pak.file_name().into_string().unwrap() + ".pak");
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
            PathBuf::from(INPUT_PATH)
                .join(unpacked_pak.file_name().into_string().unwrap() + ".pak"),
        )
        .unwrap();

        if sha != sha_correct {
            println!("SHA256 mismatch");
        }
    }
}
