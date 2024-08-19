use std::collections::HashMap;
use std::fs::File as StdFile;
use std::io::{BufReader, BufWriter, Read, Seek, Write};

pub fn read_replacement_map(input: &str) -> HashMap<u8, (u8, u8)> {
    let input = StdFile::open(input).expect("Could not open file");
    let mut reader = BufReader::new(input);
    let mut buffer = [0; 1];
    reader
        .read_exact(&mut buffer)
        .expect("could not read input file");
    let length = buffer[0];
    println!("Hashmap is {}", length);
    let mut buffer = vec![0; length as usize * 3];
    reader
        .read_exact(&mut buffer)
        .expect("Could not read replace specifications");
    let replacement_map: HashMap<u8, (u8, u8)> = buffer
        .chunks(3)
        .map(|value| (value[0], (value[1], value[2])))
        .collect();
    return replacement_map;
}

pub fn decode_file(input: &str, output: &str, replacement_map: HashMap<u8, (u8, u8)>) {
    let input = StdFile::open(input).expect("Could not open file");
    let mut reader = BufReader::new(input);
    let output = StdFile::create(output).expect("could not create output file");
    let mut writer = BufWriter::new(output);

    let mut buffer = Vec::new();
    let _bytes_read = reader.read_to_end(&mut buffer);
    let mut iter = buffer.iter();
    while let Some(&character) = iter.next() {
        let value = match replacement_map.get(&character) {
            Some(tuple) => vec![tuple.0, tuple.1],
            None => vec![character],
        };
        writer
            .write_all(&value)
            .expect("Could not write to output file");
    }
}
