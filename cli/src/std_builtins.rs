use interpret::{DynBuiltin, DynBuiltinBuilder, IValue};
use langlib::Value;

fn print_builtin(builder: &mut DynBuiltinBuilder) -> DynBuiltin {
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
	builder.new("print", Some(print))
}

fn len_builtin(builder: &mut DynBuiltinBuilder) -> DynBuiltin {
	fn len(val: IValue) -> IValue {
		let len = interpret::utils::len(&val);
		if let Some(len) = len {
			IValue::i32(len as _)
		} else {
			IValue::None()
		}
	}
	builder.new("len", Some(len))
}

pub fn builtins() -> impl Iterator<Item = DynBuiltin> {
	let mut builder = DynBuiltinBuilder::default();
	[print_builtin(&mut builder), len_builtin(&mut builder)].into_iter()
}
