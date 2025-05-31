use crate::*;
use langlib::*;

/// interpreter value
#[derive(Clone, Debug, PartialEq)]
pub enum IValue {
	Value(Value),

	Object(HashMap<String, IValue>),
	Array(Vec<IValue>),

	Builtin(DynBuiltin),
}
impl From<Value> for IValue {
	fn from(value: Value) -> Self {
		IValue::Value(value)
	}
}
impl IValue {
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
		match (self, i) {
			(IValue::Value(v), i) => v.index(i).map(IValue::Value),
			(IValue::Object(obj), i) => obj.get(&i.clone().into_str()).map(Clone::clone),
			(IValue::Array(a), Index::NumLit(i)) => a.iter().cloned().nth(*i as _),
			_ => None,
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

use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq)]
/// see [DynBuiltinBuilder]
pub struct DynBuiltin {
	id: u64,
	name: Cow<'static, str>,

	f: Option<fn(IValue) -> IValue>,
}
impl DynBuiltin {
	pub fn id(&self) -> u64 {
		self.id
	}
	pub fn name(&self) -> &Cow<'static, str> {
		&self.name
	}
	pub fn f(&self) -> Option<fn(IValue) -> IValue> {
		self.f
	}
}

#[derive(Default)]
pub struct DynBuiltinBuilder {
	id: u64,
}
impl DynBuiltinBuilder {
	pub fn new(
		&mut self,
		name: impl Into<Cow<'static, str>>,
		f: Option<fn(IValue) -> IValue>,
	) -> DynBuiltin {
		let builtin = DynBuiltin {
			id: self.id,
			name: name.into(),
			f,
		};
		self.id += 1;
		builtin
	}
}
