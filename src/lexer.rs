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
        let c = *c as u32;
        if c < 0x30 || 0x39 < c {
            break;
        }
        out.push(chars.next().unwrap());
    }
}

#[inline(always)]
fn get_second_item<I: Iterator + Clone>(iter: &I) -> Option<I::Item> {
    let mut cloned = iter.clone();
    match cloned.next() {
        None => None,
        _ => cloned.next()
    }
}

pub fn parse(input: &str, filename: &str) -> Vec<Token> {
    struct ParseState {
        tokens: Vec<Token>,
        line: usize,
        column: usize
    }
    let mut state = ParseState {
        tokens: Vec::new(),
        line: 1,
        column: 1
    };
    let mut input = input.chars().peekable();
    let push_token = |state: &mut ParseState, kind: TokenKind, text: String| {
        let len = text.len();
        state.tokens.push(Token {
            kind,
            text,
            line: state.line,
            column: state.column
        });
        state.column += len;
    };
    let show_error = |error: &str, state: &ParseState| -> ! {
        eprintln!("{}\n  at {}:{}:{}", error, filename, state.line, state.column);
        process::exit(1);
    };
    while let Some(c) = input.next() {
        match c {
            '0' => push_token(&mut state, NumLiteral, match input.peek() {
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
                    if let Some('0'..='9') = get_second_item(&input) {
                        input.next();
                        text.push('.');
                        read_numchars(&mut input, &mut text);
                        if input.peek() == Some(&'e') {
                            if let Some('1'..='9') = get_second_item(&input) {
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
                    if let Some('0'..='9') = get_second_item(&input) {
                        input.next();
                        text.push('.');
                        read_numchars(&mut input, &mut text);
                    }
                }
                if input.peek() == Some(&'e') {
                    if let Some('1'..='9') = get_second_item(&input) {
                        input.next();
                        text.push('e');
                        read_numchars(&mut input, &mut text);
                    }
                }
                push_token(&mut state, NumLiteral, text);
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
                push_token(&mut state, StrLiteral, text);
            },
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut text = c.to_string();
                while let Some(c) = input.peek() {
                    match c {
                        'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => text.push(input.next().unwrap()),
                        _ => break
                    }
                }
                push_token(&mut state, Identifier, text);
            },
            '=' => if input.peek() == Some(&'=') {
                push_token(&mut state, DoubleEqual, "==".to_string());
                input.next();
            } else {
                push_token(&mut state, SingleEqual, '='.to_string());
            },
            '!' => if input.peek() == Some(&'=') {
                push_token(&mut state, ExclEqual, "!=".to_string());
                input.next();
            } else {
                push_token(&mut state, Exclamation, '!'.to_string());
            },
            '<' => if input.peek() == Some(&'=') {
                push_token(&mut state, LessThanEq, "<=".to_string());
                input.next();
            } else {
                push_token(&mut state, LessThan, '<'.to_string());
            },
            '>' => if input.peek() == Some(&'=') {
                push_token(&mut state, GreaterThanEq, ">=".to_string());
                input.next();
            } else {
                push_token(&mut state, GreaterThan, '>'.to_string());
            },
            '\n' => {
                state.line += 1;
                state.column = 1;
            },
            '+' => push_token(&mut state, Plus, '+'.to_string()),
            '-' => push_token(&mut state, Minus, '-'.to_string()),
            '*' => push_token(&mut state, Asterisk, '*'.to_string()),
            '/' => push_token(&mut state, Slash, '/'.to_string()),
            '%' => push_token(&mut state, Percent, '%'.to_string()),
            ';' => push_token(&mut state, SemiColon, ';'.to_string()),
            '.' => push_token(&mut state, Dot, '.'.to_string()),
            '(' => push_token(&mut state, LeftParen, '('.to_string()),
            ')' => push_token(&mut state, RightParen, ')'.to_string()),
            '{' => push_token(&mut state, LeftCurly, '{'.to_string()),
            '}' => push_token(&mut state, RightCurly, '}'.to_string()),
            '[' => push_token(&mut state, LeftBracket, '['.to_string()),
            ']' => push_token(&mut state, RightBracket, ']'.to_string()),
            ' ' | '\t' => state.column += 1,
            '\r' | '\0' => (),
            _ => show_error(&format!("CharacterError: Invalid character '{}'", c), &state)
        }
    }
    state.tokens
}
