use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::str::FromStr;
use std::convert::TryInto;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} input_file alignment_file output_file [max_ngram]", args[0]);
        std::process::exit(1);
    }

    let input_file = &args[1];
    let alignment_file = &args[2];
    let output_file = &args[3];

    let max_ngram = args.get(4)
        .map(|max_ngram_str| u32::from_str(max_ngram_str).unwrap_or(10))
        .unwrap_or(10);

    let input_f = BufReader::new(File::open(input_file)?);
    let alignment_f = BufReader::new(File::open(alignment_file)?);
    let mut output_f = BufWriter::new(File::create(output_file)?);

    process_lines(input_f, alignment_f, &mut output_f, max_ngram)?;

    println!("Output written to: {}", output_file);
    Ok(())
}

fn process_lines<R1, R2, W>(
    input_f: R1,
    alignment_f: R2,
    output_f: &mut W,
    max_ngram: u32,
) -> io::Result<()>
where
    R1: BufRead,
    R2: BufRead,
    W: Write,
{
    for (pair, alignment) in input_f.lines().zip(alignment_f.lines()) {
        let (source, target) = match (pair, alignment) {
            (Ok(src), Ok(align)) => (src, align),
            _ => continue,
        };

        let source = source.clone(); // Clone source to avoid borrowing issues

        let parts: Vec<&str> = source.trim().split(" ||| ").collect();
        if parts.len() != 2 {
            continue;
        }

        for (a, b) in phrase_extraction(parts[0], parts[1], &target) {
            let a_tokens = whitespace_tokenizer(&a);
            let b_tokens = whitespace_tokenizer(&b);

            let a_len = a_tokens.len();
            let b_len = b_tokens.len();

            if 1 <= a_len as u32 && a_len as u32 <= max_ngram
                && 1 <= b_len as u32 && b_len as u32 <= max_ngram
            {
                writeln!(output_f, "{} ||| {}", a_tokens.join(" "), b_tokens.join(" ")).unwrap();
            }
        }
    }
    Ok(())
}

fn convert_alignment(a: &str) -> Vec<(i32, i32)> {
    a.split(' ')
        .map(|x| {
            let parts: Vec<&str> = x.split('-').collect();
            if parts.len() == 2 {
                (
                    parts[0].parse().unwrap_or(0),
                    parts[1].parse().unwrap_or(0),
                )
            } else {
                (0, 0)
            }
        })
        .collect()
}

fn whitespace_tokenizer<'a>(text: &'a str) -> Vec<&'a str> {
    text.split_whitespace().collect()
}

fn phrase_extraction(srctext: &str, trgtext: &str, alignment: &str) -> Vec<(String, String)> {
    let src_tokens = srctext.split_whitespace().collect::<Vec<&str>>();
    let trg_tokens = trgtext.split_whitespace().collect::<Vec<&str>>();
    let alignment_vec = convert_alignment(alignment);

    let srclen = src_tokens.len();
    let trglen = trg_tokens.len();

    let f_aligned: Vec<i32> = alignment_vec.iter().map(|(_, f)| *f).collect();

    let mut extracted_phrases: Vec<(String, String)> = Vec::new();

    for e_start in 0..srclen {
        for e_end in e_start..srclen {
            let mut f_start = trglen as i32 - 1;
            let mut f_end = -1i32;

            for &(e, f) in &alignment_vec {
                if e_start as i32 <= e && e <= e_end as i32 && f_start <= f && f <= f_end.try_into().unwrap() {
                    f_start = f_start.min(f);
                    f_end = f_end.max(f);
                }
            }

            if f_end < 0 {
                break;
            }

            for &(e, f) in &alignment_vec {
                if e_start as i32 <= e && e <= e_end as i32 {
                    if f_start <= f && f <= f_end.try_into().unwrap() {
                        let mut fe = f_end;
                        while fe < trglen as i32 {
                            let src_phrase: String = src_tokens[e_start..=e_end].join(" ");
                            let trg_phrase: String = trg_tokens[f_start as usize..=fe as usize].join(" ");
                            extracted_phrases.push((src_phrase, trg_phrase));
                            fe += 1;
                            if f_aligned.contains(&(fe as i32)) || fe == trglen as i32 {
                                break;
                            }
                        }
                        f_start -= 1;
                        if f_aligned.contains(&(f_start as i32)) || f_start < 0 {
                            break;
                        }
                    }
                }
            }
        }
    }

    extracted_phrases
}
