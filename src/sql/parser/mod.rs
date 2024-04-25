use std::{collections::BTreeMap, iter::Peekable};
use crate::{result::{Error, Result}, sql::parser::{ast::DataType, operators::PrefixOperator}};
use self::{
  ast::{AliasColumnName, Column, Expression, JoinType, Literal, Order, SearchField, Statement},
  lexer::Lexer,
  operators::{InfixOperator, Operator, PostfixOperator, Precedance}, token::{Keyword, Token}
};

mod token;
mod lexer;
mod ast;
mod operators;

pub struct Parser<'a> {
  lexer: Peekable<Lexer<'a>>
}

impl<'a> Parser<'a> {
  // Parses the input into an Abstract Syntax Tree (AST).
  pub fn parse(&mut self) -> Result<Statement> {
    let statement= self.parseStatement( )?;
    self.nextTokenIfIts(Token::Semicolon);

    self.nextExpectedToken(None)?;

    Ok(statement)
  }

  fn parseStatement(&mut self) -> Result<Statement> {
    match self.peekNextToken( )? {
      Some(Token::Keyword(Keyword::CREATE | Keyword::DROP)) => self.parseCreateOrDropStatement( ),

      Some(Token::Keyword(Keyword::BEGIN | Keyword::COMMIT | Keyword::ROLLBACK)) =>
        self.parseTransactionStatement( ),

      Some(Token::Keyword(Keyword::INSERT)) => self.parseInsertStatement( ),
      Some(Token::Keyword(Keyword::SELECT)) => self.parseSelectStatement( ),
      Some(Token::Keyword(Keyword::UPDATE)) => self.parseUpdateStatement( ),
      Some(Token::Keyword(Keyword::DELETE)) => self.parseDeleteStatement( ),

      Some(Token::Keyword(Keyword::EXPLAIN)) => self.parseExplainStatement( ),

      Some(token) => Err(Error::Parse(format!("Unexpected token {}", token))),
      None =>  Err(Error::Parse("Unexpected end of input".into( ))),
    }
  }

  fn parseCreateOrDropStatement(&mut self) -> Result<Statement> {
    match self.nextToken( )? {
      Token::Keyword(Keyword::CREATE) => match self.nextToken( )? {
        Token::Keyword(Keyword::TABLE) => self.parseCreateTableStatement( ),
        token => Err(Error::Parse(format!("Expected TABLE keyword, got {}", token)))
      },

      Token::Keyword(Keyword::DROP) => match self.nextToken( )? {
        Token::Keyword(Keyword::TABLE) => self.parseDropTableStatement( ),
        token => Err(Error::Parse(format!("Expected TABLE keyword, got {}", token)))
      },

      token => Err(Error::Parse(format!("Expected CREATE / DROP keyword, got {}", token)))
    }
  }

  fn parseCreateTableStatement(&mut self) -> Result<Statement> {
    let tableName= self.nextIdentifier( )?;

    self.nextExpectedToken(Some(Token::OpenParenthesis))?;

    let mut columns= vec![ ];
    loop {
      columns.push(self.parseColumnSpec( )?);
      if self.nextTokenIfIts(Token::Comma).is_none( ) {
        break
      }
    }

    self.nextExpectedToken(Some(Token::CloseParenthesis))?;

    Ok(Statement::CreateTable { name: tableName, columns })
  }

  fn parseDropTableStatement(&mut self) -> Result<Statement> {
    let tableName= self.nextIdentifier( )?;
    Ok(Statement::DropTable(tableName))
  }

