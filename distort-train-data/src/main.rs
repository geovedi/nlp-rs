use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
extern crate log;
extern crate rand;
use log::info;
use rand::prelude::SliceRandom;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <input_fname> <output_fname>", args[0]);
        return Ok(());
    }

    let input_fname = &args[1];
    let output_fname = &args[2];

    let input_file = File::open(input_fname)?;
    let input_reader = BufReader::new(input_file);

    let output_file = File::create(output_fname)?;
    let mut output_writer = BufWriter::new(output_file);

    for (line_no, line) in input_reader.lines().enumerate() {
        let line = line?;
        let parts: Vec<&str> = line.trim().split(" ||| ").collect();

        if parts.len() != 2 {
            continue;
        }

        let src = parts[0];
        let tgt = parts[1];
        let src_tok: Vec<&str> = src.split_whitespace().collect();
        let src_len = src_tok.len();

        // 1. always add the original pair
        writeln!(output_writer, "{} ||| {}", src, tgt)?;

        // 2. distort src but keep tgt
        for &p in &[0.1, 0.2, 0.3] {
            let mut dist_src_tok: Vec<String> = src_tok.iter().map(|&s| s.to_string()).collect();
            let mut rng = rand::thread_rng();

            for _ in 0..(src_len as f64 * p).round() as usize {
                if let Some(i) = dist_src_tok.choose_mut(&mut rng) {
                    *i = "<unk>".to_string();
                }
            }

            writeln!(output_writer, "{} ||| {}", dist_src_tok.join(" "), tgt)?;
        }

        if line_no % 10000 == 0 {
            info!("processing pair: {}", line_no);
        }
    }

    Ok(())
}
