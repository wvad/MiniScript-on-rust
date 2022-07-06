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
    while let Some(c) = chars.next_if(|c| c.is_digit(radix)) {
        prefix.push(c);
    }
    prefix
}

#[inline(always)]
fn read_numchars<I: Iterator<Item = char>>(chars: &mut iter::Peekable<I>, out: &mut String) {
    while let Some(c) = chars.next_if(|c| c.is_ascii_digit()) {
        out.push(c);
    }
}

struct ParseState {
    tokens: Vec<Token>,
    pub line: usize,
    pub column: usize
}

impl ParseState {
    #[inline(always)]
    fn push_token<S: ToString>(&mut self, kind: TokenKind, text: S) {
        let text: String = text.to_string();
        let len = text.len();
        self.tokens.push(Token {
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
            '=' => match input.next_if_eq(&'=') {
                None => state.push_token(SingleEqual, '='),
                _ => state.push_token(DoubleEqual, "==")
            },
            '!' => match input.next_if_eq(&'=') {
                None => state.push_token(ExclEqual, '!'),
                _ => state.push_token(Exclamation, "!=")
            },
            '<' => match input.next_if_eq(&'=') {
                None => state.push_token(LessThan, '<'),
                _ => state.push_token(LessThanEq, "<="),
            },
            '>' => match input.next_if_eq(&'=') {
                None => state.push_token(GreaterThan, '>'),
                _ => state.push_token(GreaterThanEq, ">=")
            },
            '\n' => {
                state.line += 1;
                state.column = 1;
            },
            '+' => state.push_token(Plus, '+'),
            '-' => state.push_token(Minus, '-'),
            '*' => state.push_token(Asterisk, '*'),
            '/' => state.push_token(Slash, '/'),
            '%' => state.push_token(Percent, '%'),
            ';' => state.push_token(SemiColon, ';'),
            '.' => state.push_token(Dot, '.'),
            '(' => state.push_token(LeftParen, '('),
            ')' => state.push_token(RightParen, ')'),
            '{' => state.push_token(LeftCurly, '{'),
            '}' => state.push_token(RightCurly, '}'),
            '[' => state.push_token(LeftBracket, '['),
            ']' => state.push_token(RightBracket, ']'),
            ' ' | '\t' => state.column += 1,
            '\r' | '\0' => (),
            _ => show_error(&format!("CharacterError: Invalid character '{}'", c), &state)
        }
    }
    state.tokens
}
