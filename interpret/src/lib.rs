use std::collections::HashMap;

mod error;
pub use error::*;

use langlib::*;

#[derive(Clone, Debug, Default)]
pub struct Context {
	variables: HashMap<String, Value>,
}
impl Context {
	pub fn new<I: IntoIterator<Item = (String, Value)>>(variables: I) -> Self {
		let variables = variables.into_iter();

		Self {
			variables: variables.collect(),
		}
	}

	pub fn resolve_reach(&self, r: &Reach) -> Result<Value> {
		match r {
			Reach::Value(val) => Ok(val.clone()),
			Reach::Expr(expr) => self.resolve_expr(expr),
			Reach::Named(name) => self
				.variables
				.get(name)
				.cloned() // .
				.ok_or_else(|| Error::VariableDoesntExist(name.clone())),
		}
	}
	pub fn resolve_expr(&self, expr: &Expr) -> Result<Value> {
		match expr {
			Expr::Reach(r) => self.resolve_reach(r),
			Expr::Add(a, b) => {
				let a = self.resolve_reach(a)?;
				let b = self.resolve_reach(b)?;

				a.add(&b).ok_or_else(|| Error::InvalidAddition { a, b })
			}
		}
	}

	/// runs the given block as-is. does not isolate context at all so unless you wanna leak
	/// internal variables you should probably use `context.clone().resolve_block()`
	pub fn resolve_block(&mut self, block: &Block) -> Result<Option<Value>> {
		for stmt in block.iter() {
			match stmt {
				Statement::SetVariable(name, val) => {
					let val = self.resolve_expr(val)?;
					self.variables.insert(name.clone(), val);
				}
				Statement::DumpContext => {
					println!("{:#?}", self.variables);
				}
				Statement::Return(None) => return Ok(None),
				Statement::Return(Some(expr)) => {
					let val = self.resolve_expr(expr)?;
					return Ok(Some(val));
				}
			}
		}
		Ok(None)
	}
}
