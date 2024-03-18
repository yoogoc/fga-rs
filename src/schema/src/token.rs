use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Token<'input> {
    Identifier(&'input str), // identifier
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
}

impl<'input> fmt::Display for Token<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Identifier(id) => f.write_str(id),
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
        }
    }
}
