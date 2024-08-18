use ndarray::{Array1, Array2};
use std::collections::HashMap;
use std::fs::File as StdFile;
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub fn replace(text: Vec<u8>, equivalent_map: Arc<HashMap<char, char>>) -> Vec<u8> {
    return text
        .into_iter()
        .map(|c| match equivalent_map.get(&(c as char)) {
            Some(replaced) => *replaced as u8,
            None => c,
        })
        .collect();
}

pub fn count_combinations(text: &Vec<u8>) -> Array2<u64> {
    let mut final_arr = Array2::<u64>::zeros((128, 128));
    text.windows(2).for_each(|pair| {
        if pair[0] < 128 && pair[1] < 128 {
            let point = final_arr.get_mut((pair[0] as usize, pair[1] as usize));
            match point {
                Some(point) => *point += 1,
                None => eprintln!("data at point {}|{} is not accessible", pair[0], pair[1]),
            }
        }
    });

    return final_arr;
}

pub fn count_unused(text: &Vec<u8>) -> Array1<u64> {
    let mut final_arr = Array1::<u64>::zeros(128);

    text.into_iter().for_each(|c| {
        if *c < 128 {
            let point = final_arr.get_mut(*c as usize);
            match point {
                Some(point) => *point += 1,
                None => eprintln!("data {} not reachable", c),
            }
        }
    });

    return final_arr;
}

pub fn create_replacement_map(
    equivalents_map: HashMap<char, char>,
    input: String,
    buffer_size: usize,
    threads: usize,
) -> HashMap<(char, char), char> {
    let equivalents_map = Arc::new(equivalents_map);
    let file = StdFile::open(input).expect("Could not open input file");
    let file_size = file.metadata().expect("Could not open file").len();
    let num_chunks = (file_size as usize + buffer_size + 1) / buffer_size;

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("Could not create Threadpool");

    let (combi_tx, combi_rx) = mpsc::channel();
    let (unused_tx, unused_rx) = mpsc::channel();
    let file = Arc::new(Mutex::new(file)); // Wrap the file in a Mutex

    let thread_spawner = thread::spawn(move || {
        pool.scope(|s| {
            for i in 0..num_chunks {
                let file = Arc::clone(&file);
                let combi_tx = combi_tx.clone();
                let unused_tx = unused_tx.clone();
                let equivalents_map = Arc::clone(&equivalents_map);
                s.spawn(move |_| {
                    let mut chunk = vec![0; buffer_size.min(file_size as usize - i * buffer_size)];

                    let mut file = file.lock().unwrap(); // Lock the Mutex before accessing the file
                    file.seek(std::io::SeekFrom::Start((i * buffer_size) as u64))
                        .expect("Could not seek in file");
                    let _amount = file.read(&mut chunk).unwrap();
                    drop(file); // Optional: explicitly drop the lock early

                    let replaced = replace(chunk, equivalents_map);
                    combi_tx
                        .send(count_combinations(&replaced))
                        .expect("Could not send combi information");
                    unused_tx
                        .send(count_unused(&replaced))
                        .expect("could not send unused information");
                })
            }
        })
    });
    let mut final_combinations = Array2::<u64>::zeros((128, 128));
    let mut final_unused = Array1::<u64>::zeros(128);
    for combinations in combi_rx {
        final_combinations += &combinations;
    }
    for unused in unused_rx {
        final_unused += &unused;
    }
    thread_spawner
        .join()
        .expect("Could not wait for threadspawner");
    let mut combination_rank: Vec<(usize, usize, u64)> = final_combinations
        .indexed_iter()
        .map(|((x, y), &value)| (x, y, value))
        .collect();
    combination_rank.sort_unstable_by_key(|(_, _, value)| std::cmp::Reverse(*value));
    println!("All sorted combinations: {:#?}", combination_rank);
    let replacement_map: HashMap<(char, char), char> = final_unused
        .indexed_iter()
        .filter(|(_index, &value)| value == 0)
        .zip(combination_rank.into_iter())
        .map(|((c, _), (replace1, replace2, _))| {
            (
                (replace1 as u8 as char, replace2 as u8 as char),
                c as u8 as char,
            )
        })
        .collect();

    return replacement_map;
}

pub fn writer(
    input: String,
    output: String,
    equivalents_map: HashMap<char, char>,
    replacement_map: HashMap<(char, char), char>,
) {
    let input = StdFile::open(input).expect("Could not open file");
    let mut reader = BufReader::new(input);
    let output = StdFile::create(output).expect("could not create output file");
    let mut writer = BufWriter::new(output);

    let mut buffer = Vec::new();

    let bytes_read = reader.read_to_end(&mut buffer);
    let mut iter = buffer.iter().peekable();
    let mut skipper = false;
    while let Some(&first) = iter.next() {
        if skipper {
            skipper = false;
            continue;
        }
        let first = match equivalents_map.get(&(first as char)) {
            Some(replacement) => *replacement,
            None => first as char,
        };
        if let Some(&&second) = iter.peek() {
            let second = match equivalents_map.get(&(second as char)) {
                Some(replacement) => *replacement,
                None => second as char,
            };

            writer
                .write_all(&[match replacement_map.get(&(first, second)) {
                    Some(replacement) => {
                        skipper = true;
                        *replacement as u8
                    }
                    None => first as u8,
                }])
                .expect("Could not write to file");
            continue;
        }
        writer
            .write_all(&[first as u8])
            .expect("Could not write last to file");
    }
    writer.flush().expect("Could not flush buffer");
}
