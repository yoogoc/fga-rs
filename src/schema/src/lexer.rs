use std::{fmt, str::CharIndices};

use itertools::{peek_nth, PeekNth};
use phf::phf_map;
use unicode_xid::UnicodeXID;

use crate::{pos::Loc, token::Token};

pub type Spanned<Token, Loc, Error> = Result<(Loc, Token, Loc), Error>;

pub struct Lexer<'input> {
    input: &'input str,
    chars: PeekNth<CharIndices<'input>>,
    comments: &'input mut Vec<(Loc, String)>,
    location: usize,
    at_begin_of_line: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LexicalError {
    UnrecognisedToken(Loc, String),
}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexicalError::UnrecognisedToken(_, t) => write!(f, "unrecognised token '{}'", t),
        }
    }
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str, comments: &'input mut Vec<(Loc, String)>) -> Self {
        Lexer {
            input,
            comments,
            location: 0,
            at_begin_of_line: true,
            chars: peek_nth(input.char_indices()),
        }
    }

    fn next_char(&mut self) -> Option<(usize, char)> {
        self.location += 1;
        self.chars.next()
    }

    fn inner_next(&mut self) -> Option<Result<(usize, Token<'input>, usize), LexicalError>> {
        loop {
            self.at_begin_of_line = false;
            match self.next_char() {
                Some((i, ch)) if UnicodeXID::is_xid_start(ch) || ch == '*' => {
                    let (id, end) = self.match_identifier(i);
                    return if let Some(w) = KEYWORDS.get(id) {
                        Some(Ok((i, *w, end)))
                    } else {
                        Some(Ok((i, Token::Identifier(id), end)))
                    };
                }
                Some((i, '{')) => return Some(Ok((i, Token::LBrace, i + 1))),
                Some((i, '}')) => return Some(Ok((i, Token::RBrace, i + 1))),
                Some((i, ':')) => return Some(Ok((i, Token::Colon, i + 1))),
                Some((i, '(')) => return Some(Ok((i, Token::LBracket, i + 1))),
                Some((i, ')')) => return Some(Ok((i, Token::RBracket, i + 1))),
                Some((i, '^')) => return Some(Ok((i, Token::Caret, i + 1))),
                Some((i, '&')) => return Some(Ok((i, Token::And, i + 1))),
                Some((i, '|')) => return Some(Ok((i, Token::Or, i + 1))),
                Some((i, '+')) => return Some(Ok((i, Token::Add, i + 1))),
                Some((i, '-')) => {
                    let peek = self.chars.peek();
                    if matches!(peek, Some((_, '>'))) {
                        return Some(Ok((i, Token::YulArrow, i + 2)));
                    } else {
                        return Some(Ok((i, Token::Sub, i + 1)));
                    }
                }
                Some((i, '#')) => return Some(Ok((i, Token::Sharp, i + 1))),
                Some((_, ' ')) | Some((_, '\t')) | Some((_, '\x0C')) => (),
                Some((i, '\n')) | Some((i, '\r')) => {
                    self.at_begin_of_line = true;
                    return Some(Ok((i, Token::Newline, i + 1)));
                }
                Some((_, '/')) => {
                    if let Err(err) = self.eat_comment() {
                        return Some(Err(err));
                    }
                }
                Some((start, _)) => {
                    let mut end;
                    loop {
                        if let Some((i, ch)) = self.next_char() {
                            end = i;
                            if ch.is_whitespace() {
                                break;
                            }
                        } else {
                            end = self.input.len();
                            break;
                        }
                    }
                    return Some(Err(LexicalError::UnrecognisedToken(
                        (start, end),
                        self.input[start..end].to_owned(),
                    )));
                }
                None => return None,
            }
        }
    }

    fn match_identifier(&mut self, start: usize) -> (&'input str, usize) {
        let end;
        loop {
            if let Some((i, ch)) = self.chars.peek() {
                if !UnicodeXID::is_xid_continue(*ch) && *ch != '*' {
                    end = *i;
                    break;
                }
                self.next_char();
            } else {
                end = self.input.len();
                break;
            }
        }

        (&self.input[start..end], end)
    }

    fn eat_comment(&mut self) -> Result<(), LexicalError> {
        let i = self.location - 1;
        let peek = self.chars.peek();
        if matches!(peek, Some((_, '/'))) {
            let end;
            loop {
                let peek = self.chars.peek();
                match peek {
                    None => {
                        end = self.input.len();
                        self.next_char();
                        break;
                    }
                    Some((offset, '\n' | '\r')) => {
                        end = offset.clone();
                        break;
                    }
                    Some(_) => {
                        self.next_char();
                    }
                }
            }
            self.comments.push(((i, end), self.input[i..end].to_owned()));
        } else {
            return Err(LexicalError::UnrecognisedToken((i, i + 1), "/".to_owned()));
        }
        Ok(())
    }

    fn eat_indentation(&mut self) -> Result<(), LexicalError> {
        loop {
            match self.chars.peek() {
                Some((_, ' ')) | Some((_, '\t')) | Some((_, '\x0C')) => {
                    self.next_char();
                }
                Some((_, '/')) => {
                    self.next_char();
                    self.eat_comment()?;
                }
                Some((_, '\n')) | Some((_, '\r')) => {
                    self.next_char();
                }
                Some(_) => {
                    self.at_begin_of_line = false;
                    break;
                }
                None => {
                    break;
                }
            }
        }
        Ok(())
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token<'input>, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.at_begin_of_line {
            if let Err(err) = self.eat_indentation() {
                return Some(Err(err));
            }
        }
        let token = self.inner_next();

        // trace!

        token
    }
}

static KEYWORDS: phf::Map<&'static str, Token> = phf_map! {
    "type" => Token::Type,
    "relation" => Token::Relation,
    "permission" => Token::Permission,
    "cond" => Token::Cond,
};
