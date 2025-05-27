use std::{iter::Peekable, marker::PhantomData};

use crate::*;

use iter_read_until::{IntoReader, Reader};
use langlib::{Expr, Statement, Value};

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
	pub fn from_iter(iter: I) -> Self {
		Self {
			iter: iter.peekable(),
		}
	}

	pub fn read_expr(&mut self) -> Result<Expr> {
		let a = self.iter.next().ok_or(Error::EOFExpr)??;
		todo!("{a:?}")
	}
	pub fn read_statement(&mut self) -> Result<Statement> {
		let a = self.iter.next().ok_or(Error::EOFStatement)??;
		match a {
			Token::Let => {
				let name = self.iter.next().ok_or(Error::ExpectedVariableName)??;
				if let Token::Ident(name) = name {
					let eq = self.iter.next().ok_or(Error::ExpectedEq)??;
					if let Token::Eq = eq {
						let expr = self.read_expr().with_context(|| {
							format!(
								"failed to read value of variable while declaring variable {name}"
							)
						})?;
						Ok(Statement::SetVariable(name, expr))
					} else {
						return Err(Error::ExpectedEq);
					}
				} else {
					return Err(Error::ExpectedVariableName);
				}
			}
			_ => todo!(),
		}
	}
}
