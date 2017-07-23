use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::io::prelude::*;
use std::path::Path;
use std::process::exit;

extern crate fldtools;
use fldtools::{ChunkList, SECTOR_LENGTH};

extern crate clap;
use clap::{Arg, App};

fn write_file(filename : &String, writer : &mut BufWriter<File>) -> ::std::io::Result<u64> {
    let file_length = fs::metadata(&filename)?.len();
    let padding_size = SECTOR_LENGTH - (file_length as usize % SECTOR_LENGTH);

    let input_file = File::open(&filename)?;
    let mut buf_reader = BufReader::new(input_file);

    let mut bytes_to_read = file_length as usize;

    while bytes_to_read > 0 {
        // We define the buffer in here so anything past the number of
        // bytes written is FF rather than leftover bytes from the last
        // read.
        // It probably doesn't actually *matter* what the padding is, but
        // if we use FF then it matches the original files.
        let mut buf = vec![0xFF; SECTOR_LENGTH];
        let read_bytes = buf_reader.read(&mut buf)?;
        writer.write(&buf)?;
        bytes_to_read -= read_bytes;
    }

    // No off-by-one errors here please
    let bytes_written = file_length + padding_size as u64;
    debug_assert!((bytes_written % 2048) == 0);
    return Ok(bytes_written);
}

fn main() {
    let matches = App::new("fldpack")
                          .version("0.1.1")
                          .author("Misty De Meo")
                          .about("Pack Magical School Lunar! FLD files")
                          .arg(Arg::with_name("target")
                              .help("The packed filename")
                              .required(true)
                              .index(1))
                          .arg(Arg::with_name("input")
                              .help("File(s) to pack")
                              .required(true)
                              .multiple(true))
                          .get_matches();

    let target = matches.value_of("target").unwrap().to_string();
    let target_path = Path::new(&target);

    let input_files = matches.values_of("input").unwrap().map(|path| String::from(path)).collect::<Vec<String>>();
    if input_files.iter().any(|path| !Path::new(path).exists()) {
        println!("One or more input files are couldn't be found!");
        exit(1);
    }

    let target_file;
    match File::create(&target_path) {
        Ok(f) => target_file = f,
        Err(e) => {
            println!("Unable to create output file {}: {}", target, e);
            exit(1);
        }
    }
    let mut writer = BufWriter::new(target_file);

    let file_lengths = input_files.iter().map(|file| fs::metadata(file).unwrap().len() as usize).collect::<Vec<usize>>();
    let chunk_list = ChunkList::build(&file_lengths);

    // Start by writing out the header
    writer.write(&chunk_list.serialize().unwrap()).unwrap();

    // Then iterate over each file
    for filename in input_files {
        match write_file(&filename, &mut writer) {
            Ok(_) => {},
            Err(e) => {
                println!("Encountered an error trying to write file {}: {}", filename, e);
                println!("Leaving partial output file in place.");
                exit(1);
            }
        }
    }

    println!("Done! Output written to {}", target);
}
