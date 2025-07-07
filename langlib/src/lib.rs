mod func;
use std::{collections::HashMap, fmt::Display};

pub use func::*;

#[derive(Clone, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Value {
	bool(bool),
	i32(i32),
	f32(f32),

	String(String),
	Array(Vec<Value>),
	Function(Function),
	Object(HashMap<String, Value>),

	None,
}
impl Value {
	pub fn is_true(&self) -> bool {
		match self {
			Value::bool(true) => true,
			&Value::i32(n) if n != 0 => true,
			_ => false,
		}
	}

	pub fn index(&self, i: &Index) -> Option<Self> {
		match (self, i) {
			(Value::Object(obj), i) => {
				let i = i.clone().into_str();
				obj.get(&i).cloned()
			}
			(Value::Array(arr), Index::NumLit(i)) => {
				Some(arr.iter().nth(*i as _).cloned().unwrap_or(Value::None))
			}
			_ => None,
		}
	}

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

			(a, Value::None) | (Value::None, a) => Some(a.clone()),

			_ => None,
		}
	}
	pub fn sub(&self, rhs: &Self) -> Option<Self> {
		match (self, rhs) {
			// this match statement contains every subtraction operation that's legal
			(Value::i32(a), Value::i32(b)) => Some(Self::i32(*a - *b)),
			(Value::f32(a), Value::f32(b)) => Some(Self::f32(*a - *b)),
			(Value::i32(a), Value::f32(b)) => Some(Self::f32(*a as f32 - *b)),
			(Value::f32(a), Value::i32(b)) => Some(Self::f32(*a - *b as f32)),

			(Value::None, a) => Value::i32(0).sub(a),
			(a, Value::None) => Some(a.clone()),

			_ => None,
		}
	}
	/// basically just PartialEq except it's lenient if it's the same but a different number type
	pub fn custom_eq(&self, rhs: &Self) -> bool {
		match (self, rhs) {
			(Self::i32(a), Self::f32(b)) | (Self::f32(b), Self::i32(a)) => *a as f32 == *b,
			(a, b) => a == b,
		}
	}
	pub fn gt(&self, rhs: &Self) -> Option<Self> {
		match (self, rhs) {
			// this match statement contains every gt operation that's legal
			(Value::i32(a), Value::i32(b)) => Some(Self::bool(*a > *b)),
			(Value::f32(a), Value::f32(b)) => Some(Self::bool(*a > *b)),
			(Value::i32(a), Value::f32(b)) => Some(Self::bool(*a as f32 > *b)),
			(Value::f32(a), Value::i32(b)) => Some(Self::bool(*a > *b as f32)),

			(Value::None, Value::None) => Some(Self::bool(false)),
			(Value::None, _) => Some(Self::bool(false)),
			(_, Value::None) => Some(Self::bool(true)),

			_ => None,
		}
	}
	pub fn lt(&self, rhs: &Self) -> Option<Self> {
		match (self, rhs) {
			// this match statement contains every lt operation that's legal
			(Value::i32(a), Value::i32(b)) => Some(Self::bool(*a < *b)),
			(Value::f32(a), Value::f32(b)) => Some(Self::bool(*a < *b)),
			(Value::i32(a), Value::f32(b)) => Some(Self::bool((*a as f32) < (*b))),
			(Value::f32(a), Value::i32(b)) => Some(Self::bool(*a < *b as f32)),

			(Value::None, Value::None) => Some(Self::bool(false)),
			(Value::None, _) => Some(Self::bool(true)),
			(_, Value::None) => Some(Self::bool(false)),

			_ => None,
		}
	}
}
impl Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Value::bool(b) => write!(f, "{b}"),
			Value::i32(a) => write!(f, "{a}"),
			Value::f32(a) => write!(f, "{a}"),
			Value::String(a) => write!(f, "{a:?}"),
			Value::Object(hash_map) => {
				write!(f, "obj {{")?;
				for (k, v) in hash_map {
					write!(f, "  {k}: {v}")?;
				}
				write!(f, "}}")
			}
			Value::Array(ivalues) => {
				write!(f, "[")?;
				for val in ivalues {
					write!(f, " {val}")?;
				}
				write!(f, " ]")
			}
			Value::Function(func) => write!(f, "{func}"),
			Value::None => write!(f, "None"),
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum Index {
	Ident(String),
	NumLit(i32),
}
impl Index {
	pub fn into_str(self) -> String {
		match self {
			Index::Ident(name) => name,
			Index::NumLit(num) => format!("{num}"),
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum IntoIndex {
	Index(Index),
	Expr(Box<Expr>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
	Reach(Reach),
	Block(Block),

	/// index something
	/// so this is used for reaching into maps (like person.name)
	/// and arrays too (like array.0)
	Index(Reach, IntoIndex),

	// these return bools
	Cmp(Reach, Reach),
	// true if the first is larger
	Gt(Reach, Reach),
	// true if the second is larger
	Lt(Reach, Reach),

	// a or b
	Or(Reach, Reach),
	// a and b
	And(Reach, Reach),

	Conditional {
		condition: Reach,
		if_true: Reach,
		if_false: Reach,
	},

	// these return one of the types they are passed
	Add(Reach, Reach),
	Sub(Reach, Reach),
	/// calls the given function. no args for now
	CallFn {
		f: Reach,
		args: Option<Reach>,
	},
}
impl Expr {
	pub fn into_reach(self) -> Reach {
		match self {
			Self::Reach(r) => r,
			e => Reach::Expr(Box::new(e)),
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
/// represents how we reach a variable
pub enum Reach {
	ArrayLiteral(Vec<Expr>),
	ObjectLiteral(Vec<(String, Expr)>),

	Value(Value),
	Expr(Box<Expr>),
	Named(String),
}
impl Reach {
	pub fn into_expr(self) -> Expr {
		match self {
			Self::Expr(expr) => *expr,
			reach => Expr::Reach(reach),
		}
	}
}
