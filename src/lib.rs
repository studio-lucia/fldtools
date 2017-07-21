extern crate byteorder;
use byteorder::{BigEndian, ReadBytesExt};

struct ChunkList {
    chunks: Vec<Chunk>,
}

struct Chunk {
    start: u32,
    length: u32,
}

impl Chunk {
    fn parse(mut data : &[u8]) -> Chunk {
        return Chunk {
            start: data.read_u32::<BigEndian>().unwrap(),
            length: data.read_u32::<BigEndian>().unwrap(),
        }
    }
}

impl ChunkList {
    fn parse(data : &[u8]) -> ChunkList {
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