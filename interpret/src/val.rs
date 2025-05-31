use crate::*;
use langlib::*;

/// interpreter value
#[derive(Clone, Debug, PartialEq)]
pub enum IValue {
	Value(Value),

	Object(HashMap<String, IValue>),
	Array(Vec<IValue>),

	BuiltinFn(BuiltinFn),
	Closure(Closure),
}
impl IValue {
	pub fn from_safe(value: Value, ctx: &Context) -> Self {
		match value {
			Value::Array(a) => {
				IValue::Array(a.into_iter().map(|a| IValue::from_safe(a, ctx)).collect())
			}
			Value::Object(a) => IValue::Object(
				a.into_iter()
					.map(|(k, v)| (k, IValue::from_safe(v, ctx)))
					.collect(),
			),
			Value::Function(f) => {
				let cl = Closure {
					ctx: ctx.push_context(),
					f,
				};
				IValue::Closure(cl)
			}
			rest => IValue::Value(rest),
		}
	}

	pub fn bool(val: bool) -> IValue {
		IValue::Value(Value::bool(val))
	}
	pub fn i32(val: i32) -> IValue {
		IValue::Value(Value::i32(val))
	}
	pub fn f32(val: f32) -> IValue {
		IValue::Value(Value::f32(val))
	}
	#[allow(non_snake_case)]
	pub fn String(val: String) -> IValue {
		IValue::Value(Value::String(val))
	}
	#[allow(non_snake_case)]
	pub fn Function(val: Function) -> IValue {
		IValue::Value(Value::Function(val))
	}
	#[allow(non_snake_case)]
	pub fn Object(val: HashMap<String, Value>) -> IValue {
		IValue::Value(Value::Object(val))
	}
	#[allow(non_snake_case)]
	pub fn None() -> IValue {
		IValue::Value(Value::None)
	}

	pub fn is_true(&self) -> bool {
		match self {
			IValue::Value(v) => v.is_true(),
			_ => false,
		}
	}

	pub fn index(&self, i: &Index) -> Option<Self> {
		let base = match (self, i) {
			(IValue::Value(v), i) => v.index(i).map(IValue::Value),
			(IValue::Object(obj), i) => obj.get(&i.clone().into_str()).map(Clone::clone),
			(IValue::Array(a), Index::NumLit(i)) => a.iter().cloned().nth(*i as _),

			_ => None,
		};
		if let Some(base) = base {
			return Some(base);
		} else {
			match (self, i) {
				(val, Index::Ident(ident)) if ident == "len" => {
					let len = utils::len(val);
					len.map(|a| IValue::i32(a as _))
				}
				_ => None,
			}
		}
	}

	pub fn add(&self, rhs: &Self) -> Option<Self> {
		match (self, rhs) {
			(IValue::Value(a), IValue::Value(b)) => a.add(b).map(IValue::Value),
			_ => None,
		}
	}
	pub fn sub(&self, rhs: &Self) -> Option<Self> {
		match (self, rhs) {
			// this match statement contains every subtraction operation that's legal
			(IValue::Value(a), IValue::Value(b)) => a.sub(b).map(IValue::Value),
			_ => None,
		}
	}
	/// basically just PartialEq except it's lenient if it's the same but a different number type
	pub fn custom_eq(&self, rhs: &Self) -> bool {
		match (self, rhs) {
			(IValue::Value(a), IValue::Value(b)) => a.custom_eq(b),
			(a, b) => a == b,
		}
	}
	pub fn gt(&self, rhs: &Self) -> Option<Self> {
		match (self, rhs) {
			// this match statement contains every gt operation that's legal
			(IValue::Value(a), IValue::Value(b)) => a.gt(b).map(IValue::Value),
			_ => None,
		}
	}
	pub fn lt(&self, rhs: &Self) -> Option<Self> {
		match (self, rhs) {
			// this match statement contains every lt operation that's legal
			(IValue::Value(a), IValue::Value(b)) => a.lt(b).map(IValue::Value),
			_ => None,
		}
	}
}
impl Display for IValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			IValue::Value(value) => write!(f, "{value}"),
			IValue::Object(hash_map) => {
				write!(f, "obj {{")?;
				for (k, v) in hash_map {
					write!(f, " {k}: {v} ")?;
				}
				write!(f, "}}")
			}
			IValue::Array(ivalues) => {
				write!(f, "[")?;
				for val in ivalues {
					write!(f, " {val}")?;
				}
				write!(f, " ]")
			}
			IValue::BuiltinFn(dyn_builtin) => write!(f, "{dyn_builtin}"),
			IValue::Closure(closure) => write!(f, "{}", closure.f()),
		}
	}
}

use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq)]
/// see [BuiltinBuilder]
pub struct BuiltinFn {
	id: u64,
	name: Cow<'static, str>,

	f: fn(IValue) -> IValue,
}
impl BuiltinFn {
	pub fn id(&self) -> u64 {
		self.id
	}
	pub fn name(&self) -> &Cow<'static, str> {
		&self.name
	}
	pub fn f(&self) -> fn(IValue) -> IValue {
		self.f
	}
}
impl Display for BuiltinFn {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "[builtin {}]", self.name)
	}
}

// #[derive(Clone, Debug, PartialEq)]
// pub struct BuiltinObj {
// 	id: u64,
// 	name: Cow<'static, str>,

// 	index: fn(Index) -> IValue,
// }

#[derive(Default)]
pub struct BuiltinBuilder {
	id: u64,
}
impl BuiltinBuilder {
	pub fn new_fn(
		&mut self,
		name: impl Into<Cow<'static, str>>,
		f: fn(IValue) -> IValue,
	) -> BuiltinFn {
		let builtin = BuiltinFn {
			id: self.id,
			name: name.into(),
			f,
		};
		self.id += 1;
		builtin
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Closure {
	ctx: Context,
	f: Function,
}
impl Closure {
	pub fn ctx(&self) -> &Context {
		&self.ctx
	}
	pub fn f(&self) -> &Function {
		&self.f
	}

	pub fn call(&mut self, args: Option<IValue>) -> Result<IValue> {
		self.ctx.call_fn(&self.f, args)
	}
}
