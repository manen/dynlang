use crate::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
	SetVariable(String, Expr),
	Return(Expr),
	/// debug
	DumpContext,
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
