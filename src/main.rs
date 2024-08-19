use std::collections::HashMap;
use std::fs::File as StdFile;
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use clap::error::ContextKind;
use clap::{Command, Parser, Subcommand};
use encode::writer;
use ndarray::{Array1, Array2};

mod decode;
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
            let replacement_map = decode::read_replacement_map(&input);
            decode::decode_file(&input, &output, replacement_map);
        }
    }
}
