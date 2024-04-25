use std::{fmt::Display, iter::Peekable, str::Chars};
use crate::result::{Error, Result};
use super::token::{Keyword, Token};

pub struct Lexer<'a> {
  input: Peekable<Chars<'a>>,
}

impl<'a> Iterator for Lexer<'a> {
  type Item = Result<Token>;

  fn next(&mut self) -> Option<Self::Item> {
    match self.scan( ) {
      Err(error) => Some(Err(error)),

      Ok(Some(token)) => Some(Ok(token)),
      Ok(None) => {
        self.input.peek( )
                  .map(|character| Err(Error::Parse(format!("Unexpected character {}", character))))
      }
    }
  }
}

impl<'a> Lexer<'a> {
  pub fn new(input: &'a str) -> Self {
    return Self {
      input: input.chars( ).peekable( )
    }
  }

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

    // Handling Decimal numbers (e.g. - 3.27)
    if let Some(decimal)= self.nextIf(|character| character == '.') {
      number.push(decimal);

      self.nextWhile(|character| character.is_ascii_digit( ))
          .map(|postDecimalDigits| number.push_str(&postDecimalDigits));
    }

    // Handling Exponential notation (e.g. - 1.8e-3 which represents 1.8 * (10 ^ -3)).
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

      '(' => Some(Token::OpenParenthesis),
      ')' => Some(Token::CloseParenthesis),

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
