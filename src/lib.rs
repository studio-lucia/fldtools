use std::io;

extern crate byteorder;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

extern crate itertools;
use itertools::Itertools;

// Size of a Mode-1 CD-ROM sector, in bytes
pub const SECTOR_LENGTH : usize = 2048;

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

    pub fn serialize(&self) -> io::Result<Vec<u8>> {
        let mut writer = vec![];
        writer.write_u32::<BigEndian>(self.start)?;
        writer.write_u32::<BigEndian>(self.length)?;

        return Ok(writer);
    }
}

fn fold_vecs<T>(mut a : Vec<T>, b : Vec<T>) -> Vec<T> {
    a.extend(b);
    return a;
}

impl ChunkList {
    pub fn build(files : &[usize]) -> ChunkList {
        // We start at the beginning of a sector because
        // the header is padded to 2048 bytes.
        let mut index = SECTOR_LENGTH;
        let mut chunks = vec![];

        for file_length in files {
            chunks.push(Chunk {
                start: index as u32,
                length: *file_length as u32,
            });

            index += *file_length;
            // Pad up to an even sector boundary if necessary
            index += SECTOR_LENGTH - (*file_length % SECTOR_LENGTH);
        }

        return ChunkList {
            chunks: chunks,
        };
    }

    pub fn parse(data : &[u8]) -> ChunkList {
        let chunks = data
            .chunks(8)
            .filter(|chunk| chunk[0] != 255)
            .map(|chunk| Chunk::parse(chunk))
            .collect();

        return ChunkList {
            chunks: chunks,
        }
    }

    pub fn serialize(&self) -> io::Result<Vec<u8>> {
        let mut serialized = self
            .chunks
            .iter()
            .map(|chunk| chunk.serialize())
            .fold_results(vec![], fold_vecs)?;

        // Pad out with FFs to reach an even sector boundary.
        serialized.resize(SECTOR_LENGTH, 255);

        return Ok(serialized);
    }
}

impl IntoIterator for ChunkList {
    type Item = Chunk;
    type IntoIter = ::std::vec::IntoIter<Chunk>;

    fn into_iter(self) -> Self::IntoIter {
        return self.chunks.into_iter();
    }
}
