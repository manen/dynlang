use std::iter::Peekable;

use crate::*;

use langlib::{Block, Expr, Function, Index, IntoIndex, Reach, Statement, Value};

#[derive(Clone, Debug)]
pub struct Parser<I: Iterator<Item = Result<Token>>> {
	iter: Peekable<I>,
}
impl<'a> Parser<Tokenizer<'a>> {
	pub fn new(src: &'a str) -> Self {
		Self {
			iter: Tokenizer::new(src).peekable(),
		}
	}
}
impl<I: Iterator<Item = Result<Token>> + Clone> Parser<I> {
	pub fn from_iter(iter: impl IntoIterator<IntoIter = I>) -> Self {
		Self {
			iter: iter.into_iter().peekable(),
		}
	}

	pub fn read_reach(&mut self) -> Result<Reach> {
		let a = self.iter.next().ok_or(Error::EOFReach)??;
		match a {
			Token::Ident(name) => {
				if name == "obj" {
					if let Some(Ok(Token::Curly(_))) = self.iter.peek() {
						if let Some(Ok(Token::Curly(map))) = self.iter.next() {
							// object literal
							let parser = Parser::from_iter(map.into_iter().map(Ok)).object();
							return Ok(Reach::ObjectLiteral(
								parser.collect::<Result<Vec<_>, _>>()?,
							));
						} else {
							panic!("peeked != iter.next()");
						}
					}
				}

				// regular variable reference
				Ok(Reach::Named(name))
			}
			Token::StrLit(s) => Ok(Reach::Value(Value::String(s))),
			Token::NumLit(s) => match s.parse() {
				Ok(a) => Ok(Reach::Value(Value::i32(a))),
				Err(i32err) => match s.parse() {
					Ok(a) => Ok(Reach::Value(Value::f32(a))),
					Err(f32err) => return Err(Error::InvalidNumLit { f32err, i32err }),
				},
			},
			Token::Brackets(b) => {
				// array literal
				let parser = Parser::from_iter(b.into_iter().map(Ok)).comma_separated_expressions();
				let elements = parser.collect::<Result<Vec<_>, _>>().with_context(|| {
					format!("while parsing expressions inside an array literal")
				})?;
				Ok(Reach::ArrayLiteral(elements))
			}
			Token::Fn => {
				let parens = self.iter.next().ok_or(Error::ExpectedFnDeclParens)??;
				if let Token::Parens(parens) = parens {
					let mut arg_names = parens.into_iter().map(|token| match token {
						Token::Ident(name) => Ok(name),
						token => Err(Error::ExpectedIdentGot(token)),
					});
					let arg_name = arg_names.next();
					let arg_name = if let Some(arg_name) = arg_name {
						Some(arg_name?)
					} else {
						None
					};
					assert!(
						arg_names.next().is_none(),
						"only one argument per function for now"
					);

					let block = self.read_block()?;
					Ok(Reach::Value(Value::Function(Function { arg_name, block })))
				} else {
					Err(Error::ExpectedFnDeclParens)
				}
			}
			Token::Parens(parens) => {
				let mut parser = Parser::from_iter(parens.into_iter().map(Ok));
				let expr = parser.read_expr().with_context(|| {
					format!(
						"while reading inside parentheses (reading a reach that's an expr in disguise)"
					)
				})?;
				Ok(Reach::Expr(Box::new(expr)))
			}
			_ => unimplemented!("{a:?} as reach"),
		}
	}
	fn expand_expr_internal(&mut self, expr: Expr) -> Result<Expr> {
		let a = match self.iter.peek() {
			Some(Ok(Token::Plus)) => {
				self.iter.next();
				let reach = expr.into_reach();

				let b = self.read_expr()?.into_reach();

				Expr::Add(reach, b)
			}
			Some(Ok(Token::Minus)) => {
				self.iter.next();

				let reach = expr.into_reach();
				let b = self.read_expr()?.into_reach();

				Expr::Sub(reach, b)
			}
			Some(Ok(Token::Dot)) => {
				self.iter.next();

				let reach = expr.into_reach();
				let b = self.read_reach().with_context(|| {
					format!("while reading right-hand side of . indexing access")
				})?;

				let index = match b {
					Reach::Named(name) => IntoIndex::Index(Index::Ident(name)),
					Reach::Value(Value::i32(i)) => IntoIndex::Index(Index::NumLit(i)),
					Reach::ArrayLiteral(arr) if arr.len() == 1 => IntoIndex::Expr(Box::new(
						arr.into_iter()
							.next()
							.expect("we just checked there's at least one element"),
					)),
					_ => return Err(Error::InvalidIndex),
				};

				Expr::Index(reach, index)
			}
			Some(Ok(Token::Eq)) => {
				let mut clone = self.clone();
				clone.iter.next();
				if let Some(Ok(Token::Eq)) = clone.iter.peek() {
					self.iter.next();
					self.iter.next();

					let b = self
						.read_expr()
						.with_context(|| format!("while reading right side of equality check"))?;
					Expr::Cmp(expr.into_reach(), b.into_reach())
				} else {
					return Err(Error::ExprExpand(expr)); // base case from outer match
				}
			}
			Some(Ok(Token::Gt)) => {
				self.iter.next();

				let b = self
					.read_expr()
					.with_context(|| format!("while reading right side of greater than check"))?;

				Expr::Gt(expr.into_reach(), b.into_reach())
			}
			Some(Ok(Token::Lt)) => {
				self.iter.next();

				let b = self
					.read_expr()
					.with_context(|| format!("while reading right side of less than check"))?;

				Expr::Lt(expr.into_reach(), b.into_reach())
			}
			Some(Ok(Token::Or)) => {
				self.iter.next();

				let b = self.read_expr().with_context(|| {
					format!("while reading right side of boolean or expression")
				})?;

				Expr::Or(expr.into_reach(), b.into_reach())
			}
			Some(Ok(Token::And)) => {
				self.iter.next();

				let b = self.read_expr().with_context(|| {
					format!("while reading right side of boolean and operation")
				})?;

				Expr::And(expr.into_reach(), b.into_reach())
			}
			Some(Ok(Token::Parens(_))) => match self.iter.next() {
				Some(Ok(Token::Parens(l))) => {
					let args = if l.len() > 0 {
						let mut parser = Parser::from_iter(l.into_iter().map(Ok));
						let expr = parser.read_expr()?;
						Some(expr.into_reach())
					} else {
						None
					};
					Expr::CallFn {
						f: expr.into_reach(),
						args,
					}
				}
				_ => panic!("iter.peek() has to be equal to iter.next() this is impossible"),
			},
			None | Some(_) => return Err(Error::ExprExpand(expr)),
		};
		Ok(a)
	}
	fn expand_expr(&mut self, expr: Expr) -> Result<Expr> {
		match self.expand_expr_internal(expr) {
			Err(Error::ExprExpand(r)) => {
				// we can't expand further
				Ok(r)
			}
			Err(err) => Err(err),
			Ok(a) => {
				// we could expand and we're gonna try to expand again
				self.expand_expr(a)
			}
		}
	}
	pub fn read_expr(&mut self) -> Result<Expr> {
		let peek = self.iter.peek();
		let reach = match peek {
			Some(Ok(Token::If)) => {
				self.iter.next();
				let cond = self
					.read_expr()
					.with_context(|| format!("while reading condition in if statement"))?;

				let if_true = self
					.read_block()
					.with_context(|| format!("while reading if true branch in if statement"))?;

				let if_false = match self.iter.peek() {
					Some(Ok(Token::Else)) => {
						self.iter.next();
						let if_false = self.read_block().with_context(|| {
							format!("while reading else branch in if statement")
						})?;
						Reach::Expr(Box::new(Expr::Block(if_false)))
					}
					_ => Reach::Value(Value::None),
				};

				Expr::Conditional {
					condition: Reach::Expr(Box::new(cond)),
					if_true: Reach::Expr(Box::new(Expr::Block(if_true))),
					if_false,
				}
			}
			_ => self.read_reach()?.into_expr(),
		};
		// above is if statement parsing

		self.expand_expr(reach)
	}
	pub fn read_statement(&mut self) -> Result<Statement> {
		let peek = self.iter.peek().ok_or(Error::EOFStatement)?.clone()?;
		match peek {
			Token::Let => {
				self.iter.next();
				let name = self.iter.next().ok_or(Error::ExpectedVariableName)??;
				if let Token::Ident(name) = name {
					let eq = self.iter.next().ok_or(Error::ExpectedEqLet)??;
					if let Token::Eq = eq {
						let expr = self
							.read_expr()
							.with_context(|| format!("while declaring variable {name}"))?;
						return Ok(Statement::SetVariable(name, expr));
					} else {
						return Err(Error::ExpectedEqLet);
					}
				} else {
					return Err(Error::ExpectedVariableName);
				}
			}
			Token::Loop => {
				self.iter.next();
				let b = self
					.read_block()
					.with_context(|| format!("while reading loop block"))?;
				return Ok(Statement::Loop(b));
			}
			Token::Break => {
				self.iter.next();
				return Ok(Statement::Break);
			}
			Token::For => {
				self.iter.next();

				let v_ident = self
					.iter
					.next()
					.ok_or_else(|| Error::ExpectedIdentFor(None))??;
				let v_name = match v_ident {
					Token::Ident(name) => name,
					other => return Err(Error::ExpectedIdentFor(Some(other))),
				};

				let in_token = self
					.iter
					.next()
					.ok_or_else(|| Error::ExpectedInFor(None))??;
				match in_token {
					Token::In => {}
					other => return Err(Error::ExpectedInFor(Some(other))),
				}

				let iter = self.read_expr().with_context(|| {
					format!("as iterator in for loop with variable name {v_name:?}")
				})?;

				let block = self
					.read_block()
					.with_context(|| format!("in a for loop that uses variable name {v_name:?}"))?;

				return Ok(Statement::LoopFor {
					v_name,
					iter,
					block,
				});
			}
			_ => {}
		}

		let expr = self
			.read_expr()
			.with_context(|| format!("while reading an expr in a statement"))?;
		match &expr {
			Expr::Reach(Reach::Named(name)) => match self.iter.peek().cloned() {
				Some(Ok(Token::Eq)) => {
					self.iter.next();
					let expr = self.read_expr()?;
					Ok(Statement::ModifyVariable(name.clone(), expr))
				}
				Some(Err(err)) => Err(err)?,
				_ => Ok(Statement::Expr(expr)),
			},
			_ => Ok(Statement::Expr(expr)),
		}
	}
	pub fn read_block(&mut self) -> Result<Block> {
		match self.iter.next().ok_or(Error::ExpectedBlock)?? {
			Token::Curly(inner) => {
				let inner = inner.into_iter().map(Ok).collect::<Vec<_>>();
				let parser = preproc(Parser::from_iter(inner).statements());
				let block = parser.collect::<Result<Vec<_>, _>>()?;
				Ok(Block(block))
			}
			_ => return Err(Error::ExpectedBlock),
		}
	}

