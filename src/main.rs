use std::{
    ffi::CStr,
    fs,
    io::{Read, Seek, Write},
    path::PathBuf,
};

use byteorder::{LittleEndian, ReadBytesExt};

const BOX_SIZE: usize = 108;

struct FileEntry {
    file_name: PathBuf,
    offset: u32,
    size: u32,
}

fn main() {
    for input_file in fs::read_dir("input").unwrap().filter_map(|f| f.ok()) {
        let mut file_handle = std::fs::File::open(input_file.path()).unwrap();

        let file_size = file_handle.read_u32::<LittleEndian>().unwrap();
        println!("File size: {}", file_size);

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

            println!("Adding File: {:?}", file_name);

            let file_name = PathBuf::from("output").join(file_name);
            fs::create_dir_all(file_name.clone().parent().unwrap()).unwrap();

            files.push(FileEntry {
                file_name,
                offset: offset + file_size as u32 * BOX_SIZE as u32 + 4,
                size,
            });
        }

        for file in files {
            file_handle
                .seek(std::io::SeekFrom::Start(file.offset as u64))
                .unwrap();
            let mut buffer = vec![0; file.size as usize];

            if file.size == 0 {
                file_handle.read_to_end(&mut buffer).unwrap();
            } else {
                file_handle.read_exact(&mut buffer).unwrap();
            }

            println!("Writing File: {:?}", file.file_name);

            let mut file_handle = std::fs::File::create(file.file_name).unwrap();
            file_handle.write_all(&buffer).unwrap();
        }
    }
}
