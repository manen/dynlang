use crate::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
	ModifyVariable(String, Expr),
	SetVariable(String, Expr),
	/// doesn't really behave like you'd expect to but it's fine (there's literally no way
	/// to even make an explicit return statement if using basicparse so it's fine)
	Return(Expr),
	/// executes the expr and does nothing with the output value \
	/// UNLESS! if it's the last instruction in a block, then it gets returned
	Expr(Expr),

	/// loops over the given block until broken out with break
	Loop(Block),
	Break,
	LoopFor {
		v_name: String,
		iter: Expr,
		block: Block,
	},

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
	/// the arg name (if any)
	pub arg_name: Option<String>,
	pub block: Block,
}
impl Function {
	pub fn new<I: IntoIterator<Item = Statement>>(arg_name: Option<String>, statements: I) -> Self {
		let block = Block(statements.into_iter().collect());
		Self { arg_name, block }
	}
}
impl Display for Function {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "fn({})", self.arg_name.as_deref().unwrap_or(""))
	}
}
