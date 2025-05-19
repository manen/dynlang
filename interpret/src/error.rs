use crate::*;

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
	#[error("attempted to access variable '{0}' that doesn't exist")]
	VariableDoesntExist(String),

	#[error("attempted invalid addition operation {a:?} + {b:?}")]
	InvalidAddition { a: Value, b: Value },
}
pub type Result<T, E = Error> = core::result::Result<T, E>;
