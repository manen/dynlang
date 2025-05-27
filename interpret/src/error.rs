use crate::*;

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
	#[error("attempted to {1} variable '{0}' that doesn't exist")]
	VariableDoesntExist(String, VariableAccessType),

	#[error("attempted invalid addition operation {a:?} + {b:?}")]
	InvalidAddition { a: Value, b: Value },
	#[error("attempted to call a variable that isn't a function: {0:?}")]
	NotAFunction(Value),

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
