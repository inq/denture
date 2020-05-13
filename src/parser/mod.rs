use std::io::BufRead;

#[derive(Debug)]
pub struct Parser {
    lines: Vec<(usize, String)>,
}

impl Parser {
    pub fn from_reader<R>(reader: R) -> Result<Self, failure::Error>
    where
        R: BufRead,
    {
        Ok(Self {
            lines: reader
                .lines()
                .enumerate()
                .map(|(idx, line)| line.map(|line| (idx, line)))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}
