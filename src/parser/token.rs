use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub id: String,
    pub data: String,
    pub span: (usize, usize),
}

pub struct TokenStream {
    lines: Box<dyn Iterator<Item = String>>,
}

impl Debug for TokenStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TokenStream")
    }
}

impl TokenStream {
    pub fn from_file(path: impl AsRef<Path>) -> Self {
        let file = File::open(path).unwrap_or_else(|e| std::process::exit(1));
        let reader = BufReader::new(file);
        let lines = reader.lines().flatten().filter(|l| !l.is_empty());
        let lines = Box::new(lines);

        Self { lines }
    }

    fn next(&mut self) -> Option<Token> {
        let next_line = self.lines.next()?;

        let mut split = next_line.split_whitespace();

        let id = split.next()?.to_string();
        let data = split.next()?.to_string();
        let span = (
            split
                .next()
                .unwrap_or_else(|| std::process::exit(42))
                .parse()
                .unwrap(),
            split
                .next()
                .unwrap_or_else(|| std::process::exit(42))
                .parse()
                .unwrap(),
        );

        Some(Token { id, data, span })
    }
}

impl Iterator for TokenStream {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
