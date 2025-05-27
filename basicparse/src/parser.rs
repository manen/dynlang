use std::{iter::Peekable, marker::PhantomData};

use crate::*;

use iter_read_until::{IntoReader, Reader};
use langlib::{Block, Expr, Function, Reach, Statement, Value};

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
impl<I: Iterator<Item = Result<Token>>> Parser<I> {
	pub fn from_iter(iter: impl IntoIterator<IntoIter = I>) -> Self {
		Self {
			iter: iter.into_iter().peekable(),
		}
	}

	pub fn read_reach(&mut self) -> Result<Reach> {
		let a = self.iter.next().ok_or(Error::EOFReach)??;
		match a {
			Token::Ident(name) => Ok(Reach::Named(name)),
			Token::StrLit(s) => Ok(Reach::Value(Value::String(s))),
			Token::NumLit(s) => match s.parse() {
				Ok(a) => Ok(Reach::Value(Value::i32(a))),
				Err(i32err) => match s.parse() {
					Ok(a) => Ok(Reach::Value(Value::f32(a))),
					Err(f32err) => return Err(Error::InvalidNumLit { f32err, i32err }),
				},
			},
			Token::Fn => {
				let parens = self.iter.next().ok_or(Error::ExpectedFnDeclParens)??;
				assert_eq!(parens, Token::Parens(Vec::new()));
				let block = self.read_block()?;

				Ok(Reach::Value(Value::Function(Function { block })))
			}
			_ => unimplemented!("{a:?} as reach"),
		}
	}
	fn expand_expr_internal(&mut self, expr: Expr) -> Result<Expr> {
		let a = match self.iter.peek() {
			Some(Ok(Token::Plus)) => {
				self.iter.next();
				let reach = expr.into_reach();

				let b = self.read_expr()?;
				let b = match b {
					Expr::Reach(r) => r,
					b => Reach::Expr(Box::new(b)),
				};

				Expr::Add(reach, b)
			}
			Some(Ok(Token::Parens(l))) if l.len() == 0 => {
				self.iter.next();
				Expr::CallFn(expr.into_reach())
			}
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
		let reach = self.read_reach()?;
		self.expand_expr(reach.into_expr())
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
						let expr = self.read_expr().with_context(|| {
							format!(
								"failed to read value of variable while declaring variable {name}"
							)
						})?;
						return Ok(Statement::SetVariable(name, expr));
					} else {
						return Err(Error::ExpectedEqLet);
					}
				} else {
					return Err(Error::ExpectedVariableName);
				}
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
}

pub struct ParserStatements<I: Iterator<Item = Result<Token>>> {
	parser: Parser<I>,
}
impl<I: Iterator<Item = Result<Token>>> Iterator for ParserStatements<I> {
	type Item = Result<Statement>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.parser.read_statement() {
			Err(Error::EOFStatement) => None,
			els => Some(els),
		}
	}
}
