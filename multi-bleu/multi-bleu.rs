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
        let reference_lines: Vec<String> = reference
            .lines()
            .map(|line| {
                if lowercase {
                    line.to_lowercase()
                } else {
                    line.to_string()
                }
            })
            .collect();

        if reference_lines.is_empty() {
            break; // Stop reading when an empty reference is encountered
        }

        references.push(reference_lines);
        ref_index += 1;
    }

    let mut correct: Vec<u64> = vec![0; MAX_NGRAM];
    let mut total: Vec<u64> = vec![0; MAX_NGRAM]; // Make total mutable
    let mut length_translation: usize = 0;
    let mut length_reference: usize = 0;
    let mut sentence_index = 0;

    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut line = String::new();

    while stdin_lock.read_line(&mut line)? > 0 {
        let sentence = if lowercase {
            line.to_lowercase()
        } else {
            line.to_string()
        };
        let words: Vec<&str> = sentence.trim().split_whitespace().collect();

        let (_, closest_length) = references[sentence_index]
            .iter()
            .map(|reference| {
                let reference_words: Vec<&str> = reference.trim().split_whitespace().collect();
                let length = reference_words.len() as isize;
                (length, reference_words)
            })
            .min_by_key(|(length, _)| (words.len() as isize - length).abs())
            .unwrap_or((9999, vec![]));

        for ref_sentence in references[sentence_index].iter() {
            let reference_words: Vec<&str> = ref_sentence.trim().split_whitespace().collect();

            for ngram in 1..=MAX_NGRAM {
                let mut ref_ngram = vec![0u64; MAX_NGRAM];
                for start in 0..=words.len() - ngram {
                    let ngram_words: Vec<&str> = words[start..start + ngram].to_vec();
                    let ngram_string = ngram_words.join(" ");
                    if reference_words
                        .windows(ngram)
                        .any(|window| window.join(" ") == ngram_string)
                    {
                        ref_ngram[ngram - 1] += 1;
                    }
                }

                for (i, &count) in ref_ngram.iter().enumerate() {
                    correct[i] = correct[i].max(count);
                }

                total[ngram - 1] += ngram as u64;
            }
        }

        length_translation += words.len();
        length_reference += closest_length.len();

        sentence_index += 1;
        line.clear();
    }

    let brevity_penalty = if length_translation < length_reference {
        (-1.0 * (length_reference as f64) / (length_translation as f64)).exp()
    } else {
        1.0
    };

    let bleu_scores: Vec<f64> = (1..=MAX_NGRAM)
        .map(|n| {
            if total[n - 1] > 0 {
                correct[n - 1] as f64 / total[n - 1] as f64
            } else {
                0.0
            }
        })
        .collect();

    let bleu = brevity_penalty * (bleu_scores.iter().sum::<f64>() / (MAX_NGRAM as f64)).exp();

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
