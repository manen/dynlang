use crate::*;

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
	#[error("attempted to access variable '{0}' that doesn't exist")]
	VariableDoesntExist(String),

	#[error("attempted invalid addition operation {a:?} + {b:?}")]
	InvalidAddition { a: Value, b: Value },
	#[error("attempted to call a variable that isn't a function: {0:?}")]
	NotAFunction(Value),
}
pub type Result<T, E = Error> = core::result::Result<T, E>;
