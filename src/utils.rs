use std::{cmp::Ordering, io::{Cursor, Read}, path::PathBuf};
use byteorder::{LittleEndian, ReadBytesExt};
use walkdir::DirEntry;

use crate::VERBOSE;

#[derive(Debug)]
pub struct FileEntry {
    pub file_name: PathBuf,
    pub offset: u32,
    pub size: u32,
}

#[allow(dead_code)]
pub fn convert_image(buffer: &mut Vec<u8>, file_name: PathBuf) -> bool {
    let mut file = Cursor::new(buffer);

    let _unknown = file.read_u32::<LittleEndian>().unwrap();
    let flags = file.read_u32::<LittleEndian>().unwrap();

    let mut skip = false;
    match flags {
        38 => {}
        54 => {}
        50 => {}
        unsupported => {
            if VERBOSE {
                println!(
                    "Unsupported DXT format: {}. {}",
                    unsupported,
                    file_name.to_string_lossy()
                );
            }
           
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
        if VERBOSE {
            println!(
                "Invalid DXT image: {}. {}",
                file_name.to_string_lossy(),
                size
            );
        }
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

    if VERBOSE {
        println!("Converted DXT image: {:?}", file_name);
    }

    return false;
}

pub fn windows_sort(a: &DirEntry, b: &DirEntry) -> Ordering {
    let mut a_filename = a.file_name().to_string_lossy().to_lowercase();
    let mut b_filename = b.file_name().to_string_lossy().to_lowercase();

    if a.path().is_file() && a_filename.contains(".") {
        a_filename = a_filename.split_once(".").unwrap().0.to_string();
    }

    if b.path().is_file() && b_filename.contains(".") {
        b_filename = b_filename.split_once(".").unwrap().0.to_string();
    }

    if a.path().is_file() && a_filename.contains(" ") {
        a_filename = a_filename.split_once(" ").unwrap().0.to_string();
    }

    if b.path().is_file() && b_filename.contains(" ") {
        b_filename = b_filename.split_once(" ").unwrap().0.to_string();
    }

    let a_is_dir = a.path().is_dir();
    let b_is_dir = b.path().is_dir();

    let check = a_filename.cmp(&b_filename);

    if check == Ordering::Equal {
        return match (a_is_dir, b_is_dir) {
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            _ => {
                a_filename = a.file_name().to_string_lossy().to_lowercase();
                b_filename = b.file_name().to_string_lossy().to_lowercase();

                alphanumeric_sort::compare_str(&a_filename, &b_filename)
            },
        };
    }

    check
}