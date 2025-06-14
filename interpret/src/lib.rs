use std::{
	cell::RefCell,
	collections::HashMap,
	fmt::{Debug, Display},
	rc::Rc,
};

mod error;
pub use error::*;

mod val;
pub use val::*;

pub mod utils;

use langlib::*;

#[derive(Clone, Default, PartialEq)]
pub struct ContextData {
	variables: HashMap<String, IValue>,
}
#[derive(Clone, Default, PartialEq)]
pub struct Context {
	ctx: Vec<Rc<RefCell<ContextData>>>,
}

impl Context {
	pub fn new<V: Into<IValue>, I: IntoIterator<Item = (String, V)>>(variables: I) -> Self {
		let variables = variables.into_iter();
		let data = ContextData {
			variables: variables.map(|(k, v)| (k, v.into())).collect(),
		};
		Self {
			ctx: vec![Rc::new(RefCell::new(data))],
		}
	}
	pub fn builtins<I: IntoIterator<Item = BuiltinFn>>(&mut self, builtins: I) {
		// for builtin in builtins {
		// 	self.set_variable(builtin.name().to_owned().into(), Value::Builtin(builtin));
		// }
		let map = builtins
			.into_iter()
			.map(|b| (b.name().to_string(), IValue::BuiltinFn(b)))
			.collect();
		self.set_variable("builtins".into(), IValue::Object(map));
	}

