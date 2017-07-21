extern crate byteorder;
use byteorder::{BigEndian, ReadBytesExt};

pub struct ChunkList {
    pub chunks: Vec<Chunk>,
}

pub struct Chunk {
    pub start: u32,
    pub length: u32,
}

impl Chunk {
    pub fn parse(mut data : &[u8]) -> Chunk {
        return Chunk {
            start: data.read_u32::<BigEndian>().unwrap(),
            length: data.read_u32::<BigEndian>().unwrap(),
        }
    }
}

impl ChunkList {
    pub fn parse(data : &[u8]) -> ChunkList {
        let chunks = data
            .chunks(16)
            .filter(|chunk| chunk[0] != 255)
            .map(|chunk| Chunk::parse(chunk))
            .collect();

        return ChunkList {
            chunks: chunks,
        }
    }
}
