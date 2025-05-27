use std::{cell::RefCell, collections::HashMap, rc::Rc};

mod error;
pub use error::*;

use langlib::*;

#[derive(Clone, Debug, Default)]
pub struct Context {
	ctx: Vec<Rc<RefCell<ContextData>>>,
}

#[derive(Clone, Debug, Default)]
pub struct ContextData {
	variables: HashMap<String, Value>,
}
impl Context {
	pub fn new<I: IntoIterator<Item = (String, Value)>>(variables: I) -> Self {
		let variables = variables.into_iter();
		let data = ContextData {
			variables: variables.collect(),
		};
		Self {
			ctx: vec![Rc::new(RefCell::new(data))],
		}
	}

	pub fn get_variable(&self, name: &str) -> Result<Value> {
		for ctx in self.ctx.iter().rev() {
			let ctx = ctx.borrow();
			if let Some(here) = ctx.variables.get(name) {
				return Ok(here.clone());
			}
		}
		Err(Error::VariableDoesntExist(
			name.into(),
			VariableAccessType::Access,
		))
	}
	/// returns the previous value
	pub fn modify_variable(&self, name: &str, val: Value) -> Result<Value> {
		for ctx in self.ctx.iter().rev() {
			let mut ctx = ctx.borrow_mut();
			if let Some(_) = ctx.variables.get(name) {
				return ctx
					.variables
					.insert(name.into(), val)
					.ok_or(Error::Impossible1);
			}
		}
		Err(Error::VariableDoesntExist(
			name.into(),
			VariableAccessType::Modify,
		))
	}
	/// appends the new variable to the topmost context window, creating one if none exist
	pub fn set_variable(&mut self, name: String, val: Value) {
		if self.ctx.len() == 0 {
			let data = ContextData {
				variables: [(name, val)].into_iter().collect(),
			};
			self.ctx.push(Rc::new(RefCell::new(data)));
			return;
		}

		let a = self
			.ctx
			.iter_mut()
			.rev()
			.next()
			.expect("we JUST made sure at least one context window exists");
		let mut a = a.borrow_mut();
		a.variables.insert(name, val);
	}

	pub fn resolve_reach(&self, r: &Reach) -> Result<Value> {
		match r {
			Reach::Value(val) => Ok(val.clone()),
			Reach::Expr(expr) => self.resolve_expr(expr),
			Reach::Named(name) => self.get_variable(name),
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
			Expr::CallFn(f) => {
				let f = self.resolve_reach(f)?;
				match f {
					Value::Function(f) => self.clone().resolve_block(&f.block),
					_ => Err(Error::NotAFunction(f)),
				}
			}
		}
	}

	/// runs the given block as-is. does not isolate context at all so unless you wanna leak
	/// internal variables you should probably use `context.clone().resolve_block()`
	pub fn resolve_block(&mut self, block: &Block) -> Result<Value> {
		for stmt in block.iter() {
			match stmt {
				Statement::SetVariable(name, val) => {
					let val = self.resolve_expr(val)?;
					self.set_variable(name.clone(), val);
				}
				Statement::ModifyVariable(name, val) => {
					let val = self.resolve_expr(val)?;
					self.modify_variable(name, val)?;
				}
				Statement::DropExpr(expr) => {
					self.resolve_expr(expr)?;
				}
				Statement::Return(expr) => {
					let val = self.resolve_expr(expr)?;
					return Ok(val);
				}

				Statement::DumpContext => {
					println!("{:#?}", self);
				}
				Statement::Pause => {
					std::io::stdin().lines().next();
				}
			}
		}
		Ok(Value::None)
	}

	/// use for debugging only
	pub fn exec<I: IntoIterator<Item = Statement>>(&mut self, block: I) -> Result<Value> {
		self.resolve_block(&Block(block.into_iter().collect()))
	}
}
