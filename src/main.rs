use std::collections::HashMap;
use std::fs::File as StdFile;
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use clap::error::ContextKind;
use clap::{Command, Parser, Subcommand};
use encode::writer;
use ndarray::{Array1, Array2};

mod encode;

#[derive(Parser)]
#[command(name = "Character Gather")]
#[command(version = "0.0.2")]
#[command(about = "Takes in text files and analyses how often character appear after each other", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Encod {
        #[clap(short, long)]
        input: String,
        #[arg(short, long, default_value_t = String::from("encoded.rtc"))]
        output: String,
        #[arg(short, long, default_value_t = 4096)]
        buffer_size: usize,
        #[arg(short, long, default_value_t = 2)]
        threads: usize,
        #[clap(short, long, value_parser, num_args = 1.., value_delimiter = ',')]
        equivalents: Vec<char>,
    },
    Decode {
        #[clap(short, long)]
        input: String,
        #[arg(short, long, default_value_t = String::from("decoded.txt"))]
        output: String,
        #[arg(short, long, default_value_t = 4096)]
        buffer_size: usize,
        #[arg(short, long, default_value_t = 2)]
        threads: usize,
    },
}

fn main() {
    let args = Args::parse();
    match args.command {
        Commands::Encod {
            input,
            output,
            buffer_size,
            threads,
            equivalents,
        } => {
            if equivalents.len() % 2 != 0 {
                eprintln!("You gave an uneven amount of replacement characters");
            }
            let equivalents_map: HashMap<char, char> = equivalents
                .chunks_exact(2)
                .map(|pair| (pair[0], pair[1]))
                .collect();

            let replacement_map = encode::create_replacement_map(
                equivalents_map.clone(),
                input.clone(),
                buffer_size,
                threads,
            );
            println!("Finished analyzing");
            println!("replacement map: {:#?}", replacement_map);
            encode::writer(input, output, equivalents_map, replacement_map);
            println!("Finished compressing file");
        }
        Commands::Decode {
            input,
            output,
            buffer_size,
            threads,
        } => {
            let input = StdFile::open(input).expect("Could not open file");
            let mut reader = BufReader::new(input);
            let output = StdFile::create(output).expect("could not create output file");
            let mut writer = BufWriter::new(output);

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
            println!("Replacement map: {:#?}", replacement_map);

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
    }
}
