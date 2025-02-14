use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

#[derive(Debug)]
pub struct Riff {
    pub chunk_size: u32,
    pub format: u16,
    pub channels: u16,
    pub sample_rate: u32,
    pub byte_rate: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,
}

const SOME_WEIRD_CONSTANT: u32 = 4194304;

impl Riff {
    pub fn as_bytes(&self, data: Vec<u8>) -> Vec<u8> {
        let size = data.len() as u32;

        let mut bytes = Vec::new();
        bytes.extend(b"RIFF".iter());
        bytes.write_u32::<LittleEndian>(size + 40).unwrap();
        bytes.extend(b"WAVEfmt ".iter());
        bytes.write_u32::<LittleEndian>(self.chunk_size).unwrap();
        bytes.write_u16::<LittleEndian>(self.format).unwrap();
        bytes.write_u16::<LittleEndian>(self.channels).unwrap();
        bytes.write_u32::<LittleEndian>(self.sample_rate).unwrap();
        bytes.write_u32::<LittleEndian>(self.byte_rate).unwrap();
        bytes.write_u16::<LittleEndian>(self.block_align).unwrap();
        bytes.write_u16::<LittleEndian>(self.bits_per_sample).unwrap();
        bytes.write_u32::<LittleEndian>(SOME_WEIRD_CONSTANT).unwrap();
        bytes.extend(b"data".iter());
        bytes.write_u32::<LittleEndian>(size).unwrap();
        bytes.extend(data.iter());
        bytes
    }

    pub fn new(buffer: &Vec<u8>) -> Self {
        let mut cursor = Cursor::new(buffer);
        cursor.set_position(16);
        let chunk_size = cursor.read_u32::<LittleEndian>().unwrap();
        let format = cursor.read_u16::<LittleEndian>().unwrap();
        let channels = cursor.read_u16::<LittleEndian>().unwrap();
        let sample_rate = cursor.read_u32::<LittleEndian>().unwrap();
        let byte_rate = cursor.read_u32::<LittleEndian>().unwrap();
        let block_align = cursor.read_u16::<LittleEndian>().unwrap();
        let bits_per_sample = cursor.read_u16::<LittleEndian>().unwrap();
        
        Self {
            chunk_size,
            format,
            channels,
            sample_rate,
            byte_rate,
            block_align,
            bits_per_sample,
        }
    }
}