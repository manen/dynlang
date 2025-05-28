use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

mod error;
pub use error::*;

use langlib::*;

#[derive(Clone, Debug, Default)]
pub struct ContextData {
	variables: HashMap<String, Value>,
}
#[derive(Clone, Debug, Default)]
pub struct Context {
	ctx: Vec<Rc<RefCell<ContextData>>>,
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
	pub fn builtins<I: IntoIterator<Item = DynBuiltin>>(&mut self, builtins: I) {
		// for builtin in builtins {
		// 	self.set_variable(builtin.name().to_owned().into(), Value::Builtin(builtin));
		// }
		let map = builtins
			.into_iter()
			.map(|b| (b.name().to_string(), Value::Builtin(b)))
			.collect();
		self.set_variable("builtins".into(), Value::Object(map));
	}

	/// clones itself and appends a new context window to the list (making newly created variables automatically get placed in the new context window)
	pub fn push_context(&self) -> Self {
		Self {
			ctx: self
				.ctx
				.iter()
				.cloned()
				.chain(std::iter::once(Default::default()))
				.collect(),
		}
	}

	pub fn variables_len(&self) -> usize {
		self.ctx.iter().map(|a| a.borrow().variables.len()).sum()
	}
	/// iterate over every variable available
	pub fn for_variables(&self, mut f: impl FnMut(&String, &Value)) {
		for ctx in &self.ctx {
			let ctx = ctx.borrow();
			for (name, val) in &ctx.variables {
				f(name, val)
			}
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

			Reach::ArrayLiteral(arr) => {
				let mut values = Vec::with_capacity(arr.len());
				for expr in arr {
					let val = self.resolve_expr(expr)?;
					values.push(val);
				}
				Ok(Value::Array(values))
			}
			Reach::ObjectLiteral(obj) => {
				let mut values = HashMap::with_capacity(obj.len());
				for (name, expr) in obj {
					values.insert(name.clone(), self.resolve_expr(expr)?);
				}
				Ok(Value::Object(values))
			}
		}
	}
	pub fn resolve_expr(&self, expr: &Expr) -> Result<Value> {
		match expr {
			Expr::Reach(r) => self.resolve_reach(r),
			Expr::Block(block) => {
				let mut window = self.push_context();
				window.resolve_block(block)
			}
			Expr::Index(a, i) => {
				let a = self.resolve_reach(a)?;
				a.index(i)
					.ok_or_else(|| Error::InvalidIndex { a, i: i.clone() })
			}
			Expr::Add(a, b) => {
				let a = self.resolve_reach(a)?;
				let b = self.resolve_reach(b)?;

				a.add(&b).ok_or_else(|| Error::InvalidAddition { a, b })
			}
			Expr::Sub(a, b) => {
				let a = self.resolve_reach(a)?;
				let b = self.resolve_reach(b)?;

				a.sub(&b).ok_or_else(|| Error::InvalidSubtraction { a, b })
			}
			Expr::Cmp(a, b) => {
				let a = self.resolve_reach(a)?;
				let b = self.resolve_reach(b)?;

				Ok(Value::bool(a.custom_eq(&b)))
			}
			Expr::Gt(a, b) => {
				let a = self.resolve_reach(a)?;
				let b = self.resolve_reach(b)?;

				a.gt(&b).ok_or_else(|| Error::InvalidGt { a, b })
			}
			Expr::Lt(a, b) => {
				let a = self.resolve_reach(a)?;
				let b = self.resolve_reach(b)?;

				a.lt(&b).ok_or_else(|| Error::InvalidLt { a, b })
			}
			Expr::Or(a, b) => {
				let a = self.resolve_reach(a)?;
				let b = self.resolve_reach(b)?;

				Ok(Value::bool(a.is_true() || b.is_true()))
			}
			Expr::And(a, b) => {
				let a = self.resolve_reach(a)?;
				let b = self.resolve_reach(b)?;

				Ok(Value::bool(a.is_true() && b.is_true()))
			}
			Expr::CallFn { f, args } => {
				let f = self.resolve_reach(f)?;
				match f {
					Value::Function(f) => {
						let args = args.clone().map(|reach| self.resolve_reach(&reach));
						let args = if let Some(args) = args {
							Some(args?)
						} else {
							None
						};

						self.call_fn(&f, args)
					}
					Value::Builtin(d) if d.f().is_some() => {
						let f = d.f().expect("we just made sure it's some");
						let args = args.clone().map(|a| self.resolve_reach(&a));
						let args = if let Some(args) = args {
							args?
						} else {
							Value::None
						};

						Ok(f(args))
					}
					_ => Err(Error::NotAFunction(f)),
				}
			}
			Expr::Conditional {
				condition,
				if_true,
				if_false,
			} => {
				let condition = self.resolve_reach(condition)?;
				if condition.is_true() {
					let if_true = self.resolve_reach(if_true)?;
					Ok(if_true)
				} else {
					let if_false = self.resolve_reach(if_false)?;
					Ok(if_false)
				}
			}
		}
	}

	/// runs the given block as-is. does not isolate context at all so unless you wanna leak
	/// internal variables you should probably use `context.clone().resolve_block()`
	pub fn resolve_block(&mut self, block: &Block) -> Result<Value> {
		let len = block.0.len();
		for (i, stmt) in block.iter().enumerate() {
			let last = i == len - 1;
			match stmt {
				Statement::SetVariable(name, val) => {
					let val = self.resolve_expr(val)?;
					self.set_variable(name.clone(), val);
				}
				Statement::ModifyVariable(name, val) => {
					let val = self.resolve_expr(val)?;
					self.modify_variable(name, val)?;
				}
				Statement::Expr(expr) => {
					let val = self.resolve_expr(expr)?;
					if last {
						return Ok(val);
					}
				}
				Statement::Return(expr) => {
					let val = self.resolve_expr(expr)?;
					return Ok(val);
				}

				Statement::DumpContext => {
					println!("{}", self);
				}
				Statement::Pause => {
					std::io::stdin().lines().next();
				}
			}
		}
		Ok(Value::None)
	}
	/// safely calls the given function
	pub fn call_fn(&self, f: &Function, args: Option<Value>) -> Result<Value> {
		let mut ctx = self.push_context();
		match (&f.arg_name, args) {
			(Some(name), Some(val)) => {
				ctx.set_variable(name.clone(), val);
			}
			(Some(name), None) => return Err(Error::MissingArg(name.clone())),
			_ => {}
		}

		ctx.resolve_block(&f.block)
	}

	/// use for debugging only
	pub fn exec<I: IntoIterator<Item = Statement>>(&mut self, block: I) -> Result<Value> {
		self.resolve_block(&Block(block.into_iter().collect()))
	}
}

impl Display for Context {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Context [")?;
		let small = self.variables_len() < 3;
		if small {
			for ctx in &self.ctx {
				let ctx = ctx.borrow();
				for (name, val) in &ctx.variables {
					write!(f, " {name}: {val:?},")?;
				}
			}
			write!(f, " ]")
		} else {
			writeln!(f)?;
			for ctx in &self.ctx {
				let ctx = ctx.borrow();
				for (name, val) in &ctx.variables {
					writeln!(f, "  {name}: {val:?}")?;
				}
			}
			write!(f, "]")
		}
	}
}
