use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Token<'input> {
    Identifier(&'input str), // identifier
    Comment(&'input str),    // identifier
    Type,                    // type
    Relation,                // relation
    Permission,              // permission
    Cond,                    // cond
    Colon,                   // :
    LBracket,                // (
    RBracket,                // )
    Caret,                   // ^
    And,                     // &
    Or,                      // |
    Add,                     // +
    Sub,                     // -
    YulArrow,                // ->
    Sharp,                   // #
    Newline,                 // \n
    LBrace,                  // {
    RBrace,                  // }
    Star,                    // \*
    Slash,                   // /
    Percent,                 // %
    Comma,                   // ,
    Semicolon,               // ;
    Eq,                      // =
    ExclamationMark,         // \!
    Gt,                      // >
    Lt,                      // <
    Condition,               // condition
    Int,
    Uint,
    Double,
    Bool,
    Bytes,
    String,
    Duration,
    Timestamp,
    Any,
    List,
    Map,
    IPaddress,
}

impl<'input> fmt::Display for Token<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Identifier(id) => f.write_str(id),
            Token::Comment(comment) => {
                f.write_str("// ")?;
                f.write_str(comment)
            }
            Token::Type => f.write_str("type"),
            Token::Relation => f.write_str("relation"),
            Token::Permission => f.write_str("permission"),
            Token::Cond => f.write_str("cond"),
            Token::Colon => f.write_str(":"),
            Token::LBracket => f.write_str("("),
            Token::RBracket => f.write_str(")"),
            Token::Caret => f.write_str("^"),
            Token::And => f.write_str("&"),
            Token::Or => f.write_str("|"),
            Token::Add => f.write_str("+"),
            Token::Sub => f.write_str("-"),
            Token::YulArrow => f.write_str("->"),
            Token::Sharp => f.write_str("#"),
            Token::Newline => f.write_str("\\n"),
            Token::LBrace => f.write_str("{"),
            Token::RBrace => f.write_str("}"),
            Token::Star => f.write_str("*"),
            Token::Slash => f.write_str("/"),
            Token::Percent => f.write_str("%"),
            Token::Comma => f.write_str(","),
            Token::Semicolon => f.write_str(";"),
            Token::Eq => f.write_str("="),
            Token::ExclamationMark => f.write_str("!"),
            Token::Gt => f.write_str(">"),
            Token::Lt => f.write_str("<"),
            Token::Condition => f.write_str("condition"),
            Token::Int => f.write_str("Int"),
            Token::Uint => f.write_str("Uint"),
            Token::Double => f.write_str("Double"),
            Token::Bool => f.write_str("Bool"),
            Token::Bytes => f.write_str("Bytes"),
            Token::String => f.write_str("String"),
            Token::Duration => f.write_str("Duration"),
            Token::Timestamp => f.write_str("Timestamp"),
            Token::Any => f.write_str("Any"),
            Token::List => f.write_str("List"),
            Token::Map => f.write_str("Map"),
            Token::IPaddress => f.write_str("IPaddress"),
        }
    }
}
