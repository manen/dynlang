use crate::*;

pub fn len(val: &IValue) -> Option<usize> {
	let len = match val {
		IValue::Array(arr) => arr.len(),
		IValue::Value(Value::Array(arr)) => arr.len(),
		IValue::Value(Value::String(s)) => s.len(),
		_ => return None,
	};
	Some(len)
}
