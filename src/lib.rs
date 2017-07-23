//! Tools to read and write FLD files, as used by [*Magical School Lunar!* (1997)](https://en.wikipedia.org/wiki/Lunar:_Samposuru_Gakuen)
//! for the Sega Saturn.
//! FLD files are concatenated chunks of arbitrary data; the game uses them
//! to bundle together sets of related files. These chunks are essentialy
//! unnamed files, and an FLD file can be treated as a flat filesystem.
//!
//! More information on the FLD data format can be found in
//! this set of [Magical School Lunar! notes](https://github.com/mistydemeo/magicaldata/blob/master/files/SXX.FLD.md).

use std::io;

extern crate byteorder;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

extern crate itertools;
use itertools::Itertools;

/// The size of a Mode-1 CD-ROM sector, in bytes.
/// Because the components of FLD files are padded out to even sector boundaries,
/// this is useful to calculate offsets.
pub const SECTOR_LENGTH : usize = 2048;

/// Represents the header of an FLD file.
/// A ChunkList is a simple collection of one or more data chunks.
pub struct ChunkList {
    pub chunks: Vec<Chunk>,
}

/// Represents a single data chunk within an FLD file.
pub struct Chunk {
    /// The offset of this chunk from the beginning of the file.
    pub start: u32,
    /// The length of this chunk; this is the original, unpadded size.
    pub length: u32,
}

impl Chunk {
    /// Parses a chunk from raw data.
    /// `data` should be an 8-byte slice containing two 32-bit big endian
    /// integers, as read directly out of an FLD header.
    pub fn parse(mut data : &[u8]) -> io::Result<Chunk> {
        return Ok(Chunk {
            start: data.read_u32::<BigEndian>()?,
            length: data.read_u32::<BigEndian>()?,
        });
    }

    /// Serializes this Chunk into its binary representation.
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
    /// Builds a ChunkList given a list of file sizes.
    /// This will calculate offsets for each file to be packed
    /// within an FLD file and use that to generate a set of Chunks.
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

    /// Parses an FLD header from raw data, and returns a ChunkList.
    /// This skips any portion of the header which begins with `0xFF`,
    /// since FLD files are padded using sets of `0xFF`s.
    pub fn parse(data : &[u8]) -> io::Result<ChunkList> {
        let chunks : Vec<Chunk> = data
            .chunks(8)
            .filter(|chunk| chunk[0] != 255)
            .map(|chunk| Chunk::parse(chunk))
            .collect::<io::Result<Vec<Chunk>>>()?;

        return Ok(ChunkList {
            chunks: chunks,
        });
    }

    /// Serializes this ChunkList into its binary representation.
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
