mod func;
pub use func::*;

#[derive(Clone, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Value {
	i32(i32),
	f32(f32),

	String(String),
	Array(Vec<Value>),

	Function(Function),
}
impl Value {
	pub fn add(&self, rhs: &Self) -> Option<Self> {
		match (self, rhs) {
			// this match statement contains every addition operation that's legal
			(Value::i32(a), Value::i32(b)) => Some(Self::i32(*a + *b)),
			(Value::f32(a), Value::f32(b)) => Some(Self::f32(*a + *b)),
			(Value::i32(a), Value::f32(b)) => Some(Self::f32(*a as f32 + *b)),
			(Value::f32(a), Value::i32(b)) => Some(Self::f32(*a + *b as f32)),
			(Value::String(a), Value::String(b)) => Some(Self::String(format!("{a}{b}"))),

			(Value::Array(a), Value::Array(b)) => Some(Self::Array(
				a.iter().cloned().chain(b.iter().cloned()).collect(), // a then b
			)),
			(Value::Array(a), new) => Some(Self::Array(
				a.iter()
					.cloned()
					.chain(std::iter::once(new.clone()))
					.collect(), // entire array and then the new element
			)),
			(new, Value::Array(a)) => Some(Self::Array(
				std::iter::once(new.clone())
					.chain(a.iter().cloned())
					.collect(), // new element then the entire array
			)),

			_ => None,
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
	Add(Reach, Reach),
}

#[derive(Clone, Debug, PartialEq)]
/// represents how we reach a variable
pub enum Reach {
	Expr(Box<Expr>),
	Named(String),
}
