use std::{collections::BTreeMap, default};

pub enum Statement {
  Begin {
    readonly: bool,
    asOfVersion: Option<u64>
  },

  CreateTable {
    name: String,
    columns: Vec<Column>
  },
  DropTable(String),

  Insert {
    table: String,
    columns: Option<Vec<String>>,
    values: Vec<Vec<Expression>>
  },
  Select {
    selections: Vec<(Expression, Option<AliasColumnName>)>,
    from: Vec<SearchField>,
    r#where: Option<Expression>,
    groupBy: Vec<Expression>,
    having: Option<Expression>,
    order: Vec<(Expression, Order)>,
    limit: Option<Expression>,
    offset: Option<Expression>
  },
  Update {
    table: String,
    updates: BTreeMap<String, Expression>, // TODO: Understand why a BTree is used instead of a
                                           // Hashmap.
    r#where: Option<Expression>
  },
  Delete {
    table: String,
    r#where: Option<Expression>
  },

  Commit,
  Rollback,

  Explain(Box<Statement>)
}

#[derive(Default)]
pub struct Column {
  pub name: String,
  pub dataType: DataType,

  pub unique: bool,
  pub nullable: Option<bool>, // The value of a column can sometimes be nullable, even if the NULL
                              // keyword is not explicitly used.
  pub default: Option<Expression>,

  pub primaryKey: bool,
  pub index: bool,

  pub references: Option<String>
}

#[derive(Default)]
pub enum DataType {
  Boolean,
  Integer,
  Float,
  String,

  #[default]
  Phantom
}

pub enum Expression {
  Field(Option<String>, String),
  Literal(Literal),
  FunctionCall(String, Vec<Expression>),
  Operation(Operation),

  // Only used during the planning stage - to break off expression subtrees.
  Column,
}

impl From<Literal> for Expression {
  fn from(literal: Literal) -> Self {
    Self::Literal(literal)
  }
}

impl From<Operation> for Expression {
  fn from(operation: Operation) -> Self {
    Self::Operation(operation)
  }
}

pub enum Literal {
  Null,
  Boolean(bool),
  Integer(i64),
  Float(f64),
  String(String),
}

pub enum Operation {
  // Done by logical operators.
  And(Box<Expression>, Box<Expression>),
  Not(Box<Expression>),
  Or(Box<Expression>, Box<Expression>),

  // Done by comparison operators.
  Equal(Box<Expression>, Box<Expression>),
  GreaterThan(Box<Expression>, Box<Expression>),
  GreaterThanOrEqual(Box<Expression>, Box<Expression>),
  IsNull(Box<Expression>),
  LessThan(Box<Expression>, Box<Expression>),
  LessThanOrEqual(Box<Expression>, Box<Expression>),
  NotEqual(Box<Expression>, Box<Expression>),

  // Done by mathematical operators.
  Add(Box<Expression>, Box<Expression>),
  Assert(Box<Expression>),
  Divide(Box<Expression>, Box<Expression>),
  Exponentiate(Box<Expression>, Box<Expression>),
  Factorial(Box<Expression>),
  Modulo(Box<Expression>, Box<Expression>),
  Multiply(Box<Expression>, Box<Expression>),
  Negate(Box<Expression>),
  Subtract(Box<Expression>, Box<Expression>),

  // Done by string operators.
  Like(Box<Expression>, Box<Expression>),
}

pub enum Order {
  Ascending,
  Descending,
}

pub type AliasColumnName= String;

pub enum SearchField {
  Table {
    name: String,
    alias: Option<String>
  },
  Join {
    left: Box<SearchField>,
    right: Box<SearchField>,
    r#type: JoinType,
    predicate: Option<Expression>
  }
}

pub enum JoinType {
  Cross,
  Inner,
  Left,
  Right,
}