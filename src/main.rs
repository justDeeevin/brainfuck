#![feature(ascii_char)]

use rustyline::error::ReadlineError;
use std::{collections::HashMap, fs::File, io::Read};

type Key = i32;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    let mut buf = HashMap::new();
    let mut ptr = 0;

    if let Some(path) = args.get(1) {
        let mut file = File::open(path).unwrap();
        let mut code = String::new();
        file.read_to_string(&mut code).unwrap();
        eval(&code, 0, &mut buf, &mut ptr);
    } else {
        loop {
            let mut rl = rustyline::DefaultEditor::new().unwrap();
            let line = rl.readline(">> ");
            match line {
                Ok(line) => {
                    eval(&line, 0, &mut buf, &mut ptr);
                    println!();
                }
                Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                    break;
                }
                Err(err) => {
                    panic!("{}", err);
                }
            }
        }
    }
}

fn eval(code: &str, mut col_offset: usize, buf: &mut HashMap<Key, u8>, ptr: &mut i32) {
    let mut looped: Option<Vec<char>> = None;

    for (line, column, char) in code.lines().enumerate().flat_map(|(line, s)| {
        s.chars()
            .enumerate()
            .map(move |(column, c)| (line, column, c))
    }) {
        if let Some(looped) = &mut looped {
            looped.push(char);
        }

        match char {
            '>' => {
                *ptr += 1;
            }
            '<' => {
                *ptr -= 1;
            }
            '+' => {
                buf.entry(*ptr).and_modify(|v| *v += 1).or_insert(1);
            }
            '-' => {
                buf.entry(*ptr)
                    .and_modify(|v| {
                        if *v > 0 {
                            *v -= 1
                        }
                    })
                    .or_insert(0);
            }
            '.' => {
                print!("{}", buf.get(ptr).copied().unwrap_or(0) as char);
            }
            ',' => {
                let input = console::Term::stdout().read_char().unwrap();
                print!("{}", input);
                buf.insert(*ptr, input as u8);
            }
            '[' => {
                looped = Some(Vec::new());
            }
            ']' => {
                let Some(code) = &looped else {
                    panic!(
                        "Unmatched ']' (at line {} column {})",
                        line,
                        column + col_offset
                    );
                };
                let code = code.iter().take(code.len() - 1).collect::<String>();
                while !(buf.get(ptr).is_none() || buf.get(ptr) == Some(&0)) {
                    eval(&code, column + col_offset, buf, ptr);
                }
                looped = None;
            }
            _ => continue,
        }
        col_offset = 0;
    }
}
