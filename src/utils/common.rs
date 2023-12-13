#![allow(dead_code, unused_variables)]

use crc::CRC_32_ISO_HDLC;

pub const PNG_MAGICBYTES: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
pub const PNG_IEND: [u8; 4] = [73, 69, 78, 68];

pub const PNG_CUSTOMCHUNK: [u8; 4] = [13, 37, 13, 37];

pub const CHUNK_LEN_SIZE: usize = 4;
pub const CHUNK_HEADER_SIZE: usize = 4;
pub const CHUNK_CRC_SIZE: usize = 4;

pub const NONCE_LEN: usize = 12;
pub const SALT_LEN: usize = 16;

pub struct Pngchunk {
    length: [u8; CHUNK_LEN_SIZE],
    header: [u8; CHUNK_HEADER_SIZE],
    content: Vec<u8>,
    crc: [u8; CHUNK_CRC_SIZE],
}

impl Default for Pngchunk {
    fn default() -> Self {
        Pngchunk {
            length: [0, 0, 0, 0],
            header: [0, 0, 0, 0],
            content: Vec::new(),
            crc: [0, 0, 0, 0],
        }
    }
}
impl Pngchunk {
    pub fn new() -> Self {
        Pngchunk {
            length: [0, 0, 0, 0],
            header: [0, 0, 0, 0],
            content: Vec::new(),
            crc: [0, 0, 0, 0],
        }
    }
    pub fn load_from_slice(&mut self, slice: Vec<u8>) {
        self.length = slice[0..CHUNK_LEN_SIZE].try_into().unwrap();
        self.header = slice[CHUNK_LEN_SIZE..CHUNK_LEN_SIZE + CHUNK_HEADER_SIZE]
            .try_into()
            .unwrap();
        self.content = slice[CHUNK_LEN_SIZE + CHUNK_HEADER_SIZE
            ..CHUNK_LEN_SIZE + CHUNK_HEADER_SIZE + self.chunk_length()]
            .to_vec();
        self.crc = slice[CHUNK_LEN_SIZE + CHUNK_HEADER_SIZE + self.chunk_length()
            ..CHUNK_LEN_SIZE + CHUNK_HEADER_SIZE + self.chunk_length() + CHUNK_CRC_SIZE]
            .try_into()
            .unwrap();
    }

    pub fn create_from_content(&mut self, header: [u8; CHUNK_HEADER_SIZE], content: Vec<u8>) {
        self.length = u32::try_from(content.len()).unwrap().to_be_bytes();
        self.header = header;
        self.content = content;
        self.crc = self.calc_crc()
    }

    pub fn chunk_len(&self) -> usize {
        self.chunk_length()
    }

    pub fn content(&self) -> &Vec<u8> {
        &self.content
    }

    pub fn len(&self) -> usize {
        CHUNK_LEN_SIZE + CHUNK_HEADER_SIZE + self.chunk_length() + CHUNK_CRC_SIZE
    }

    fn calc_crc(&mut self) -> [u8; 4] {
        let mut bytes = self.length.to_vec();
        bytes.extend(self.header);
        bytes.extend(&self.content);
        let crc = crc::Crc::<u32>::new(&CRC_32_ISO_HDLC).checksum(bytes.as_slice());
        return crc.to_be_bytes();
    }
    fn chunk_length(&self) -> usize {
        let b: [u8; CHUNK_LEN_SIZE] = self.length.try_into().expect("slice with incorrect length");
        ((b[0] as usize) << 24)
            + ((b[1] as usize) << 16)
            + ((b[2] as usize) << 8)
            + ((b[3] as usize) << 0)
    }

    pub fn flatten(&mut self) -> Vec<u8> {
        let mut bytes = self.length.to_vec();
        bytes.extend(self.header);
        bytes.extend(&self.content);
        bytes.extend(self.crc);
        return bytes;
    }
}

pub fn find_sequence<T>(haystack: &[T], needle: &[T]) -> Option<usize>
where
    for<'a> &'a [T]: PartialEq,
{
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
