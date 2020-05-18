mod line;

use line::LineTokenizer;

#[derive(Debug)]
pub enum Operator {
    Colon,
    Plus,
}

#[derive(Debug)]
pub enum Token {
    Comment(String),
    Identifier(String),
    Whitespaces(String),
    Operator(Operator),
    BinNumber(String),
    OctNumber(String),
    DecNumber(String),
    HexNumber(String),
}

#[derive(Debug)]
pub struct Tokenizer {
    lines: Vec<LineTokenizer>,
}

impl Tokenizer {
    pub fn from_reader<R>(reader: R) -> Result<Self, failure::Error>
    where
        R: std::io::BufRead,
    {
        let lines = reader
            .lines()
            .map(|line| LineTokenizer::from_str(&line?).map_err(|err| failure::Error::from(err)))
            .collect::<Result<_, _>>()?;

        Ok(Self { lines })
    }
}
