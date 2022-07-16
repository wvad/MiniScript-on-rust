use crate::lexer::{Token, TokenKind};
use std::{collections::VecDeque, fmt};
use Expression::*;

type ExprPtr = Box<Expression>;

pub enum Expression {
    StringValue(String),
    NumberValue(f64),
    Variable(String),
    MemberAccess(ExprPtr, ExprPtr),
    FunctionCall(ExprPtr, Vec<Expression>),
    LogicalNot(ExprPtr),
    UnaryNegation(ExprPtr),
    Typeof(ExprPtr),
    Multiplication(ExprPtr, ExprPtr),
    Division(ExprPtr, ExprPtr),
    Remainder(ExprPtr, ExprPtr),
    Addition(ExprPtr, ExprPtr),
    Subtraction(ExprPtr, ExprPtr),
    LessThan(ExprPtr, ExprPtr),
    LessThanEq(ExprPtr, ExprPtr),
    GreaterThan(ExprPtr, ExprPtr),
    GreaterThanEq(ExprPtr, ExprPtr),
    Equality(ExprPtr, ExprPtr),
    Inequality(ExprPtr, ExprPtr),
    LogicalAnd(ExprPtr, ExprPtr),
    LogicalOr(ExprPtr, ExprPtr),
    Assignment(ExprPtr, ExprPtr),
}

impl Expression {
    #[inline(always)]
    fn boxing(self) -> ExprPtr {
        Box::new(self)
    }
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StringValue(s) => write!(f, "{}", s),
            NumberValue(n) => write!(f, "{}", n),
            Variable(s) => write!(f, "var({})", s),
            MemberAccess(left, right) => write!(f, "access({:?}, {:?})", left, right),
            FunctionCall(left, right) => write!(f, "call({:?}, {:?})", left, right),
            LogicalNot(left) => write!(f, "not({:?})", left),
            UnaryNegation(left) => write!(f, "minus({:?})", left),
            Typeof(left) => write!(f, "type({:?})", left),
            Multiplication(left, right) => write!(f, "mul({:?}, {:?})", left, right),
            Division(left, right) => write!(f, "div({:?}, {:?})", left, right),
            Remainder(left, right) => write!(f, "rem({:?}, {:?})", left, right),
            Addition(left, right) => write!(f, "add({:?}, {:?})", left, right),
            Subtraction(left, right) => write!(f, "sub({:?}, {:?})", left, right),
            LessThan(left, right) => write!(f, "lt({:?}, {:?})", left, right),
            LessThanEq(left, right) => write!(f, "le({:?}, {:?})", left, right),
            GreaterThan(left, right) => write!(f, "gt({:?}, {:?})", left, right),
            GreaterThanEq(left, right) => write!(f, "ge({:?}, {:?})", left, right),
            Equality(left, right) => write!(f, "eq({:?}, {:?})", left, right),
            Inequality(left, right) => write!(f, "nq({:?}, {:?})", left, right),
            LogicalAnd(left, right) => write!(f, "and({:?}, {:?})", left, right),
            LogicalOr(left, right) => write!(f, "or({:?}, {:?})", left, right),
            Assignment(left, right) => write!(f, "asin({:?}, {:?})", left, right),
        }
    }
}

fn parse_value_expr(tokens: &mut VecDeque<Token>) -> Result<Expression, String> {
    if let Some(token) = tokens.pop_front() {
        match token.kind {
            TokenKind::StrLiteral(value) => Ok(StringValue(value)),
            TokenKind::NumLiteral(value) => Ok(NumberValue(value.value)),
            TokenKind::Identifier(value) => Ok(Variable(value)),
            TokenKind::LeftParen => {
                let expr = parse_expression(tokens);
                let token = tokens.pop_front().unwrap();
                if token.kind != TokenKind::RightParen {
                    Err(format!("Expected ')' but found '{:?}'", token.kind))
                } else {
                    expr
                }
            }
            _ => Err(format!("Expected primary but found '{:?}'", token.kind)),
        }
    } else {
        Err("Unexpected end of input".to_string())
    }
}

fn parse_primary(tokens: &mut VecDeque<Token>) -> Result<Expression, String> {
    let mut expr = parse_value_expr(tokens)?;
    while let Some(token) = tokens.front() {
        match token.kind {
            TokenKind::Dot => {
                tokens.pop_front();
                expr = MemberAccess(expr.boxing(), parse_value_expr(tokens)?.boxing());
            }
            TokenKind::LeftParen => {
                tokens.pop_front();
                let mut args = Vec::new();
                while let Some(token) = tokens.front() {
                    if token.kind == TokenKind::RightParen {
                        tokens.pop_front();
                        break;
                    }
                    args.push(parse_expression(tokens)?);
                    if let Some(token) = tokens.front() {
                        if token.kind == TokenKind::Comma {
                            tokens.pop_front();
                        } else if token.kind != TokenKind::RightParen {
                            return Err(format!(
                                "Expected ',' or ')' but found '{:?}'",
                                token.kind
                            ));
                        }
                    } else {
                        return Err("Unexpected end of input".to_string());
                    }
                }
                expr = FunctionCall(expr.boxing(), args);
            }
            _ => break,
        }
    }
    Ok(expr)
}

