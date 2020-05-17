use super::{Operator, Token};
use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "invalid character: {:?}, {}", state, c)]
    InvalidCharacter { state: String, c: char },
    #[fail(display = "invalid terminal state: {:?}", state)]
    InvalidTerminalState { state: String },
}

#[derive(Debug)]
enum StringQuote {
    Single,
    Double,
    Single3,
    Double3,
}

#[derive(Debug, Copy, Clone)]
enum NumberType {
    Hex,
    Oct,
    Bin,
    Dec,
}

#[derive(Debug, Copy, Clone)]
enum NumberState {
    Normal,
    Underscore,
}

#[derive(Debug)]
enum State {
    Indent,
    Identifier(usize),
    Comment(usize),
    Zero,
    ZeroPadded(usize),
    StringPrefixSingle,
    StringPrefixDouble,
    String(StringQuote, usize),
    Whitespaces(usize),
    Number(NumberType, NumberState, usize),
    Empty,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug)]
pub struct LineTokenizer {
    offset: usize,
    tokens: Vec<Token>,
}

#[inline]
fn match_first_char(index: usize, c: char) -> Option<State> {
    Some(match c.to_ascii_lowercase() {
        '#' => State::Comment(index),
        'b' | 'f' | 'r' | 'u' => State::StringPrefixSingle,
        '0' => State::Zero,
        c if c.is_numeric() => State::Number(NumberType::Dec, NumberState::Normal, index),
        c if unicode_xid::UnicodeXID::is_xid_start(c) => State::Identifier(index),
        _ => return None,
    })
}

impl LineTokenizer {
    pub fn from_str(input: &str) -> Result<Self, Error> {
        let mut state = State::Indent;
        let mut offset = 0;
        let mut tokens = vec![];

        for (index, c) in input.char_indices() {
            state = match (state, c) {
                (State::Indent, ' ') => State::Indent,
                (s @ State::Indent, c) => {
                    offset = index;
                    match_first_char(index, c).ok_or(Error::InvalidCharacter {
                        state: s.to_string(),
                        c,
                    })?
                }
                (ref s @ State::Whitespaces(starts_at), c) => {
                    tokens.push(Token::Whitespaces(input[starts_at..index].to_string()));
                    match_first_char(index, c).ok_or(Error::InvalidCharacter {
                        state: s.to_string(),
                        c,
                    })?
                }
                (s @ State::Zero, c) => match c {
                    'x' | 'X' => State::Number(NumberType::Hex, NumberState::Normal, index - 1),
                    'b' | 'B' => State::Number(NumberType::Bin, NumberState::Normal, index - 1),
                    'o' | 'O' => State::Number(NumberType::Oct, NumberState::Normal, index - 1),
                    '0' => State::ZeroPadded(index - 1),
                    c => {
                        return Err(Error::InvalidCharacter {
                            state: s.to_string(),
                            c,
                        })
                    }
                },
                (State::StringPrefixSingle, c) => {
                    let p = input[index - 1..].chars().next().unwrap();
                    match (p.to_ascii_lowercase(), c.to_ascii_lowercase()) {
                        ('b' | 'f', 'r') | ('r', 'b' | 'f') => State::StringPrefixDouble,
                        (p, '\'') => State::String(StringQuote::Single, index - 1),
                        (p, '\"') => State::String(StringQuote::Double, index - 1),
                        (p, c) => State::Identifier(index - 1),
                    }
                }
                (ref s @ State::Number(NumberType::Dec, number_state, starts_at), c) => {
                    match (c, number_state) {
                        (c, _) if c.is_numeric() => {
                            State::Number(NumberType::Dec, NumberState::Normal, starts_at)
                        }
                        ('_', NumberState::Normal) => {
                            State::Number(NumberType::Dec, NumberState::Underscore, starts_at)
                        }
                        (' ', NumberState::Normal) => {
                            // TODO: Support operators
                            tokens.push(Token::DecNumber(input[starts_at..index].to_string()));
                            State::Whitespaces(index)
                        }
                        _ => {
                            return Err(Error::InvalidCharacter {
                                state: s.to_string(),
                                c,
                            })
                        }
                    }
                }
                (ref s @ State::Identifier(starts_at), c) => match c {
                    ' ' => {
                        tokens.push(Token::Identifier(input[starts_at..index].to_string()));
                        State::Whitespaces(index)
                    }
                    c if unicode_xid::UnicodeXID::is_xid_continue(c) => {
                        State::Identifier(starts_at)
                    }
                    ':' => {
                        tokens.push(Token::Identifier(input[starts_at..index].to_string()));
                        tokens.push(Token::Operator(Operator::Colon));
                        State::Empty
                    }
                    c => {
                        return Err(Error::InvalidCharacter {
                            state: s.to_string(),
                            c,
                        });
                    }
                },
                (State::Comment(starts_at), _) => State::Comment(starts_at),
                (state, c) => {
                    return Err(Error::InvalidCharacter {
                        state: state.to_string(),
                        c,
                    });
                }
            }
        }
        match state {
            State::Comment(starts_at) => {
                tokens.push(Token::Comment(input[starts_at..].to_string()))
            }

            State::Number(NumberType::Dec, NumberState::Normal, starts_at) => {
                tokens.push(Token::DecNumber(input[starts_at..].to_string()))
            }
            State::Indent | State::Whitespaces(_) | State::Empty => (),
            State::Identifier(starts_at) => {
                tokens.push(Token::Identifier(input[starts_at..].to_string()))
            }
            state => {
                return Err(Error::InvalidTerminalState {
                    state: state.to_string(),
                })
            }
        }

        Ok(Self { offset, tokens })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_number() {
        match LineTokenizer::from_str("100").unwrap().tokens.as_slice() {
            [Token::DecNumber(num)] => {
                assert_eq!(num, "100", "{}", num);
            }
            etc => {
                panic!("{:?}", etc);
            }
        }

        match LineTokenizer::from_str("100_000_000")
            .unwrap()
            .tokens
            .as_slice()
        {
            [Token::DecNumber(num)] => {
                assert_eq!(num, "100_000_000", "{}", num);
            }
            etc => {
                panic!("{:?}", etc);
            }
        }

        assert!(LineTokenizer::from_str("100_000_000_").is_err());
    }
}
