use std::num::{ParseFloatError, ParseIntError};

use crate::Token;

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
	#[error("reader error while tokenizing:\n{0}")]
	Read(#[from] readuntil_ext::Error),
	#[error("{context}:\n{err}")]
	Context { context: String, err: Box<Self> },

	#[error("tokenizer tokenized everything in its reader. this error is hidden most of the time")]
	TokenizerFinished,

	#[error("unexpected end of input while reading expr")]
	EOFExpr,
	#[error("unexpected end of input while reading statement")]
	EOFStatement,
	#[error("unexpected end of input while reading reach (value or named variable)")]
	EOFReach,
	#[error("expected name of variable after let")]
	ExpectedVariableName,
	#[error("expected eq sign after name of variable in variable declaration")]
	ExpectedEq,

	#[error("invalid number literal, couldn't parse as i32 or f32")]
	InvalidNumLit {
		i32err: ParseIntError,
		f32err: ParseFloatError,
	},
	#[error("invalid first token while reading a reach: {0:?}")]
	InvalidFirstReach(Token),
	#[error("expected parens at function declaration")]
	ExpectedFnDeclParens,
	#[error("expected block/opening curly braces")]
	ExpectedBlock,
}
impl Error {
	pub fn with_context(self, context: String) -> Self {
		Self::Context {
			context,
			err: Box::new(self),
		}
	}
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

pub trait ResultExt {
	type T;
	type E: Into<Error>;

	fn with_context(self, f: impl FnOnce() -> String) -> Result<Self::T>;
}
impl<T, E: Into<Error>> ResultExt for Result<T, E> {
	type T = T;
	type E = E;

	fn with_context(self, f: impl FnOnce() -> String) -> Result<Self::T> {
		self.map_err(Into::into)
			.map_err(|err| err.with_context(f()))
	}
}
