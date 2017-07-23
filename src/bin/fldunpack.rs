use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;
use std::process::exit;

extern crate fldtools;
use fldtools::{Chunk, ChunkList, SECTOR_LENGTH};

extern crate clap;
use clap::{Arg, App};

fn get_output_name(file : &Path, index : usize, extension : &str) -> String {
    return format!("{}.{}.{}", 
        file.file_stem().unwrap().to_str().unwrap(),
        index,
        extension);
}

// Given an input file and a file to write to, reads `chunk_data.length` of
// data from the input beginning at `chunk_data.start`.
// Data is read/written in 2048-byte chunks.
fn write_chunk(mut reader : &mut BufReader<&mut File>, mut target : &File, chunk_data : &Chunk) -> ::std::io::Result<u32> {
    reader.seek(SeekFrom::Start(chunk_data.start as u64))?;
    let mut bytes_to_read = chunk_data.length as usize;

    while bytes_to_read > 0 {
        let buf_size;
        if bytes_to_read >= 2048 {
            buf_size = SECTOR_LENGTH;
        } else {
            buf_size = bytes_to_read as usize;
        }

        let mut buf : Vec<u8> = vec![0; buf_size];
        reader.read_exact(&mut buf)?;
        target.write(&buf)?;
        bytes_to_read -= buf_size;
    }

    return Ok(chunk_data.length);
}

fn main() {
    let matches = App::new("fldunpack")
                          .version("0.1.0")
                          .author("Misty De Meo")
                          .about("Unpack Magical School Lunar! FLD files")
                          .arg(Arg::with_name("input")
                              .help("The file to unpack")
                              .required(true)
                              .index(1))
                          .arg(Arg::with_name("output_dir")
                              .help("Directory to unpack to")
                              .required(false)
                              .index(2))
                          .arg(Arg::with_name("extension")
                              .help("File extension of extracted files (default: .chunk)")
                              .long("extension")
                              .takes_value(true))
                          .get_matches();
    let input = matches.value_of("input").unwrap().to_string();
    let input_path = Path::new(&input);

    let output_dir;
    match matches.value_of("output_dir") {
        Some(dir) => output_dir = String::from(dir),
        None => output_dir = String::from(input_path.parent().unwrap().to_string_lossy()),
    }
    let output_path = Path::new(&output_dir);

    let extension = matches.value_of("extension").unwrap_or("chunk").to_string();

    let mut input_file;
    match File::open(&input_path) {
        Ok(f) => input_file = f,
        Err(e) => {
            println!("Unable to open file {}: {}", input, e);
            exit(1);
        }
    }

    let mut buf_reader = BufReader::new(&mut input_file);
    // The header is exactly one 2048-byte sector;
    // we'll also read 2048-byte increments as we go.
    let mut data : Vec<u8> = vec![0; SECTOR_LENGTH];
    match buf_reader.read_exact(&mut data) {
        Ok(_) => {},
        Err(e) => {
            println!("Unable to read file {}: {}", input, e);
            exit(1);
        }
    }

    let chunk_list = ChunkList::parse(&data).unwrap();
    for (i, chunk) in chunk_list.into_iter().enumerate() {
        let output_name = get_output_name(input_path, i, &extension);
        let output_name_path = Path::new(&output_name);
        let output = output_path.join(output_name_path);
        let mut output_file = File::create(&output).unwrap();

        println!("Writing chunk {} to {}", i, output_name);

        match write_chunk(&mut buf_reader, &mut output_file, &chunk) {
            Ok(_) => {},
            Err(e) => {
                println!("Error when trying to write chunk {}: {}", output_name, e);
                exit(1);
            }
        }
    }
}