fn parse_unary(tokens: &mut VecDeque<Token>) -> Result<Expression, String> {
    if let Some(token) = tokens.front() {
        match token.kind {
            TokenKind::Exclamation => {
                tokens.pop_front();
                return Ok(LogicalNot(parse_unary(tokens)?.boxing()));
            }
            TokenKind::TypeofKeyword => {
                tokens.pop_front();
                return Ok(Typeof(parse_unary(tokens)?.boxing()));
            }
            TokenKind::Minus => {
                tokens.pop_front();
                return Ok(UnaryNegation(parse_unary(tokens)?.boxing()));
            }
            _ => (),
        }
    }
    parse_primary(tokens)
}

fn parse_muldiv(tokens: &mut VecDeque<Token>) -> Result<Expression, String> {
    let mut left = parse_unary(tokens)?;
    while let Some(token) = tokens.front() {
        match token.kind {
            TokenKind::Asterisk => {
                tokens.pop_front();
                left = Multiplication(left.boxing(), parse_unary(tokens)?.boxing());
            }
            TokenKind::Slash => {
                tokens.pop_front();
                left = Division(left.boxing(), parse_unary(tokens)?.boxing());
            }
            TokenKind::Percent => {
                tokens.pop_front();
                left = Remainder(left.boxing(), parse_unary(tokens)?.boxing());
            }
            _ => break,
        }
    }
    Ok(left)
}

fn parse_addsub(tokens: &mut VecDeque<Token>) -> Result<Expression, String> {
    let mut left = parse_muldiv(tokens)?;
    while let Some(token) = tokens.front() {
        match token.kind {
            TokenKind::Plus => {
                tokens.pop_front();
                left = Addition(left.boxing(), parse_muldiv(tokens)?.boxing());
            }
            TokenKind::Minus => {
                tokens.pop_front();
                left = Subtraction(left.boxing(), parse_muldiv(tokens)?.boxing());
            }
            _ => break,
        }
    }
    Ok(left)
}

fn parse_relational(tokens: &mut VecDeque<Token>) -> Result<Expression, String> {
    let mut left = parse_addsub(tokens)?;
    while let Some(token) = tokens.front() {
        match token.kind {
            TokenKind::LessThan => {
                tokens.pop_front();
                left = LessThan(left.boxing(), parse_addsub(tokens)?.boxing());
            }
            TokenKind::LessThanEq => {
                tokens.pop_front();
                left = LessThanEq(left.boxing(), parse_addsub(tokens)?.boxing());
            }
            TokenKind::GreaterThan => {
                tokens.pop_front();
                left = GreaterThan(left.boxing(), parse_addsub(tokens)?.boxing());
            }
            TokenKind::GreaterThanEq => {
                tokens.pop_front();
                left = GreaterThanEq(left.boxing(), parse_addsub(tokens)?.boxing());
            }
            _ => break,
        }
    }
    Ok(left)
}

fn parse_equality(tokens: &mut VecDeque<Token>) -> Result<Expression, String> {
    let mut left = parse_relational(tokens)?;
    while let Some(token) = tokens.front() {
        match token.kind {
            TokenKind::DoubleEqual => {
                tokens.pop_front();
                left = Equality(left.boxing(), parse_relational(tokens)?.boxing());
            }
            TokenKind::ExclEqual => {
                tokens.pop_front();
                left = Inequality(left.boxing(), parse_relational(tokens)?.boxing());
            }
            _ => break,
        }
    }
    Ok(left)
}

fn parse_logical_and(tokens: &mut VecDeque<Token>) -> Result<Expression, String> {
    let mut left = parse_equality(tokens)?;
    while tokens
        .front()
        .map(|token| token.kind == TokenKind::DoubleAnd)
        .unwrap_or(false)
    {
        tokens.pop_front();
        left = LogicalAnd(left.boxing(), parse_equality(tokens)?.boxing());
    }
    Ok(left)
}

fn parse_logical_or(tokens: &mut VecDeque<Token>) -> Result<Expression, String> {
    let mut left = parse_logical_and(tokens)?;
    while tokens
        .front()
        .map(|token| token.kind == TokenKind::DoublePipe)
        .unwrap_or(false)
    {
        tokens.pop_front();
        left = LogicalOr(left.boxing(), parse_logical_and(tokens)?.boxing());
    }
    Ok(left)
}

fn parse_assignment(tokens: &mut VecDeque<Token>) -> Result<Expression, String> {
    let left = parse_logical_or(tokens)?;
    if let Some(token) = tokens.front() {
        if token.kind == TokenKind::SingleEqual {
            tokens.pop_front();
            return Ok(Assignment(
                left.boxing(),
                parse_assignment(tokens)?.boxing(),
            ));
        }
    }
    Ok(left)
}

pub fn parse_expression(tokens: &mut VecDeque<Token>) -> Result<Expression, String> {
    parse_assignment(tokens)
}
