use {
  super::{ast::Statement, lexer::Lexer},
  std::iter::Peekable,
};

pub struct Parser<'parser> {
  lexer: Peekable<Lexer<'parser>>,
}

impl<'parser> Parser<'parser> {
  pub fn new(statement: &'parser str) -> Self {
    Self {
      lexer: Lexer::new(statement).peekable(),
    }
  }
}

impl Parser<'_> {
  pub fn parse(&mut self) -> anyhow::Result<Statement> {
    unimplemented!()
  }
}
