use std::*;
use TokenKind::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,

    // Literals
    StrLiteral,
    NumLiteral,

    // Operators and Symbols
    SingleEqual,
    SemiColon,
    Dot,
    DoubleEqual,
    ExclEqual,
    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    LeftParen,
    RightParen,
    LeftCurly,
    RightCurly,
    LeftBracket,
    RightBracket,
    Exclamation
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub line: usize,
    pub column: usize
}

#[inline(always)]
fn parse_as_integer<I: Iterator<Item = char>>(mut prefix: String, chars: &mut iter::Peekable<I>, radix: u32) -> String {
    while let Some(ch) = chars.peek() {
        if !ch.is_digit(radix) {
            break;
        }
        prefix.push(chars.next().unwrap());
    }
    prefix
}

#[inline(always)]
fn read_numchars<I: Iterator<Item = char>>(chars: &mut iter::Peekable<I>, out: &mut String) {
    while let Some(c) = chars.peek() {
        if !c.is_ascii_degit() {
            break;
        }
        out.push(chars.next().unwrap());
    }
}

struct ParseState {
    tokens: Vec<Token>,
    pub line: usize,
    pub column: usize,
    pub filename: &str
}

impl ParseState {
    fn push_token(&mut self, kind: TokenKind, text: String) {
        let len = text.len();
        state.tokens.push(Token {
            kind,
            text,
            line: self.line,
            column: self.column
        });
        self.column += len;
    }
}

pub fn parse(input: &str, filename: &str) -> Vec<Token> {
    let mut state = ParseState {
        tokens: Vec::new(),
        line: 1,
        column: 1
    };
    let mut input = input.chars().peekable();
    let show_error = |error: &str, state: &ParseState| -> ! {
        eprintln!("{}\n  at {}:{}:{}", error, filename, state.line, state.column);
        process::exit(1);
    };
    while let Some(c) = input.next() {
        match c {
            '0' => state.push_token(NumLiteral, match input.peek() {
                Some('x') => {
                    input.next();
                    parse_as_integer("0x".to_string(), &mut input, 16)
                },
                Some('o') => {
                    input.next();
                    parse_as_integer("0o".to_string(), &mut input, 8)
                },
                Some('b') => {
                    input.next();
                    parse_as_integer("0b".to_string(), &mut input, 2)
                },
                Some('.') => {
                    let mut text = '0'.to_string();
                    if let Some('0'..='9') = input.clone().nth(2) {
                        input.next();
                        text.push('.');
                        read_numchars(&mut input, &mut text);
                        if input.peek() == Some(&'e') {
                            if let Some('1'..='9') = input.clone().nth(2) {
                                input.next();
                                text.push('e');
                                read_numchars(&mut input, &mut text);
                            }
                        }
                    }
                    text
                },
                _ => '0'.to_string()
            }),
            '1'..='9' => {
                let mut text = c.to_string();
                read_numchars(&mut input, &mut text);
                if input.peek() == Some(&'.') {
                    if let Some('0'..='9') = input.clone().nth(2) {
                        input.next();
                        text.push('.');
                        read_numchars(&mut input, &mut text);
                    }
                }
                if input.peek() == Some(&'e') {
                    if let Some('1'..='9') = input.clone().nth(2) {
                        input.next();
                        text.push('e');
                        read_numchars(&mut input, &mut text);
                    }
                }
                state.push_token(NumLiteral, text);
            },
            '"' => {
                let mut text = '"'.to_string();
                while let Some(c) = input.next() {
                    match c {
                        '\\' => match input.peek() {
                            Some('n' | 't' | 'r' | '\\' | '"') => {
                                text.push('\\');
                                text.push(input.next().unwrap());
                            },
                            _ => show_error("LiteralError: Invalid escape sequence", &state)
                        },
                        '\n' => show_error("LiteralError: Unterminated string literal", &state),
                        '"' => break,
                        _ => text.push(c)
                    }
                }
                text.push('"');
                state.push_token(StrLiteral, text);
            },
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut text = c.to_string();
                while let Some(c) = input.peek() {
                    match c {
                        'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => text.push(input.next().unwrap()),
                        _ => break
                    }
                }
                state.push_token(Identifier, text);
            },
            '=' => if input.peek() == Some(&'=') {
                state.push_token(DoubleEqual, "==".to_string());
                input.next();
            } else {
                state.push_token(SingleEqual, '='.to_string());
            },
            '!' => if input.peek() == Some(&'=') {
                state.push_token(ExclEqual, "!=".to_string());
                input.next();
            } else {
                state.push_token(Exclamation, '!'.to_string());
            },
            '<' => if input.peek() == Some(&'=') {
                state.push_token(LessThanEq, "<=".to_string());
                input.next();
            } else {
                state.push_token(LessThan, '<'.to_string());
            },
            '>' => if input.peek() == Some(&'=') {
                state.push_token(GreaterThanEq, ">=".to_string());
                input.next();
            } else {
                state.push_token(GreaterThan, '>'.to_string());
            },
            '\n' => {
                state.line += 1;
                state.column = 1;
            },
            '+' => state.push_token(Plus, '+'.to_string()),
            '-' => state.push_token(Minus, '-'.to_string()),
            '*' => state.push_token(Asterisk, '*'.to_string()),
            '/' => state.push_token(Slash, '/'.to_string()),
            '%' => state.push_token(Percent, '%'.to_string()),
            ';' => state.push_token(SemiColon, ';'.to_string()),
            '.' => state.push_token(Dot, '.'.to_string()),
            '(' => state.push_token(LeftParen, '('.to_string()),
            ')' => state.push_token(RightParen, ')'.to_string()),
            '{' => state.push_token(LeftCurly, '{'.to_string()),
            '}' => state.push_token(RightCurly, '}'.to_string()),
            '[' => state.push_token(LeftBracket, '['.to_string()),
            ']' => state.push_token(RightBracket, ']'.to_string()),
            ' ' | '\t' => state.column += 1,
            '\r' | '\0' => (),
            _ => show_error(&format!("CharacterError: Invalid character '{}'", c), &state)
        }
    }
    state.tokens
}
