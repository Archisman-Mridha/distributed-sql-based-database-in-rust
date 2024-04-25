use crate::result::Result;
use super::{ast::{Expression, Operation}, token::{Keyword, Token}, Parser};

pub type Precedance= u8;

// Represents whether the operator for the given operand is on the left / right.
pub enum Associativity {
  Left = 1,
  Right = 0
}

pub trait Operator: Sized {
  // Converts the given token to an operator.
  fn fromToken(token: &Token) -> Option<Self>;

  // Augments the operator by parsing any modifiers if present.
  fn augment(self, parser: &mut Parser) -> Result<Self>;

  // Returns the operator associativity (whether the operator for the given operand is on the left /
  // right).
  fn associativity(&self) -> Associativity;

  // Returns the operator's precedance.
  fn precedance(&self) -> Precedance;
}

pub enum PrefixOperator {
  Minus,
  Not,
  Plus
}

impl Operator for PrefixOperator {
  fn fromToken(token: &Token) -> Option<Self> {
    Some(match token {
      Token::Plus => Self::Plus,
      Token::Minus => Self::Minus,
      Token::Keyword(Keyword::NOT) => Self::Not,

      _ => return None
    })
  }

  fn augment(self, _parser: &mut Parser) -> Result<Self> {
    Ok(self)
  }

  fn associativity(&self) -> Associativity {
    Associativity::Right
  }

  fn precedance(&self) -> Precedance {
    9
  }
}

impl PrefixOperator {
  pub fn operate(&self, rhs: Expression) -> Expression {
    match self {
      Self::Plus => Operation::Assert(Box::new(rhs)).into( ),
      Self::Minus => Operation::Negate(Box::new(rhs)).into( ),

      Self::Not => Operation::Not(Box::new(rhs)).into( )
    }
  }
}

pub enum InfixOperator {
  Add,
  Divide,
  Modulo,
  Multiply,
  Subtract,
  Exponentiate,

  Equal,
  NotEqual,
  GreaterThan,
  GreaterThanOrEqual,
  LessThan,
  LessThanOrEqual,

  And,
  Or,
  Like
}

impl Operator for InfixOperator {
  fn fromToken(token: &Token) -> Option<Self> {
    Some(match token {
      Token::Asterisk => Self::Multiply,
      Token::Caret => Self::Exponentiate,
      Token::Minus => Self::Subtract,
      Token::Percent => Self::Modulo,
      Token::Plus => Self::Add,
      Token::Slash => Self::Divide,

      Token::Equal => Self::Equal,
      Token::NotEqual => Self::NotEqual,
      Token::GreaterThan => Self::GreaterThan,
      Token::GreaterThanOrEqual => Self::GreaterThanOrEqual,
      Token::LessThan => Self::LessThan,
      Token::LessThanOrEqual => Self::LessThanOrEqual,
      Token::LessOrGreaterThan => Self::NotEqual,

      Token::Keyword(Keyword::AND) => Self::And,
      Token::Keyword(Keyword::OR) => Self::Or,
      Token::Keyword(Keyword::LIKE) => Self::Like,

      _ => return None
    })
  }

  fn augment(self, _parser: &mut Parser) -> Result<Self> {
    Ok(self)
  }

  fn associativity(&self) -> Associativity {
    match self {
      Self::Exponentiate => Associativity::Left,
      _ => Associativity::Right
    }
  }

  fn precedance(&self) -> Precedance {
    match self {
      Self::Or => 1,
      Self::And => 2,

      Self::Equal | Self::NotEqual | Self::Like => 3,

      Self::GreaterThan
      | Self::GreaterThanOrEqual
      | Self::LessThan
      | Self::LessThanOrEqual => 4,

      Self::Add | Self::Subtract => 5,
      Self::Multiply | Self::Divide | Self::Modulo => 6,
      Self::Exponentiate => 7
    }
  }
}

impl InfixOperator {
  pub fn operate(&self, lhs: Expression, rhs: Expression) -> Expression {
    let lhs= Box::new(lhs);
    let rhs= Box::new(rhs);

    match self {
      Self::Add => Operation::Add(lhs, rhs),
      Self::Subtract => Operation::Subtract(lhs, rhs),
      Self::Multiply => Operation::Multiply(lhs, rhs),
      Self::Divide => Operation::Divide(lhs, rhs),
      Self::Modulo => Operation::Modulo(lhs, rhs),
      Self::Exponentiate => Operation::Exponentiate(lhs, rhs),

      Self::Equal => Operation::Equal(lhs, rhs),
      Self::NotEqual => Operation::NotEqual(lhs, rhs),
      Self::GreaterThan => Operation::GreaterThan(lhs, rhs),
      Self::GreaterThanOrEqual => Operation::GreaterThanOrEqual(lhs, rhs),
      Self::LessThan => Operation::LessThan(lhs, rhs),
      Self::LessThanOrEqual => Operation::LessThanOrEqual(lhs, rhs),

      Self::And => Operation::And(lhs, rhs),
      Self::Or => Operation::Or(lhs, rhs),
      Self::Like => Operation::Like(lhs, rhs)
    }.into( )
  }
}

pub enum PostfixOperator {
  Factorial,
  IsNull {
    not: bool,
  }
}

impl Operator for PostfixOperator {
  fn fromToken(token: &Token) -> Option<Self> {
    Some(match token {
      Token::Exclamation => Self::Factorial,
      Token::Keyword(Keyword::IS) => Self::IsNull { not: false },

      _ => return None
    })
  }

  fn augment(mut self, parser: &mut Parser) -> Result<Self> {
    #[allow(clippy::single_match)]
    match &mut self {
      Self::IsNull { ref mut not } => {
        if parser.nextTokenIfIts(Keyword::NOT.into( )).is_some( ) {
          *not= true;
        }
        parser.nextExpectedToken(Some(Token::Keyword(Keyword::NULL)))?;
      },

      _ => { }
    };
    Ok(self)
  }

  fn associativity(&self) -> Associativity {
    Associativity::Left
  }

  fn precedance(&self) -> Precedance {
    8
  }
}

impl PostfixOperator {
  pub fn operate(&self, lhs: Expression) -> Expression {
    let lhs= Box::new(lhs);
    match self {
      Self::IsNull { not } =>
        if *not {
          Operation::Not(Box::new(Operation::IsNull(lhs).into( )))}
        else {
          Operation::IsNull(lhs)},

      Self::Factorial => Operation::Factorial(lhs)
    }.into( )
  }
}