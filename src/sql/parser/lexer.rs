use std::{fmt::Display, iter::Peekable, str::Chars};
use crate::result::{Error, Result};

pub struct Lexer<'a> {
  input: Peekable<Chars<'a>>,
}

impl<'a> Iterator for Lexer<'a> {
  type Item = Result<Token>;

  fn next(&mut self) -> Option<Self::Item> {
    unimplemented!( )
  }
}

impl<'a> Lexer<'a> {
  // Scans the input for the next token (Ignores leading whitespaces).
  fn scan(&mut self) -> Result<Option<Token>> {
    self.ignoreLeadingWhitespaces( );

    match self.input.peek( ) {
      None => Ok(None),

      Some(character) if character.is_ascii_digit( ) => Ok(self.scanNumber( )),
      Some(character) if character.is_alphabetic( ) => Ok(self.scanIdentifier( )),

      // NOTE : Single quotes delimit a string constant (literal) / a date-time constant. And double quotes
      // delimit identifiers (e.g. table / column / index names).
      Some('\'') => self.scanStringLiteral( ),
      Some('"') => self.scanQuotedIdentifier( ),

      Some(_) => Ok(self.scanSymbol( ))
    }
  }

  fn ignoreLeadingWhitespaces(&mut self) {
    self.nextWhile(|character| character.is_whitespace( ));
  }

  fn scanNumber(&mut self) -> Option<Token> {
    let mut number: String= self.nextWhile(|character| character.is_ascii_digit( ))?;

    // CASE: Decimal numbers (e.g. - 3.27)
    if let Some(decimal)= self.nextIf(|character| character == '.') {
      number.push(decimal);

      self.nextWhile(|character| character.is_ascii_digit( ))
        .map(|postDecimalDigits| number.push_str(&postDecimalDigits));
    }

    // CASE: Exponential notation (e.g. - 1.8e-3 which represents 1.8 * (10 ^ -3)).
    if let Some(e)= self.nextIf(|character| character == 'e' || character == 'E') {
      number.push(e);

      if let Some('+') | Some('-')= self.input.peek( ) {
        number.push(self.input.next( ).unwrap( ));}

      self.nextWhile(|character| character.is_ascii_digit( ))
        .map(|postSignDigits| number.push_str(&postSignDigits));
    }

    Some(Token::Number(number))
  }

  fn scanIdentifier(&mut self) -> Option<Token> {
    let mut identifierName= self.nextIf(|character| character.is_alphabetic( ))?.to_string( );

    self.nextWhile(|character| character.is_alphabetic( ) || character == '_')
      .map(|remainingCharacters| identifierName.push_str(&remainingCharacters));

    Keyword::from_str(&identifierName)
      .map(|keyword| Token::Keyword(keyword))
      .or_else(| | Some(Token::Identifier(identifierName.to_lowercase( ))))
  }

  fn scanQuotedIdentifier(&mut self) -> Result<Option<Token>> {
    if self.nextIf(|character| character == '"').is_none( ) {
      return Ok(None)}

    let mut identifierName= String::new( );

    loop {
      match self.input.next( ) {
        Some(character) => identifierName.push(character),
        Some('"') => break,
        None => return Err(Error::Parse("Unexpected end of quoted identifier".to_string( ))),
      }
    }

    Ok(Some(Token::Identifier(identifierName)))
  }

  fn scanStringLiteral(&mut self) -> Result<Option<Token>> {
    if self.nextIf(|character| character == '\'').is_none( ) {
      return Ok(None)}

    let mut value= String::new( );

    loop {
      match self.input.next( ) {
        Some(character) => value.push(character),
        Some('\'') => break,
        None => return Err(Error::Parse("Unexpected end of string literal".to_string( ))),
      }
    }

    Ok(Some(Token::Identifier(value)))
  }

  fn scanSymbol(&mut self) -> Option<Token> {
    self.nextIfToken(|character| match character {
      '.' => Some(Token::Period),

      '=' => Some(Token::Equal),
      '>' => Some(Token::GreaterThan),
      '<' => Some(Token::LessThan),

      '+' => Some(Token::Plus),
      '-' => Some(Token::Minus),
      '*' => Some(Token::Asterisk),
      '/' => Some(Token::Slash),
      '^' => Some(Token::Caret),
      '%' => Some(Token::Percent),

      '!' => Some(Token::Exclamation),
      '?' => Some(Token::Question),
      ',' => Some(Token::Comma),
      ';' => Some(Token::Semicolon),

      '(' => Some(Token::OpenParen),
      ')' => Some(Token::CloseParen),

      _ => None,
    })
      .map(|token| match token {
        Token::Exclamation => {
          if self.nextIf(|character| character == '=').is_some( ) { Token::NotEqual }
          else { token }
        },

        Token::LessThan => {
          if self.nextIf(|character| character == '=').is_some( ) { Token::LessThanOrEqual }
          else if self.nextIf(|character| character == '>').is_some( ) { Token::LessOrGreaterThan }
          else { token }
        },

        Token::GreaterThan => {
          if self.nextIf(|character| character == '=').is_some( ) { Token::GreaterThanOrEqual }
          else { token }
        },

        _ => token
      })
  }
}