  fn parseColumnSpec(&mut self) -> Result<Column> {
    let mut column= Column {
      name: self.nextIdentifier( )?,

      dataType: match self.nextToken( )? {
        Token::Keyword(Keyword::BOOL) => DataType::Boolean,
        Token::Keyword(Keyword::BOOLEAN) => DataType::Boolean,

        Token::Keyword(Keyword::DOUBLE) => DataType::Float,
        Token::Keyword(Keyword::FLOAT) => DataType::Float,
        Token::Keyword(Keyword::INT) => DataType::Integer,
        Token::Keyword(Keyword::INTEGER) => DataType::Integer,

        Token::Keyword(Keyword::CHAR) => DataType::String,
        Token::Keyword(Keyword::STRING) => DataType::String,
        Token::Keyword(Keyword::TEXT) => DataType::String,
        Token::Keyword(Keyword::VARCHAR) => DataType::String,

        token => return Err(Error::Parse(format!("Unexpected token {}", token)))
      },

      ..Default::default( )
    };

    while let Some(Token::Keyword(keyword))= self.nextTokenIfItsKeyword( ) {
      match keyword {
        Keyword::PRIMARY => {
          self.nextExpectedToken(Some(Keyword::KEY.into( )))?;
          column.primaryKey= true;
        },

        Keyword::INDEX => column.index= true,

        Keyword::UNIQUE => column.unique= true,

        Keyword::NULL => {
          if let Some(false)= column.nullable {
            return Err(Error::Value(
              format!("Column {} can't be both not-nullable and nullable", column.name))
            )
          }
          column.nullable= Some(true)
        },

        Keyword::NOT => {
          self.nextExpectedToken(Some(Keyword::NULL.into( )))?;
          if let Some(true)= column.nullable {
            return Err(Error::Value(
              format!("Column {} can't be both nullable and not-nullable", column.name)))
          }
          column.nullable= Some(false)
        },

        Keyword::DEFAULT => column.default= Some(self.parseExpression(0)?),

        Keyword::REFERENCES => column.references= Some(self.nextIdentifier( )?),

        keyword => return Err(Error::Parse(format!("Unexpected keyword {}", keyword))),
      }
    }

    Ok(column)
  }

  fn parseTransactionStatement(&mut self) -> Result<Statement> {
    match self.nextToken( )? {
      Token::Keyword(Keyword::BEGIN) => {
        self.nextTokenIfIts(Keyword::TRANSACTION.into( ));

        let mut readonly= false;
        if self.nextTokenIfIts(Keyword::READ.into( )).is_some( ) {
          match self.nextToken( )? {
            Token::Keyword(Keyword::ONLY) => readonly= true,
            Token::Keyword(Keyword::WRITE) => { },

            token => return Err(Error::Parse(format!("Unexpected token {}", token)))
          }
        }

        let mut asOfVersion= None;
        if self.nextTokenIfIts(Keyword::AS.into( )).is_some( ) {
          self.nextExpectedToken(Some(Keyword::OF.into( )))?;
          self.nextExpectedToken(Some(Keyword::SYSTEM.into( )))?;
          self.nextExpectedToken(Some(Keyword::TIME.into( )))?;

          match self.nextToken( )? {
            Token::Number(n) => asOfVersion= Some(n.parse::<u64>( )?),
            token => return Err(Error::Parse(format!("Unexpected token {}, wanted number", token)))
          }
        }

        Ok(Statement::Begin { readonly, asOfVersion })
      },

      Token::Keyword(Keyword::COMMIT) => Ok(Statement::Commit),
      Token::Keyword(Keyword::ROLLBACK) => Ok(Statement::Rollback),

      token => Err(Error::Parse(format!("Unexpected token {}", token))),
    }
  }

  fn parseInsertStatement(&mut self) -> Result<Statement> {
    self.nextExpectedToken(Some(Keyword::INSERT.into( )))?;
    self.nextExpectedToken(Some(Keyword::INTO.into( )))?;
    let table= self.nextIdentifier( )?;

    let columns=
      if self.nextTokenIfIts(Token::OpenParenthesis).is_some( ) {
        let mut columns= vec![ ];
        loop {
          columns.push(self.nextIdentifier( )?);
          match self.nextToken( )? {
            Token::CloseParenthesis => break,
            Token::Comma => continue,
            token => return Err(Error::Parse(format!("Unexpected token {}", token))),
          }
        }
        Some(columns)
      }
      else { None };

    self.nextExpectedToken(Some(Keyword::VALUES.into( )))?;

    let mut values= vec![ ];
    loop {
      self.nextExpectedToken(Some(Token::OpenParenthesis))?;
      let mut expressions= vec![ ];
      loop {
        expressions.push(self.parseExpression(0)?);
        match self.nextToken( )? {
          Token::CloseParenthesis => break,
          Token::Comma => continue,
          token => return Err(Error::Parse(format!("Unexpected token {}", token))),
        }
      }
      values.push(expressions);

      if self.nextTokenIfIts(Token::Comma).is_none( ) {
        break
      }
    }

    Ok(Statement::Insert { table, columns, values })
  }

