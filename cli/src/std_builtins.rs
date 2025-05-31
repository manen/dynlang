use interpret::{BuiltinBuilder, BuiltinFn, IValue};
use langlib::Value;

fn print_builtin(builder: &mut BuiltinBuilder) -> BuiltinFn {
	fn print(value: IValue) -> IValue {
		match value {
			IValue::Value(Value::bool(a)) => println!("{a}"),
			IValue::Value(Value::i32(a)) => println!("{a}"),
			IValue::Value(Value::f32(a)) => println!("{a}"),
			IValue::Value(Value::String(a)) => println!("{a}"),
			IValue::Value(Value::Array(a)) => println!("{a:?}"),
			IValue::Value(Value::Object(a)) => println!("{a:?}"),
			IValue::Value(a) => println!("{a:?}"),

			IValue::Object(a) => println!("{a:?}"),
			IValue::Array(a) => println!("{a:?}"),

			_ => println!("{value:?}"),
		}
		IValue::None()
	}
	builder.new_fn("print", print)
}

fn len_builtin(builder: &mut BuiltinBuilder) -> BuiltinFn {
	fn len(val: IValue) -> IValue {
		let len = interpret::utils::len(&val);
		if let Some(len) = len {
			IValue::i32(len as _)
		} else {
			IValue::None()
		}
	}
	builder.new_fn("len", len)
}

fn to_string_builtin(builder: &mut BuiltinBuilder) -> BuiltinFn {
	fn to_string(val: IValue) -> IValue {
		IValue::String(format!("{val}"))
	}
	builder.new_fn("to_string", to_string)
}

pub fn builtins() -> impl Iterator<Item = BuiltinFn> {
	let mut builder = BuiltinBuilder::default();
	[
		print_builtin(&mut builder),
		len_builtin(&mut builder),
		to_string_builtin(&mut builder),
	]
	.into_iter()
}