impl<'a> Lexer<'a> {
  // Grabs the next character if it matches the predicate.
  fn nextIf<P>(&mut self, predicate: P) -> Option<char>
    where P: Fn(char) -> bool
  {
    self.input.peek( )
              .filter(|&character| predicate(*character))?;
    self.input.next( )
  }

  // Grabs the next single-character token if the predicate function returns one.
  fn nextIfToken<P>(&mut self, parseCharacterToToken: P) -> Option<Token>
    where P: Fn(char) -> Option<Token>
  {
    let token = self.input.peek( ).and_then(|&character| parseCharacterToToken(character))?;
    self.input.next( );
    Some(token)
  }

  // Grabs the next contiguous characters that match the predicate.
  fn nextWhile<P>(&mut self, predicate: P) -> Option<String>
    where P: Fn(char) -> bool
  {
    let mut string= String::new( );

    while let Some(character)= self.nextIf(&predicate) {
      string.push(character);}

    Some(string).filter(|value| !value.is_empty( ))
  }
}

pub enum Token {
  Number(String),
  String(String),

  Identifier(String), // Represents name of a database object (like table, column, index etc.)
  Period,

  Keyword(Keyword),

  Equal,
  GreaterThan,
  GreaterThanOrEqual,
  LessThan,
  LessThanOrEqual,
  LessOrGreaterThan,
  NotEqual,

  Plus,
  Minus,
  Asterisk,
  Slash,
  Caret,
  Percent,

  Exclamation,
  Comma,
  Semicolon,
  Question,

  OpenParen,
  CloseParen,
}

impl Display for Token {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      Token::Number(n) => n,
      Token::String(s) => s,

      Token::Identifier(s) => s,
      Token::Period => ".",

      Token::Keyword(k) => k.to_str( ),

      Token::Equal => "=",
      Token::GreaterThan => ">",
      Token::GreaterThanOrEqual => ">=",
      Token::LessThan => "<",
      Token::LessThanOrEqual => "<=",
      Token::LessOrGreaterThan => "<>",
      Token::NotEqual => "!=",

      Token::Plus => "+",
      Token::Minus => "-",
      Token::Asterisk => "*",
      Token::Slash => "/",
      Token::Caret => "^",
      Token::Percent => "%",

      Token::Exclamation => "!",
      Token::Comma => ",",
      Token::Semicolon => ";",
      Token::Question => "?",

      Token::OpenParen => "(",
      Token::CloseParen => ")",
    })
  }
}

impl From<Keyword> for Token {
  fn from(keyword: Keyword) -> Self {
    Token::Identifier(keyword.to_str( ).to_string( ))
  }
}

pub enum Keyword {
  And,
  As,
  Asc,
  Begin,
  Bool,
  Boolean,
  By,
  Char,
  Commit,
  Create,
  Cross,
  Default,
  Delete,
  Desc,
  Double,
  Drop,
  Explain,
  False,
  Float,
  From,
  Group,
  Having,
  Index,
  Infinity,
  Inner,
  Insert,
  Int,
  Integer,
  Into,
  Is,
  Join,
  Key,
  Left,
  Like,
  Limit,
  NaN,
  Not,
  Null,
  Of,
  Offset,
  On,
  Only,
  Or,
  Order,
  Outer,
  Primary,
  Read,
  References,
  Right,
  Rollback,
  Select,
  Set,
  String,
  System,
  Table,
  Text,
  Time,
  Transaction,
  True,
  Unique,
  Update,
  Values,
  Varchar,
  Where,
  Write,
}