	/// clones itself and appends a new context window to the list (making newly created variables automatically get placed in the new context window)
	pub fn push_window(&self) -> Self {
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
	pub fn for_variables(&self, mut f: impl FnMut(&String, &IValue)) {
		for ctx in &self.ctx {
			let ctx = ctx.borrow();
			for (name, val) in &ctx.variables {
				f(name, val)
			}
		}
	}
	pub fn get_variable(&self, name: &str) -> Result<IValue> {
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
	pub fn modify_variable(&self, name: &str, val: impl Into<IValue>) -> Result<IValue> {
		let val = val.into();
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
	pub fn set_variable(&mut self, name: String, val: impl Into<IValue>) {
		let val = val.into();
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

	pub fn resolve_reach(&self, r: &Reach) -> Result<IValue> {
		match r {
			Reach::Value(val) => Ok(IValue::from_safe(val.clone(), self)),
			Reach::Expr(expr) => self.resolve_expr(expr),
			Reach::Named(name) => self.get_variable(name),

			Reach::ArrayLiteral(arr) => {
				let mut values = Vec::with_capacity(arr.len());
				for expr in arr {
					let val = self.resolve_expr(expr)?;
					values.push(val);
				}
				Ok(IValue::Array(values))
			}
			Reach::ObjectLiteral(obj) => {
				let mut values = HashMap::with_capacity(obj.len());
				for (name, expr) in obj {
					values.insert(name.clone(), self.resolve_expr(expr)?);
				}
				Ok(IValue::Object(values))
			}
		}
	}
	pub fn resolve_index(&self, index: IntoIndex) -> Result<Index> {
		match index {
			IntoIndex::Index(i) => Ok(i),
			IntoIndex::Expr(expr) => {
				let val = self.resolve_expr(expr.as_ref())?;
				match val {
					IValue::Value(Value::i32(i)) => Ok(Index::NumLit(i)),
					IValue::Value(Value::String(s)) => Ok(Index::Ident(s)),
					_ => Err(Error::InvalidExprFromIntoIndex(val)),
				}
			}
		}
	}
	pub fn resolve_expr(&self, expr: &Expr) -> Result<IValue> {
		match expr {
			Expr::Reach(r) => self.resolve_reach(r),
			Expr::Block(block) => {
				let mut window = self.push_window();
				window.resolve_block(block)
			}
			Expr::Index(a, i) => {
				let a = self.resolve_reach(a)?;
				let i = self.resolve_index(i.clone())?;
				a.index(&i)
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

				Ok(IValue::from_safe(Value::bool(a.custom_eq(&b)), self))
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

				Ok(IValue::from_safe(
					Value::bool(a.is_true() || b.is_true()),
					self,
				))
			}
			Expr::And(a, b) => {
				let a = self.resolve_reach(a)?;
				let b = self.resolve_reach(b)?;

				Ok(IValue::from_safe(
					Value::bool(a.is_true() && b.is_true()),
					self,
				))
			}
			Expr::CallFn { f, args } => {
				let f = self.resolve_reach(f)?;
				match f {
					IValue::Value(Value::Function(f)) => {
						eprintln!("calling a function, not a closure");
						let args = args.clone().map(|reach| self.resolve_reach(&reach));
						let args = if let Some(args) = args {
							Some(args?)
						} else {
							None
						};

						self.call_fn(&f, args)
					}
					IValue::BuiltinFn(d) => {
						let f = d.f();
						let args = args.clone().map(|a| self.resolve_reach(&a));
						let args = if let Some(args) = args {
							args?
						} else {
							IValue::None()
						};

						f(args)
					}
					IValue::Closure(mut cl) => {
						let args = args.clone().map(|a| self.resolve_reach(&a));
						let args = if let Some(args) = args {
							Some(args?)
						} else {
							None
						};
						cl.call(args)
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
	/// internal variables you should probably use `context.clone().resolve_block()` \
	///
	/// if break is called it'll return Err(Error::Break) make sure to catch that
	pub fn resolve_block(&mut self, block: &Block) -> Result<IValue> {
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

				Statement::Loop(block) => loop {
					let out = self.resolve_block(block);
					match out {
						Err(Error::Break) => break,
						a => a?,
					};
				},
				Statement::Break => return Err(Error::Break),
				Statement::LoopFor {
					v_name,
					iter,
					block,
				} => {
					let v_name = v_name.clone();

					let mut ctx = self.push_window();
					ctx.set_variable(v_name.clone(), IValue::None());

					let iter = self.resolve_expr(iter)?;
					match iter {
						IValue::Object(obj) => {
							let next = obj
								.get("next")
								.cloned()
								.ok_or_else(|| Error::ForNotAnIterator(IValue::Object(obj)))?;

							let mut next = match next {
								IValue::Closure(cl) => cl,
								next => return Err(Error::ForNextIsntAClosure(next)),
							};

							loop {
								let next = next.call(None)?;
								match next {
									IValue::Value(Value::None) => break,
									val => {
										// set the variable with the name requested to the value generated by the next fn
										ctx.modify_variable(&v_name, val)?;
										ctx.resolve_block(block)?;
									}
								}
							}
						}
						IValue::Array(arr) => {
							for next in arr {
								// set the variable with the name requested to the value generated by the next fn
								ctx.modify_variable(&v_name, next)?;
								ctx.resolve_block(block)?;
							}
						}
						val => return Err(Error::ForNotAnObject(val)),
					};
				}

				Statement::DumpContext => {
					println!("{}", self);
				}
				Statement::Pause => {
					std::io::stdin().lines().next();
				}
			}
		}
		Ok(IValue::Value(Value::None))
	}

	/// safely calls the given function
	pub fn call_fn(&self, f: &Function, args: Option<IValue>) -> Result<IValue> {
		let mut ctx = self.push_window();
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
	pub fn exec<I: IntoIterator<Item = Statement>>(&mut self, block: I) -> Result<IValue> {
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
					write!(f, " {name}: {val},")?;
				}
			}
			write!(f, " ]")
		} else {
			writeln!(f)?;
			for ctx in &self.ctx {
				let ctx = ctx.borrow();
				for (name, val) in &ctx.variables {
					writeln!(f, "  {name}: {val}")?;
				}
			}
			write!(f, "]")
		}
	}
}
impl Debug for Context {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Display::fmt(self, f)
	}
}
