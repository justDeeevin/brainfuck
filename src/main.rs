use rustyline::error::ReadlineError;
use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    io::Read,
};

type Key = i32;

#[derive(Debug)]
struct Token {
    pub line: usize,
    pub column: usize,
    pub token_type: TokenType,
}

#[derive(Debug, PartialEq, Eq)]
enum TokenType {
    Right,
    Left,
    Plus,
    Minus,
    Out,
    In,
    JumpForward(Option<usize>),
    JumpBackward(Option<usize>),
    Ignore,
}

impl From<char> for TokenType {
    fn from(value: char) -> Self {
        match value {
            '>' => TokenType::Right,
            '<' => TokenType::Left,
            '+' => TokenType::Plus,
            '-' => TokenType::Minus,
            '.' => TokenType::Out,
            ',' => TokenType::In,
            '[' => TokenType::JumpForward(None),
            ']' => TokenType::JumpBackward(None),
            _ => TokenType::Ignore,
        }
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    let mut buf = HashMap::new();
    let mut head = 0;

    if let Some(path) = args.get(1) {
        let mut file = File::open(path).unwrap();
        let mut code = String::new();
        file.read_to_string(&mut code).unwrap();
        run(&parse(&code), &mut buf, &mut head).unwrap();
    } else {
        println!("Brainfuck REPL");
        println!("Ctrl-D to exit");
        println!("Send \"!\" to reset");
        loop {
            let mut rl = rustyline::DefaultEditor::new().unwrap();
            let line = rl.readline(">> ");
            match line {
                Ok(line) => {
                    if line == "!" {
                        buf.clear();
                        head = 0;
                        continue;
                    }
                    if let Err(e) = run(&parse(&line), &mut buf, &mut head) {
                        println!("{}", e);
                    } else {
                        println!();
                    }
                }
                Err(ReadlineError::Eof) => {
                    break;
                }
                Err(ReadlineError::Interrupted) => {
                    continue;
                }
                Err(err) => {
                    panic!("{}", err);
                }
            }
        }
    }
}

fn parse(code: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut opens = VecDeque::new();
    for (i, (line, column, char)) in code
        .lines()
        .enumerate()
        .flat_map(|(line, s)| {
            s.chars()
                .enumerate()
                .map(move |(column, c)| (line + 1, column + 1, c))
        })
        .enumerate()
    {
        let mut token_type = char.into();
        if token_type == TokenType::JumpForward(None) {
            opens.push_back(i);
        }
        if token_type == TokenType::JumpBackward(None) {
            if let Some(back) = opens.pop_back() {
                tokens[back].token_type = TokenType::JumpForward(Some(i + 1));
                token_type = TokenType::JumpBackward(Some(back + 1));
            }
        }
        let token = Token {
            line,
            column,
            token_type,
        };
        tokens.push(token);
    }

    tokens
}

fn run(tokens: &[Token], buf: &mut HashMap<Key, u8>, head: &mut Key) -> Result<(), String> {
    let mut instruction = 0;
    while let Some(token) = tokens.get(instruction) {
        match token.token_type {
            TokenType::Ignore => {}
            TokenType::Right => *head += 1,
            TokenType::Left => *head -= 1,
            TokenType::Plus => {
                buf.entry(*head)
                    .and_modify(|v| {
                        if *v == u8::MAX {
                            *v = 0;
                        } else {
                            *v += 1;
                        }
                    })
                    .or_insert(1);
            }
            TokenType::Minus => {
                buf.entry(*head)
                    .and_modify(|v| {
                        if *v == 0 {
                            *v = u8::MAX;
                        } else {
                            *v -= 1;
                        }
                    })
                    .or_insert(u8::MAX);
            }
            TokenType::Out => {
                print!("{}", buf.get(head).copied().unwrap_or(0) as char);
            }
            TokenType::In => {
                let input = console::Term::stdout().read_char().unwrap();
                print!("{}", input);
                buf.insert(*head, input as u8);
            }
            TokenType::JumpForward(Some(i)) => {
                if buf.get(head).copied().unwrap_or(0) == 0 {
                    instruction = i;
                    continue;
                }
            }
            TokenType::JumpBackward(Some(i)) => {
                if buf.get(head).copied().unwrap_or(0) != 0 {
                    instruction = i;
                    continue;
                }
            }
            TokenType::JumpForward(None) => {
                return Err(format!(
                    "Unmatched '[' (line {} column {})",
                    token.line, token.column
                ));
            }
            TokenType::JumpBackward(None) => {
                return Err(format!(
                    "Unmatched ']' (line {} column {})",
                    token.line, token.column
                ));
            }
        }

        instruction += 1;
    }

    Ok(())
}
