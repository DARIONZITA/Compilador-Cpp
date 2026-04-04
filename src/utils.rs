use crate::token::{Token, TokenInfo};

pub fn eh_letra(c: char) -> bool {
    c.is_ascii_alphabetic()
}

pub fn eh_digito(c: char) -> bool {
    c.is_ascii_digit()
}

pub fn eh_espaco(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r')
}

pub fn push_token(tokens: &mut Vec<TokenInfo>, kind: Token, lexema: &mut String) {
    if !lexema.is_empty() {
        tokens.push(TokenInfo {
            kind,
            lexema: lexema.clone(),
        });
        lexema.clear();
    }
}