  fn parseSelectStatement(&mut self) -> Result<Statement> {
    Ok(Statement::Select {
      selections: self.parseSelectClause( )?,
      from:       self.parseFromClause( )?,
      r#where:    self.parseWhereClause( )?,
      groupBy:    self.parseGroupByClause( )?,
      having:     self.parseHavingClause( )?,
      order:      self.parseOrderClause( )?,

      limit: self.nextTokenIfIts(Keyword::LIMIT.into( ))
              .map(|_| self.parseExpression(0)).transpose( )?,

      offset: self.nextTokenIfIts(Keyword::OFFSET.into( ))
                .map(|_| self.parseExpression(0)).transpose( )?,
    })
  }

  fn parseUpdateStatement(&mut self) -> Result<Statement> {
    self.nextExpectedToken(Some(Keyword::UPDATE.into( )))?;
    let table= self.nextIdentifier( )?;

    self.nextExpectedToken(Some(Keyword::SET.into( )))?;
    let mut updates= BTreeMap::new( );
    loop {
      let column= self.nextIdentifier( )?;
      self.nextExpectedToken(Some(Token::Equal))?;
      let value= self.parseExpression(0)?;

      if updates.contains_key(&column) {
        return Err(Error::Value(format!("Duplicate values for column {}", &column)))}
      updates.insert(column, value);

      if self.nextTokenIfIts(Token::Comma).is_none( ) {
        break
      }
    }

    Ok(Statement::Update { table, updates, r#where: self.parseWhereClause( )? })
  }

  fn parseDeleteStatement(&mut self) -> Result<Statement> {
    self.nextExpectedToken(Some(Keyword::DELETE.into( )))?;
    self.nextExpectedToken(Some(Keyword::FROM.into( )))?;

    Ok(Statement::Delete { table: self.nextIdentifier( )?, r#where: self.parseWhereClause( )? })
  }

  fn parseExplainStatement(&mut self) -> Result<Statement> {
    self.nextExpectedToken(Some(Keyword::EXPLAIN.into( )))?;
    if let Some(Token::Keyword(Keyword::EXPLAIN)) = self.peekNextToken( )? {
      return Err(Error::Parse("Cannot nest EXPLAIN statements".into( )))}

    Ok(Statement::Explain(Box::new(self.parseStatement( )?)))
  }

  fn parseSelectClause(&mut self) -> Result<Vec<(Expression, Option<AliasColumnName>)>> {
    self.nextExpectedToken(Some(Keyword::SELECT.into( )))?;

    let mut selections= vec![ ];
    loop {
      if self.nextTokenIfIts(Token::Asterisk).is_some( ) && selections.is_empty( ) {
        return Ok(selections)}

      let expression= self.parseExpression(0)?;
      let label= match self.peekNextToken( )? {

        Some(Token::Keyword(Keyword::AS)) => {
          let _= self.nextToken( )?;
          Some(self.nextIdentifier( )?)
        },
        Some(Token::Identifier(_)) => Some(self.nextIdentifier( )?),

        _ => None
      };
      selections.push((expression, label));

      if self.nextTokenIfIts(Token::Comma).is_none( ) {
        break
      }
    }
    Ok(selections)
  }

  fn parseFromClause(&mut self) -> Result<Vec<SearchField>> {
    self.nextExpectedToken(Some(Keyword::FROM.into( )))?;

    let mut searchFields= vec![ ];
    loop {
      let mut searchField= self.parseFromTableClause( )?;

      while let Some(joinType)= self.parseJoinClause( )? {
        searchField= SearchField::Join {
          left: Box::new(searchField),
          right: Box::new(self.parseFromTableClause( )?),

          predicate: match &joinType {
            JoinType::Cross => None,

            _ => {
              self.nextExpectedToken(Some(Keyword::ON.into( )))?;
              Some(self.parseExpression(0)?)
            }
          },
 
          r#type: joinType
        };
      }

      searchFields.push(searchField);

      if self.nextTokenIfIts(Token::Comma).is_none( ) {
        break
      }
    }
    Ok(searchFields)
  }

  fn parseFromTableClause(&mut self) -> Result<SearchField> {
    let tablename= self.nextIdentifier( )?;
    let alias= match self.peekNextToken( )? {

      Some(Token::Keyword(Keyword::AS)) => {
        let _= self.nextToken( )?;
        Some(self.nextIdentifier( )?)
      },
      Some(Token::Identifier(_)) => Some(self.nextIdentifier( )?),

      _ => None
    };
    Ok(SearchField::Table { name: tablename, alias })
  }

  fn parseJoinClause(&mut self) -> Result<Option<JoinType>> {
    // Only writing JOIN indiciates an INNER JOIN.
    if self.nextTokenIfIts(Keyword::JOIN.into( )).is_some( ) {
      return Ok(Some(JoinType::Inner))}

    let joinType= match self.peekNextToken( )? {
      Some(Token::Keyword(keyword)) => match keyword {
        Keyword::CROSS => JoinType::Cross,
        Keyword::INNER => JoinType::Inner,
        Keyword::LEFT => JoinType::Left,
        Keyword::RIGHT => JoinType::Right,

        _ => return Ok(None)
      },
      _ => return Ok(None),
    };
    self.nextExpectedToken(Some(Keyword::JOIN.into( )))?;
    Ok(Some(joinType))
  }

  fn parseWhereClause(&mut self) -> Result<Option<Expression>> {
    if self.nextTokenIfIts(Keyword::WHERE.into( )).is_none( ) {
      return Ok(None)}
    Ok(Some(self.parseExpression(0)?))
  }

  fn parseGroupByClause(&mut self) -> Result<Vec<Expression>> {
    let mut groupingParameters= vec![ ];

    if self.nextTokenIfIts(Keyword::WHERE.into( )).is_none( ) {
      return Ok(groupingParameters)}

    self.nextExpectedToken(Some(Keyword::BY.into( )))?;

    loop {
      groupingParameters.push(self.parseExpression(0)?);
      if self.nextTokenIfIts(Token::Comma).is_none( ) {
        break
      }
    }
    Ok(groupingParameters)
  }

  fn parseHavingClause(&mut self) -> Result<Option<Expression>> {
    if self.nextTokenIfIts(Keyword::HAVING.into( )).is_none( ) {
      return Ok(None)}
    Ok(Some(self.parseExpression(0)?))
  }

  fn parseOrderClause(&mut self) -> Result<Vec<(Expression, Order)>> {
    if self.nextTokenIfIts(Keyword::ORDER.into( )).is_none( ) {
      return Ok(vec![ ])}

    self.nextExpectedToken(Some(Keyword::BY.into( )))?;

    let mut orderingRules= vec![ ];
    loop {
      orderingRules.push((
        self.parseExpression(0)?,
        self.nextTokenIfIts(Keyword::DESC.into( ))
          .map(|_| Order::Descending)
          .unwrap_or(Order::Ascending)
      ));

      if self.nextTokenIfIts(Token::Comma).is_none( ) {
        break
      }
    }
    Ok(orderingRules)
  }

  // Parses an expression containing atleast one operand operated on by any number of operands.
  // An example expression : -5 * 2! + 3.
  // NOTE : It uses the Precedance Climbing Algorithm.
  // FIX: Case -5! - since factorials of negative numbers cannot be calculated.
  fn parseExpression(&mut self, minOperatorPrecedance: Precedance) -> Result<Expression> {
    let mut lhs=
      if let Some(prefixOperator)= self.nextIfOperator::<PrefixOperator>(minOperatorPrecedance)? {
        self.parseExpression(minOperatorPrecedance + prefixOperator.associativity( ) as Precedance)?}
      else {
        self.parseExpressionOperand( )?};

    while let Some(postfixOperator)= self.nextIfOperator::<PostfixOperator>(minOperatorPrecedance)? {
      lhs= postfixOperator.operate(lhs);}

    while let Some(infixOperator)= self.nextIfOperator::<InfixOperator>(minOperatorPrecedance)? {
      lhs= infixOperator.operate(lhs, self.parseExpression(minOperatorPrecedance + infixOperator.associativity( ) as Precedance)?);}

    Ok(lhs)
  }

  fn parseExpressionOperand(&mut self) -> Result<Expression> {
    Ok(match self.nextToken( )? {

      Token::Identifier(identifier) => {
        if self.nextTokenIfIts(Token::OpenParenthesis).is_some( ) {
          let mut arguments= vec![ ];

          while self.nextTokenIfIts(Token::CloseParenthesis).is_some( ) {
            if !arguments.is_empty( ) {
              self.nextExpectedToken(Some(Token::Comma))?;}

            arguments.push(
              // Handling COUNT(*).
              if (identifier == "COUNT") && self.nextExpectedToken(Some(Token::Asterisk))?.is_some( ) {
                Literal::Boolean(true).into( )}

              else { self.parseExpression(0)? }
            );
          }

          Expression::FunctionCall(identifier, arguments)
        }
        else {
          let mut field= self.nextIdentifier( )?;

          let mut relation= None;
          if self.nextTokenIfIts(Token::Period).is_some( ) {
            relation= Some(field);
            field= self.nextIdentifier( )?;
          }

          Expression::Field(relation, field)
        }
      },

      Token::Number(value) =>
        if value.chars( ).all(|character| character.is_ascii_digit( )) {
          Literal::Integer(value.parse( )?).into( )}
        else {
          Literal::Float(value.parse( )?).into( )},

      Token::OpenParenthesis => {
        let expression= self.parseExpression(0)?;
        self.nextExpectedToken(Some(Token::CloseParenthesis.into( )))?;
        expression
      },

      Token::String(value) => Literal::String(value).into( ),

      Token::Keyword(Keyword::FALSE) => Literal::Boolean(false).into( ),
      Token::Keyword(Keyword::TRUE) => Literal::Boolean(true).into( ),

      Token::Keyword(Keyword::INFINITY) => Literal::Float(f64::INFINITY).into( ),
      Token::Keyword(Keyword::NAN) => Literal::Float(f64::NAN).into( ),

      Token::Keyword(Keyword::NULL) => Literal::Null.into( ),

      token => return Err(Error::Parse(format!("Expected expression operand, found {}", token))),
    })
  }
}

impl<'a> Parser<'a> {
  pub fn new(input: &'a str) -> Self {
    return Parser {
      lexer: Lexer::new(input).peekable( )
    }
  }

  // Gets the next lexed token and returns it. Returns error, if not found.
  fn nextToken(&mut self) -> Result<Token> {
    self.lexer.next( )
      .unwrap_or_else(| | Err(Error::Parse("Unexpected end of tokens".into( ))))
  }

  // Peeks for the next lexed token and returns it.
  fn peekNextToken(&mut self) -> Result<Option<Token>> {
    self.lexer.peek( ).cloned( ).transpose( )
  }

  // If the next lexed token matches the given expected token, then grabs and returns it. Otherwise,
  // returns error.
  fn nextExpectedToken(&mut self, expectedToken: Option<Token>) -> Result<Option<Token>> {
    if let Some(expectedToken)= expectedToken {
      let actualNextToken= self.nextToken( )?;
      if actualNextToken == expectedToken {
        return Ok(Some(actualNextToken))}
      return Err(Error::Parse(format!("Expected token {}, got {}", expectedToken, actualNextToken)))
    }

    else if let Some(actualNextToken)= self.peekNextToken( )? {
      return Err(Error::Parse(format!("Unexpected token {}", actualNextToken)))}

    Ok(None)
  }

  // Gets the next lexed identifier token and returns it. Returns error, if not found.
  fn nextIdentifier(&mut self) -> Result<String> {
    match self.nextToken( )? {
      Token::Identifier(identifier) => Ok(identifier),
      token => Err(Error::Parse(format!("Expected identifier, got {}", token)))
    }
  }

  // Grabs and returns the next token, if it satisfies the given predicate function.
  fn nextTokenIf<F: Fn(&Token) -> bool>(&mut self, predicate: F) -> Option<Token> {
    self.peekNextToken( ).unwrap_or(None)
      .filter(|peekedNextToken| predicate(peekedNextToken))?;
    self.nextToken( ).ok( )
  }

  // Grabs and returns the next token, if it matches the given expected token.
  fn nextTokenIfIts(&mut self, expectedToken: Token) -> Option<Token> {
    self.nextTokenIf(|nextToken| nextToken == &expectedToken)
  }

  // Grabs and returns the next token, if it's a keyword.
  fn nextTokenIfItsKeyword(&mut self) -> Option<Token> {
    self.nextTokenIf(|nextToken| matches!(nextToken, Token::Keyword(_)))
  }

  // Grabs and returns the next operator, if it satisfies the given operator type and has precedance
  // higher than the given minimum precedance.
  // It will return error if operator augmentation goes wrong.
  fn nextIfOperator<O: Operator>(&mut self, minPrecedence: Precedance) -> Result<Option<O>> {
    if let Some(operator)= self.peekNextToken( )
                               .unwrap_or(None)
                               .and_then(|token| O::fromToken(&token))
                               .filter(|operator| operator.precedance( ) >= minPrecedence)
    {
      self.nextToken( )?;
      return Ok(Some(operator.augment(self)?))
    }

    Ok(None)
  }
}