use std::*;
use TokenKind::*;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub struct NumLiteralData {
    pub value: f64,
    str_len: usize
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(String),

    // Literals
    StrLiteral(String),
    NumLiteral(NumLiteralData),

    // keywords
    TypeofKeyword,

    // Operators and Symbols
    SingleEqual,
    SemiColon,
    Dot,
    Comma,
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
    Exclamation,
    DoubleAnd,
    DoublePipe
}

impl TokenKind {
    #[inline(always)]
    fn new_num_literal(value: f64, str_len: usize) -> TokenKind {
        NumLiteral(NumLiteralData { value, str_len })
    }
    fn try_into_float(value: &String) -> Result<TokenKind, num::ParseFloatError> {
        <f64 as str::FromStr>::from_str(value)
        .map(|n| Self::new_num_literal(n, value.len()))
    }
    pub fn get_str_len(&self) -> usize {
        match self {
            StrLiteral(s) => s.len(),
            Identifier(id) => id.len(),
            NumLiteral(num) => num.str_len,
            SingleEqual | SemiColon | LessThan | GreaterThan | Plus | Minus |
            Asterisk | Slash | Percent | LeftParen | RightParen | LeftCurly |
            Dot | RightCurly | LeftBracket | RightBracket | Exclamation | Comma => 1,
            DoubleEqual | ExclEqual | LessThanEq | GreaterThanEq | DoubleAnd | DoublePipe => 2,
            TypeofKeyword => 6
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize
}

#[inline(always)]
fn read_numchars<I: Iterator<Item = char>>(chars: &mut iter::Peekable<I>, out: &mut String) {
    while let Some(c) = chars.next_if(|c| c.is_ascii_digit()) {
        out.push(c);
    }
}

#[inline(always)]
fn parse_int_with_prefix<I>(chars: &mut iter::Peekable<I>, mut len_init: usize, radix: u32) -> TokenKind
    where I: Iterator<Item = char> + Clone {
    if let Some('0'..='9') = chars.clone().nth(1) {} else {
        return TokenKind::new_num_literal(0., 1)
    }
    chars.next();
    let mut val: f64 = 0.0;
    while let Some(c) = chars.next_if(|c| c.is_digit(radix)) {
        val *= radix as f64;
        val += u8::from_str_radix(&c.to_string(), radix).unwrap() as f64;
        len_init += 1;
    }
    TokenKind::new_num_literal(val, len_init)
}

#[inline(always)]
fn parse_number_starting_with_0<I>(chars: &mut iter::Peekable<I>) -> Result<TokenKind, num::ParseFloatError>
    where I: Iterator<Item = char> + Clone {
    Ok(match chars.peek() {
        Some('x') => parse_int_with_prefix(chars, 2, 16),
        Some('o') => parse_int_with_prefix(chars, 2, 8),
        Some('b') => parse_int_with_prefix(chars, 2, 2),
        Some('.') => {
            let mut text = '0'.to_string();
            if let Some('0'..='9') = chars.clone().nth(1) {
                chars.next();
                text.push('.');
                read_numchars(chars, &mut text);
                if chars.peek() == Some(&'e') {
                    if let Some('1'..='9') = chars.clone().nth(1) {
                        chars.next();
                        text.push('e');
                        read_numchars(chars, &mut text);
                    }
                }
            }
            TokenKind::try_into_float(&text)?
        },
        _ => TokenKind::new_num_literal(0., 1)
    })
}

#[derive(Debug)]
pub struct ParseState {
    tokens: VecDeque<Token>,
    pub line: usize,
    pub column: usize
}

impl ParseState {
    fn new() -> Self {
        Self {
            tokens: VecDeque::new(),
            line: 1,
            column: 1
        }
    }
    fn push_token(&mut self, kind: TokenKind) {
        let len = kind.get_str_len();
        self.tokens.push_back(Token {
            kind,
            line: self.line,
            column: self.column
        });
        self.column += len;
    }
}

#[derive(Debug)]
pub enum LexerErrorKind {
    InvalidFloatLiteral,
    InvalidStringEscapeSequence,
    UnterminatedStringLiteral,
    InvalidCharacter(char)
}

#[derive(Debug)]
pub struct LexerError {
    pub state: ParseState,
    pub kind: LexerErrorKind
}

impl LexerError {    
    fn new(state: ParseState, kind: LexerErrorKind) -> Self {
        Self { state, kind }
    }
}

pub fn parse(input: &str) -> Result<VecDeque<Token>, LexerError> {
    let mut state = ParseState::new();
    let mut input = input.chars().peekable();
    while let Some(c) = input.next() {
        match c {
            '0' => state.push_token(
                match parse_number_starting_with_0(&mut input) {
                    Ok(t) => t,
                    _ => return Err(LexerError::new(state, LexerErrorKind::InvalidFloatLiteral))
                }
            ),
            '1'..='9' => {
                let mut text = c.to_string();
                read_numchars(&mut input, &mut text);
                if input.peek() == Some(&'.') {
                    if let Some('0'..='9') = input.clone().nth(1) {
                        input.next();
                        text.push('.');
                        read_numchars(&mut input, &mut text);
                    }
                }
                if input.peek() == Some(&'e') {
                    if let Some('1'..='9') = input.clone().nth(1) {
                        input.next();
                        text.push('e');
                        read_numchars(&mut input, &mut text);
                    }
                }
                state.push_token(TokenKind::try_into_float(&text).expect("Invalid float literal"));
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
                            _ => return Err(LexerError::new(state, LexerErrorKind::InvalidStringEscapeSequence))
                        },
                        '\n' => return Err(LexerError::new(state, LexerErrorKind::UnterminatedStringLiteral)),
                        '"' => break,
                        _ => text.push(c)
                    }
                }
                text.push('"');
                state.push_token(StrLiteral(text));
            },
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut text = c.to_string();
                while let Some(c) = input.next_if(|c| match c {
                    'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => true,
                    _ => false
                }) {
                    text.push(c);
                }
                state.push_token(match text.as_str() {
                    "typeof" => TypeofKeyword,
                    _ => Identifier(text)
                });
            },
            '=' => state.push_token(match input.next_if_eq(&'=') {
                None => SingleEqual,
                _ => DoubleEqual
            }),
            '!' => state.push_token(match input.next_if_eq(&'=') {
                None => Exclamation,
                _ => ExclEqual
            }),
            '<' => state.push_token(match input.next_if_eq(&'=') {
                None => LessThan,
                _ => LessThanEq,
            }),
            '>' => state.push_token(match input.next_if_eq(&'=') {
                None => GreaterThan,
                _ => GreaterThanEq
            }),
            '\n' => {
                state.line += 1;
                state.column = 1;
            },
            '+' => state.push_token(Plus),
            '-' => state.push_token(Minus),
            '*' => state.push_token(Asterisk),
            '/' => state.push_token(Slash),
            '%' => state.push_token(Percent),
            ';' => state.push_token(SemiColon),
            '.' => state.push_token(Dot),
            ',' => state.push_token(Comma),
            '(' => state.push_token(LeftParen),
            ')' => state.push_token(RightParen),
            '{' => state.push_token(LeftCurly),
            '}' => state.push_token(RightCurly),
            '[' => state.push_token(LeftBracket),
            ']' => state.push_token(RightBracket),
            ' ' | '\t' => state.column += 1,
            '\r' => (),
            '&' => if let Some(_) = input.next_if_eq(&'&') {
                    input.next();
                    state.push_token(DoublePipe);
                } else {
                    return Err(LexerError::new(state, LexerErrorKind::InvalidCharacter(c)))
                },
            '|' => if let Some(_) = input.next_if_eq(&'|') {
                    input.next();
                    state.push_token(DoubleAnd);
                } else {
                    return Err(LexerError::new(state, LexerErrorKind::InvalidCharacter(c)))
                },
            _ => return Err(LexerError::new(state, LexerErrorKind::InvalidCharacter(c)))
        }
    }
    Ok(state.tokens)
}
