use std::io;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
enum IOBES {
    I,
    O,
    B,
    E,
    S,
}

impl FromStr for IOBES {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "I" => Ok(IOBES::I),
            "O" => Ok(IOBES::O),
            "B" => Ok(IOBES::B),
            "E" => Ok(IOBES::E),
            "S" => Ok(IOBES::S),
            _ => Err(()),
        }
    }
}

struct Token {
    token: String,
    chunk: String,
    ner: String,
    srls: Vec<String>,
}

impl Token {
    fn new(parts: Vec<&str>) -> Token {
        Token {
            token: parts[0].to_string(),
            chunk: parts[4].to_string(),
            ner: parts[5].to_string(),
            srls: parts[7..].to_vec().iter().map(|s| s.to_string()).collect(),
        }
    }
}

fn main() {
    let mut block = Vec::new();
    let input = io::stdin();
    let mut buffer = String::new();

    while input.read_line(&mut buffer).unwrap() > 0 {
        let line = buffer.trim();
        if line.is_empty() {
            process(&block);
            block.clear();
        } else {
            block.push(line.to_string());
        }
        buffer.clear();
    }

    if !block.is_empty() {
        process(&block);
    }
}

fn process(block: &[String]) {
    let tokens: Vec<Token> = block
        .iter()
        .map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            Token::new(parts)
        })
        .collect();

    let srls_transposed = transpose(&tokens.iter().map(|t| t.srls.clone()).collect());

    let offsets: Vec<(usize, usize)> = tokens
        .iter()
        .scan(0, |last_end, token| {
            let start = *last_end;
            let end = start + token.token.len();
            *last_end = end;
            Some((start, end))
        })
        .collect();

    let sentence: String = offsets
        .iter()
        .zip(tokens.iter().map(|t| &t.token))
        .map(|((start, end), token)| {
            let space = " ".repeat(*start - *end);
            space + token
        })
        .collect();

    let mut phrases = Vec::new();

    for (i, tok) in tokens.iter().map(|t| &t.chunk).enumerate() {
        if *tok == "O" {
            continue;
        }

        let (iobes, label) = tok.split_at(1);
        let label = label.trim_matches(|c| c == '-' || c == '+');

        match IOBES::from_str(iobes) {
            Ok(IOBES::B) => phrases.push(i),
            Ok(IOBES::E) => {
                if let Some(last) = phrases.last() {
                    let start = *last;
                    let end = i + 1;
                    println!(
                        "{0}\t{1}",
                        label,
                        &sentence[offsets[start].0..offsets[end - 1].1]
                    );
                }
                phrases.clear();
            }
            Ok(IOBES::S) | Ok(IOBES::I) => {}
            _ => (),
        }
    }

    for (i, tok) in tokens.iter().map(|t| &t.ner).enumerate() {
        if *tok == "O" {
            continue;
        }

        let (iobes, label) = tok.split_at(1);
        let label = label.trim_matches(|c| c == '-' || c == '+');

        match IOBES::from_str(iobes) {
            Ok(IOBES::B) => phrases.push(i),
            Ok(IOBES::E) => {
                if let Some(last) = phrases.last() {
                    let start = *last;
                    let end = i + 1;
                    println!(
                        "{0}\t{1}",
                        label,
                        &sentence[offsets[start].0..offsets[end - 1].1]
                    );
                }
                phrases.clear();
            }
            Ok(IOBES::S) | Ok(IOBES::I) => {}
            _ => (),
        }
    }

    for srl_labels in srls_transposed.iter() {
        let mut phrases = Vec::new();

        for (i, tok) in srl_labels.iter().enumerate() {
            if *tok == "O" {
                continue;
            }

            let (iobes, label) = tok.split_at(1);
            let label = label.trim_matches(|c| c == '-' || c == '+');

            match IOBES::from_str(iobes) {
                Ok(IOBES::B) => phrases.push(i),
                Ok(IOBES::E) => {
                    if let Some(last) = phrases.last() {
                        let start = *last;
                        let end = i + 1;
                        println!(
                            "{0}\t{1}",
                            label,
                            &sentence[offsets[start].0..offsets[end - 1].1]
                        );
                    }
                    phrases.clear();
                }
                Ok(IOBES::S) | Ok(IOBES::I) => {}
                _ => (),
            }
        }
    }
}

fn transpose<T: Clone>(matrix: &Vec<Vec<T>>) -> Vec<Vec<T>> {
    if matrix.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let row_len = matrix[0].len();

    for i in 0..row_len {
        let mut row = Vec::new();
        for j in 0..matrix.len() {
            row.push(matrix[j][i].clone());
        }
        result.push(row);
    }

    result
}
