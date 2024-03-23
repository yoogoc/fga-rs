pub mod ast;
mod grammar;
pub mod lexer;
mod pos;
pub mod token;

#[cfg(test)]
mod tests;

pub use ast::*;
pub use grammar::SchemaParser as Parser;
use lalrpop_util::ParseError;
pub use lexer::Lexer;
use lexer::LexicalError;
pub use pos::Loc;
pub use token::*;

pub fn parse<'a>(input: &'a str) -> Result<(Schema, Vec<(Loc, String)>), Vec<(Loc, String)>> {
    let mut errors = vec![];
    let mut comments = vec![];
    let lexer = lexer::Lexer::new(input, &mut comments);
    let res = Parser::new().parse(input, lexer);
    match res {
        Ok(s) => Ok((s, comments)),
        Err(e) => {
            match e {
                ParseError::InvalidToken { location } => {
                    errors.push(((location, location), format!("parser-invalid-token")));
                }
                ParseError::UnrecognizedEof { location, expected } => {
                    errors.push((
                        (location, location),
                        format!("parser-unrecognized-eof: excepted: {:?}", expected),
                    ));
                }
                ParseError::UnrecognizedToken { token, expected } => {
                    errors.push((
                        (token.0, token.2),
                        format!("parser-unrecognized-token: excepted: {:?}", expected),
                    ));
                }
                ParseError::ExtraToken { token } => {
                    errors.push(((token.0, token.2), format!("parser-extra-token")));
                }
                ParseError::User { error } => match error {
                    LexicalError::UnrecognisedToken(loc, token) => {
                        errors.push((loc, format!("lex-unrecognized-token: {:?}", token)));
                    }
                },
            }
            Err(errors)
        }
    }
}
