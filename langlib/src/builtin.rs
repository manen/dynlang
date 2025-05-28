use crate::*;

use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq)]
/// see [DynBuiltinBuilder]
pub struct DynBuiltin {
	id: u64,
	name: Cow<'static, str>,

	f: Option<fn(Value) -> Value>,
}
impl DynBuiltin {
	pub fn id(&self) -> u64 {
		self.id
	}
	pub fn name(&self) -> &Cow<'static, str> {
		&self.name
	}
	pub fn f(&self) -> Option<fn(Value) -> Value> {
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
		f: Option<fn(Value) -> Value>,
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
