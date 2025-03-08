use {
  super::token::Token,
  crate::sql::token::Keyword,
  anyhow::anyhow,
  std::{iter::Peekable, str::Chars},
};

pub struct Lexer<'lexer> {
  characters: Peekable<Chars<'lexer>>,
}

impl<'lexer> Lexer<'lexer> {
  pub fn new(input: &'lexer str) -> Self {
    Self {
      characters: input.chars().peekable(),
    }
  }
}

impl Iterator for Lexer<'_> {
  type Item = anyhow::Result<Token>;

  fn next(&mut self) -> Option<Self::Item> {
    match self.lex() {
      Ok(Some(token)) => Some(Ok(token)),

      //
      Ok(None) => self
        .characters
        .peek()
        .map(|character| Err(anyhow!("Unexpected character {character}"))),

      Err(error) => Some(Err(error)),
    }
  }
}

impl Lexer<'_> {
  pub fn lex(&mut self) -> anyhow::Result<Option<Token>> {
    self.ignore_whitespaces();

    match self.characters.peek() {
      Some(character) if character.is_numeric() => Ok(self.scan_number()),
      Some('\'') => self.scan_string(),

      Some('"') => self.scan_quoted_identifier(),
      Some(character) if character.is_alphabetic() => Ok(self.scan_identifier_or_keyword()),

      Some(_) => Ok(self.scan_symbol()),

      None => Ok(None),
    }
  }
}

impl Lexer<'_> {
  fn scan_number(&mut self) -> Option<Token> {
    let mut number = self
      .next_if(|character| character.is_ascii_digit())?
      .to_string();

    while let Some(digit) = self.next_if(|character| character.is_ascii_digit()) {
      number.push(digit);
    }

    // Scan the fractional part, if present.
    if let Some(dot) = self.next_if(|character| character == '.') {
      number.push(dot);
      while let Some(digit) = self.next_if(|character| character.is_ascii_digit()) {
        number.push(digit);
      }
    }

    // Scan the exponential part, if present.
    if let Some(exponential) = self.next_if(|character| (character == 'e') || (character == 'E')) {
      number.push(exponential);
      if let Some(sign) = self.next_if(|character| (character == '+') || (character == '-')) {
        number.push(sign);
      }
      while let Some(digit) = self.next_if(|character| character.is_ascii_digit()) {
        number.push(digit);
      }
    }

    Some(Token::Number(number))
  }

  fn scan_string(&mut self) -> anyhow::Result<Option<Token>> {
    if self.next_if(|character| character == '\'').is_none() {
      return Ok(None);
    }

    let mut string = String::new();
    loop {
      match self.characters.next() {
        // In SQL, inside a string, '' is an escape sequence for '.
        // So if you want to have 'It's a nice day!', you'll need to write :
        // 'It''s a nice day'.
        Some('\'') if self.next_is('\'') => string.push('\''),

        Some('\'') => break,

        Some(character) => string.push(character),

        None => return Err(anyhow!("String ended unexpectedly")),
      }
    }

    Ok(Some(Token::String(string)))
  }

  // Double quoted identifiers are also called delimited identifiers.
  // Case is preserved for a delimited identifiers.
  fn scan_quoted_identifier(&mut self) -> anyhow::Result<Option<Token>> {
    if self.next_if(|character| character == '"').is_none() {
      return Ok(None);
    }

    let mut identifier = String::new();
    loop {
      match self.characters.next() {
        // In SQL, inside an identifier, "" is an escape sequence for ".
        // So if you want to have "she said "hello"", you'll need to write :
        // "she said ""hello""".
        Some('"') if self.next_is('"') => identifier.push('"'),

        Some('"') => break,

        Some(character) => identifier.push(character),

        None => return Err(anyhow!("Identifier ended unexpectedly")),
      }
    }

    Ok(Some(Token::Identifier(identifier)))
  }

  // The non-delimited identifier / keyword gets transformed to lowercase.
  fn scan_identifier_or_keyword(&mut self) -> Option<Token> {
    // Identifiers which are not surrounded by any double quotes, are called non-delimited
    // identifiers. They must start with an alphabetic character and can contain only alphabets,
    // digits or underscores.
    let mut identifier_or_keyword = self
      .next_if(|character| character.is_alphabetic())?
      .to_lowercase()
      .to_string();
    while let Some(character) = self
      .next_if(|character| character.is_alphabetic() || character.is_numeric() || character == '_')
    {
      identifier_or_keyword.extend(character.to_lowercase());
    }

    match Keyword::try_from(identifier_or_keyword.as_str()) {
      Ok(keyword) => Some(Token::Keyword(keyword)),

      _ => {
        let identifier = identifier_or_keyword;
        Some(Token::Identifier(identifier))
      }
    }
  }

  fn scan_symbol(&mut self) -> Option<Token> {
    let mut token = self
      .characters
      .peek()
      .and_then(|character| -> Option<Token> {
        Some(match character {
          '.' => Token::Period,
          '=' => Token::Equal,
          '>' => Token::GreaterThan,
          '<' => Token::LessThan,
          '+' => Token::Plus,
          '-' => Token::Minus,
          '*' => Token::Asterisk,
          '/' => Token::Slash,
          '^' => Token::Caret,
          '%' => Token::Percent,
          '!' => Token::Exclamation,
          '?' => Token::Question,
          ',' => Token::Comma,
          ';' => Token::Semicolon,
          '(' => Token::OpenParenthesis,
          ')' => Token::CloseParenthesis,

          _ => return None,
        })
      })?;

    token = match token {
      // Handle symbols with two characters.
      Token::Exclamation if self.next_is('=') => Token::NotEqual,
      Token::LessThan if self.next_is('>') => Token::LessOrGreaterThan,
      Token::LessThan if self.next_is('=') => Token::LessThanOrEqual,
      Token::GreaterThan if self.next_is('=') => Token::GreaterThanOrEqual,

      _ => token,
    };

    Some(token)
  }

  // Used to ignore any contiguous whitespace characters, starting from the next position.
  fn ignore_whitespaces(&mut self) {
    while self
      .next_if(|character| character.is_whitespace())
      .is_some()
    {}
  }

  // Consumes and returns the character in the next position, if the given condition is met.
  fn next_if(&mut self, condition: impl Fn(char) -> bool) -> Option<char> {
    self.characters.next_if(|character| condition(*character))
  }

  // Returns whether the next character is the expected character or not.
  // And also, if so, then the next character is consumed.
  fn next_is(&mut self, expected_character: char) -> bool {
    self
      .next_if(|character| character == expected_character)
      .is_some()
  }
}