impl Keyword {
  pub fn from_str(identifier: &str) -> Option<Self> {
    Some(match identifier.to_uppercase( ).as_ref( ) {
      "AS" => Self::As,
      "ASC" => Self::Asc,
      "AND" => Self::And,
      "BEGIN" => Self::Begin,
      "BOOL" => Self::Bool,
      "BOOLEAN" => Self::Boolean,
      "BY" => Self::By,
      "CHAR" => Self::Char,
      "COMMIT" => Self::Commit,
      "CREATE" => Self::Create,
      "CROSS" => Self::Cross,
      "DEFAULT" => Self::Default,
      "DELETE" => Self::Delete,
      "DESC" => Self::Desc,
      "DOUBLE" => Self::Double,
      "DROP" => Self::Drop,
      "EXPLAIN" => Self::Explain,
      "FALSE" => Self::False,
      "FLOAT" => Self::Float,
      "FROM" => Self::From,
      "GROUP" => Self::Group,
      "HAVING" => Self::Having,
      "INDEX" => Self::Index,
      "INFINITY" => Self::Infinity,
      "INNER" => Self::Inner,
      "INSERT" => Self::Insert,
      "INT" => Self::Int,
      "INTEGER" => Self::Integer,
      "INTO" => Self::Into,
      "IS" => Self::Is,
      "JOIN" => Self::Join,
      "KEY" => Self::Key,
      "LEFT" => Self::Left,
      "LIKE" => Self::Like,
      "LIMIT" => Self::Limit,
      "NAN" => Self::NaN,
      "NOT" => Self::Not,
      "NULL" => Self::Null,
      "OF" => Self::Of,
      "OFFSET" => Self::Offset,
      "ON" => Self::On,
      "ONLY" => Self::Only,
      "OR" => Self::Or,
      "ORDER" => Self::Order,
      "OUTER" => Self::Outer,
      "PRIMARY" => Self::Primary,
      "READ" => Self::Read,
      "REFERENCES" => Self::References,
      "RIGHT" => Self::Right,
      "ROLLBACK" => Self::Rollback,
      "SELECT" => Self::Select,
      "SET" => Self::Set,
      "STRING" => Self::String,
      "SYSTEM" => Self::System,
      "TABLE" => Self::Table,
      "TEXT" => Self::Text,
      "TIME" => Self::Time,
      "TRANSACTION" => Self::Transaction,
      "TRUE" => Self::True,
      "UNIQUE" => Self::Unique,
      "UPDATE" => Self::Update,
      "VALUES" => Self::Values,
      "VARCHAR" => Self::Varchar,
      "WHERE" => Self::Where,
      "WRITE" => Self::Write,
      _ => return None,
    })
  }

  pub fn to_str(&self) -> &str {
    match self {
      Self::As => "AS",
      Self::Asc => "ASC",
      Self::And => "AND",
      Self::Begin => "BEGIN",
      Self::Bool => "BOOL",
      Self::Boolean => "BOOLEAN",
      Self::By => "BY",
      Self::Char => "CHAR",
      Self::Commit => "COMMIT",
      Self::Create => "CREATE",
      Self::Cross => "CROSS",
      Self::Default => "DEFAULT",
      Self::Delete => "DELETE",
      Self::Desc => "DESC",
      Self::Double => "DOUBLE",
      Self::Drop => "DROP",
      Self::Explain => "EXPLAIN",
      Self::False => "FALSE",
      Self::Float => "FLOAT",
      Self::From => "FROM",
      Self::Group => "GROUP",
      Self::Having => "HAVING",
      Self::Index => "INDEX",
      Self::Infinity => "INFINITY",
      Self::Inner => "INNER",
      Self::Insert => "INSERT",
      Self::Int => "INT",
      Self::Integer => "INTEGER",
      Self::Into => "INTO",
      Self::Is => "IS",
      Self::Join => "JOIN",
      Self::Key => "KEY",
      Self::Left => "LEFT",
      Self::Like => "LIKE",
      Self::Limit => "LIMIT",
      Self::NaN => "NAN",
      Self::Not => "NOT",
      Self::Null => "NULL",
      Self::Of => "OF",
      Self::Offset => "OFFSET",
      Self::On => "ON",
      Self::Only => "ONLY",
      Self::Outer => "OUTER",
      Self::Or => "OR",
      Self::Order => "ORDER",
      Self::Primary => "PRIMARY",
      Self::Read => "READ",
      Self::References => "REFERENCES",
      Self::Right => "RIGHT",
      Self::Rollback => "ROLLBACK",
      Self::Select => "SELECT",
      Self::Set => "SET",
      Self::String => "STRING",
      Self::System => "SYSTEM",
      Self::Table => "TABLE",
      Self::Text => "TEXT",
      Self::Time => "TIME",
      Self::Transaction => "TRANSACTION",
      Self::True => "TRUE",
      Self::Unique => "UNIQUE",
      Self::Update => "UPDATE",
      Self::Values => "VALUES",
      Self::Varchar => "VARCHAR",
      Self::Where => "WHERE",
      Self::Write => "WRITE",
    }
  }
}

impl Display for Keyword {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.to_str( ))
  }
}
