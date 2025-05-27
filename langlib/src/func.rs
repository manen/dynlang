use crate::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
	ModifyVariable(String, Expr),
	SetVariable(String, Expr),
	Return(Expr),
	/// executes the expr and does nothing with the output value \
	/// UNLESS! if it's the last instruction in a block, then it gets returned
	Expr(Expr),

	/// debug
	DumpContext,
	/// debug, pauses execution until a key is pressed
	Pause,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Block(pub Vec<Statement>);
impl Block {
	pub fn iter(&self) -> impl Iterator<Item = &Statement> {
		self.0.iter()
	}
}

#[derive(Clone, Debug, PartialEq)]
/// functions don't have args for now lol
pub struct Function {
	pub block: Block,
}
impl Function {
	pub fn new<I: IntoIterator<Item = Statement>>(statements: I) -> Self {
		let block = Block(statements.into_iter().collect());
		Self { block }
	}
}
