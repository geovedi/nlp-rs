use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::process::exit;

const MAX_NGRAM: usize = 4;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} [-lc] reference < hypothesis", args[0]);
        eprintln!("Reads the references from reference or reference0, reference1, ...");
        exit(1);
    }

    let mut lowercase = false;
    let mut args_iter = args.iter().skip(1);

    if let Some(arg) = args_iter.next() {
        if arg == "-lc" {
            lowercase = true;
        } else {
            args_iter = args.iter().skip(1); // Reset iterator
        }
    }

    let stem = args_iter.next().unwrap_or_else(|| {
        eprintln!("usage: multi-bleu [-lc] reference < hypothesis");
        exit(1);
    });

    let mut references: Vec<Vec<String>> = Vec::new();
    let mut ref_index = 0;

    while let Ok(reference) = fs::read_to_string(format!("{}{}", stem, ref_index)) {
        let mut reference_lines: Vec<String> = Vec::new();
        for line in reference.lines() {
            let line = if lowercase { line.to_lowercase() } else { line.to_string() };
            reference_lines.push(line);
        }
        references.push(reference_lines);
        ref_index += 1;
    }

    if references.is_empty() {
        eprintln!("ERROR: could not find reference file {}", stem);
        exit(1);
    }

    let mut correct: Vec<u64> = vec![0; MAX_NGRAM];
    let total: Vec<u64> = vec![0; MAX_NGRAM];
    let (mut length_translation, mut length_reference) = (0, 0);
    let mut sentence_index = 0;

    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut line = String::new();

    while stdin_lock.read_line(&mut line)? > 0 {
        let sentence = if lowercase { line.to_lowercase() } else { line.to_string() };
        let words: Vec<&str> = sentence.trim().split_whitespace().collect();

        let mut closest_diff = 9999;
        let mut closest_length = 9999;

        for reference in &references[sentence_index] {
            let reference_words: Vec<&str> = reference.trim().split_whitespace().collect();
            let length = reference_words.len();
            let diff = (words.len() as isize - length as isize).abs();

            if diff < closest_diff {
                closest_diff = diff;
                closest_length = length;
            } else if diff == closest_diff {
                closest_length = closest_length.min(length);
            }

            let mut ref_ngram = vec![0u64; MAX_NGRAM];
            for n in 1..=MAX_NGRAM {
                let mut ref_ngram_n = vec![0u64; MAX_NGRAM];
                for _start in 0..=words.len() - n {
                    //let ngram: String = (1..=n)
                    //    .map(|i| words[_start + i - 1])
                    //    .collect::<Vec<&str>>()
                    //    .join(" ");
                    ref_ngram_n[n - 1] += 1;
                }
                for (ngram, &count) in ref_ngram_n.iter().enumerate() {
                    if ref_ngram[ngram] < count {
                        ref_ngram[ngram] = count;
                    }
                }
            }

            let mut t_ngram = vec![0u64; MAX_NGRAM];
            for n in 1..=MAX_NGRAM {
                let mut t_ngram_n = vec![0u64; MAX_NGRAM];
                for _start in 0..=words.len() - n {
                    //let ngram: String = (1..=n)
                    //    .map(|i| words[_start + i - 1])
                    //    .collect::<Vec<&str>>()
                    //    .join(" ");
                    t_ngram_n[n - 1] += 1;
                }
                for (ngram, &count) in t_ngram_n.iter().enumerate() {
                    t_ngram[ngram] += count;
                    if let Some(&ref_count) = ref_ngram.get(ngram) {
                        if ref_count >= count {
                            correct[n - 1] += count;
                        } else {
                            correct[n - 1] += ref_count;
                        }
                    }
                }
            }
        }

        length_translation += words.len();
        length_reference += closest_length;
        sentence_index += 1;
        line.clear();
    }

    let brevity_penalty = if length_translation < length_reference {
        (-1.0 * (length_reference as f64) / (length_translation as f64)).exp()
    } else {
        1.0
    };

    let mut bleu_scores: Vec<f64> = vec![0.0; MAX_NGRAM];

    for n in 1..=MAX_NGRAM {
        if total[n - 1] > 0 {
            bleu_scores[n - 1] = correct[n - 1] as f64 / total[n - 1] as f64;
        } else {
            bleu_scores[n - 1] = 0.0;
        }
    }

    let bleu = brevity_penalty
        * (bleu_scores.iter().sum::<f64>() / (MAX_NGRAM as f64)).exp();

    println!(
        "BLEU = {:.2}, {:.1}/{:.1}/{:.1}/{:.1} (BP={:.3}, ratio={:.3}, hyp_len={}, ref_len={})",
        100.0 * bleu,
        100.0 * bleu_scores[0],
        100.0 * bleu_scores[1],
        100.0 * bleu_scores[2],
        100.0 * bleu_scores[3],
        brevity_penalty,
        (length_translation as f64) / (length_reference as f64),
        length_translation,
        length_reference
    );

    Ok(())
}
