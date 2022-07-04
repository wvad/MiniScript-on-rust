trait Expression {
  fn typing(&self);
  fn evaluate(&self);
}

pub enum ExpressionKind {
  String,
  Number,
  Variable,
  MemberAccess,
  FunctionCall,
  LogicalNot,
  UnaryNegation,
  Typeof,
  Multiplication,
  Division,
  Remainder,
  Addition,
  Subtraction,
  LessThan,
  LessThanEq,
  GreaterThan,
  GreaterThanEq,
  Equality,
  LogicalAnd,
  LogicalOr,
  Assignment
}
