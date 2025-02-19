use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    cmp::Ordering,
    fs::{self},
    io::{Cursor, Error, ErrorKind, Read},
    path::PathBuf,
};
use walkdir::DirEntry;

use crate::{riff::Riff, VERBOSE};

#[derive(Debug)]
pub struct FileEntry {
    pub file_name: PathBuf,
    pub offset: u32,
    pub size: u32,
}

pub fn convert_image(buffer: &mut Vec<u8>, file_path: PathBuf) -> bool {
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
                    file_path.to_string_lossy()
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
    let image_count = file.read_u32::<LittleEndian>().unwrap();
    let _full_width = file.read_u32::<LittleEndian>().unwrap();
    let _full_height = file.read_u32::<LittleEndian>().unwrap();

    for _ in 0..image_count {
        let current_width = file.read_u32::<LittleEndian>().unwrap();
        let current_height = file.read_u32::<LittleEndian>().unwrap();
        let size = file.read_u32::<LittleEndian>().unwrap();
    
        let mut input = vec![0; size as usize];
        file.read_exact(&mut input).unwrap();
    
        let mut output = vec![0; (current_width * current_height * 4) as usize];
    
        match flags {
            38 => texpresso::Format::Bc1.decompress(
                &mut input,
                current_width as usize,
                current_height as usize,
                &mut output,
            ),
            54 => texpresso::Format::Bc3.decompress(
                &mut input,
                current_width as usize,
                current_height as usize,
                &mut output,
            ),
            50 => texpresso::Format::Bc3.decompress(
                &mut input,
                current_width as usize,
                current_height as usize,
                &mut output,
            ),
            _ => {
                return false;
            }
        }
    
        let mut image = image::ImageBuffer::new(current_width, current_height);

        image.enumerate_pixels_mut().for_each(|(x, y, pixel)| {
            let index = ((y * current_width + x) * 4) as usize;
            let r = output[index];
            let g = output[index + 1];
            let b = output[index + 2];
            let a = output[index + 3];
            *pixel = image::Rgba([r, g, b, a]);
        });
    
        let mut name = format!("{}", file_path.file_stem().unwrap().to_string_lossy());
        if image_count > 1 {
            name += &format!("-{}", current_width);
        }
        let file_name = file_path.with_file_name(name).with_extension("png");

        image
            .save(&file_name)
            .unwrap();
    
        if VERBOSE {
            println!("Converted DXT image: {:?}", file_name);
        }
    }

    return false;
}

pub fn convert_adpcm_to_wav(buffer: Vec<u8>, file_name: PathBuf) -> Result<(), Error> {
    let mut file = Cursor::new(buffer);
    let mut riff = Riff::new(&mut file);

    if riff.format != 0x2 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("Unsupported format: {:X}", riff.format),
        ));
    }
    riff.format = 1;

    if VERBOSE {
        println!("Converting ADPCM audio: {:?}", file_name);
        println!("{:?}", riff);
    }

    let mut data = vec![];
    file.read_to_end(&mut data).unwrap();

    let mut output = vec![];

    let mut low_adpcm_state = audio_codec_algorithms::AdpcmImaState::new();
    let mut top_adpcm_state = audio_codec_algorithms::AdpcmImaState::new();

    for chunk in data {
        let low = audio_codec_algorithms::decode_adpcm_ima(chunk >> 4, &mut low_adpcm_state);
        output.push(low);

        let top_state = match riff.channels {
            1 => &mut low_adpcm_state,
            2 => &mut top_adpcm_state,
            _ => panic!("Unsupported channel count: {}", riff.channels),
        };

        let top = audio_codec_algorithms::decode_adpcm_ima(chunk & 0x0F, top_state);
        output.push(top);
    }

    let data = output
        .iter()
        .flat_map(|x| x.to_le_bytes())
        .collect::<Vec<u8>>();

    fs::write(&file_name, riff.as_bytes(data)).unwrap();

    return Ok(());
}

pub fn convert_wav_to_adpcm(buffer: Vec<u8>) -> Result<Vec<u8>, Error> {
    let mut file = Cursor::new(buffer);
    let mut riff = Riff::new(&mut file);

    if riff.format != 0x1 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("Unsupported format: {:X}", riff.format),
        ));
    }
    riff.format = 2;

    let mut data = vec![];
    let mut low_adpcm_state = audio_codec_algorithms::AdpcmImaState::new();
    let mut top_adpcm_state = audio_codec_algorithms::AdpcmImaState::new();

    let mut byte = 0;
    let mut index = 0;

    loop {
        match file.read_i16::<LittleEndian>() {
            Ok(wav) => {
                if index == 4 {
                    let top_state = match riff.channels {
                        1 => &mut low_adpcm_state,
                        2 => &mut top_adpcm_state,
                        _ => panic!("Unsupported channel count: {}", riff.channels),
                    };

                    byte |= audio_codec_algorithms::encode_adpcm_ima(wav, top_state);
                } else {
                    byte = audio_codec_algorithms::encode_adpcm_ima(wav, &mut low_adpcm_state) << 4;
                }

                index += 4;
                if index == 8 {
                    data.push(byte);
                    index = 0;
                }
            }
            Err(_) => break,
        }
    }

    return Ok(riff.as_bytes(data));
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
            }
        };
    }

    check
}