	pub fn statements(self) -> ParserStatements<I> {
		ParserStatements { parser: self }
	}
	pub fn comma_separated_expressions(self) -> ParserCSE<I> {
		ParserCSE { parser: self }
	}
	pub fn object(self) -> ParserObj<I> {
		ParserObj { parser: self }
	}
}

pub struct ParserStatements<I: Iterator<Item = Result<Token>> + Clone> {
	parser: Parser<I>,
}
impl<I: Iterator<Item = Result<Token>> + Clone> Iterator for ParserStatements<I> {
	type Item = Result<Statement>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.parser.read_statement() {
			Err(Error::EOFStatement) => None,
			els => Some(els),
		}
	}
}

/// comma separated expressions \
/// expect they're really not even separated by a comma but shshsh
pub struct ParserCSE<I: Iterator<Item = Result<Token>> + Clone> {
	parser: Parser<I>,
}
impl<I: Iterator<Item = Result<Token>> + Clone> Iterator for ParserCSE<I> {
	type Item = Result<Expr>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.parser.read_expr() {
			Err(Error::EOFExpr) => None,
			Err(Error::EOFReach) => None,
			els => Some(els),
		}
	}
}

pub struct ParserObj<I: Iterator<Item = Result<Token>> + Clone> {
	parser: Parser<I>,
}
impl<I: Iterator<Item = Result<Token>> + Clone> Iterator for ParserObj<I> {
	type Item = Result<(String, Expr)>;

	fn next(&mut self) -> Option<Self::Item> {
		let name = match self.parser.iter.next()? {
			Ok(Token::Ident(name) | Token::StrLit(name)) => name,
			Ok(Token::NumLit(name)) => name,
			Err(err) => {
				return Some(Err(
					err.with_context(format!("while parsing an object literal"))
				));
			}
			Ok(_) => return Some(Err(Error::ExpectedIdentObj)),
		};

		let colon = self.parser.iter.next();
		match colon {
			Some(Ok(Token::Colon)) => {
				// yippee
			}
			_ => {
				return Some(Err(match colon {
					None => Error::ExpectedColonObj(None),
					Some(Ok(t)) => Error::ExpectedColonObj(Some(t)),
					Some(Err(err)) => err.with_context(format!(
						"while trying to parse a colon in an object literal"
					)),
				}));
			}
		}

		let val = self
			.parser
			.read_expr()
			.with_context(|| format!("while parsing an object literal"));
		let val = match val {
			Ok(a) => a,
			Err(err) => return Some(Err(err)),
		};

		// all of this just so we can read key-value pairs fml

		Some(Ok((name, val)))
	}
}
