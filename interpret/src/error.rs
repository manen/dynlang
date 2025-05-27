use crate::*;

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
	#[error("attempted to {1} variable '{0}' that doesn't exist")]
	VariableDoesntExist(String, VariableAccessType),

	#[error("attempted invalid addition operation {a:?} + {b:?}")]
	InvalidAddition { a: Value, b: Value },
	#[error("attempted invalid subtraction operation {a:?} - {b:?}")]
	InvalidSubtraction { a: Value, b: Value },
	#[error("attempted invalid greater than operation {a:?} > {b:?}")]
	InvalidGt { a: Value, b: Value },
	#[error("attempted invalid less than operation {a:?} < {b:?}")]
	InvalidLt { a: Value, b: Value },
	#[error("attempted to call a variable that isn't a function: {0:?}")]
	NotAFunction(Value),

	#[error("missing argument to function: expected arg {0}")]
	MissingArg(String),

	#[error(
		"impossible case: we checked if this variable existed before and it did, but when replacing it with a new value it turns out it didn't exist"
	)]
	Impossible1,
}
pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Clone, Debug, thiserror::Error)]
pub enum VariableAccessType {
	#[error("access")]
	Access,
	#[error("modify")]
	Modify,
}
