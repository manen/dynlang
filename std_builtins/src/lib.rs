use langlib::{DynBuiltin, DynBuiltinBuilder, Value};

fn print_builtin(builder: &mut DynBuiltinBuilder) -> DynBuiltin {
	fn print(value: Value) -> Value {
		match value {
			Value::bool(a) => println!("{a}"),
			Value::i32(a) => println!("{a}"),
			Value::f32(a) => println!("{a}"),
			Value::String(a) => println!("{a}"),
			Value::Array(a) => println!("{a:?}"),
			Value::Object(a) => println!("{a:?}"),
			_ => println!("{value:?}"),
		}
		Value::None
	}
	builder.new("print", Some(print))
}

pub fn builtins() -> impl Iterator<Item = DynBuiltin> {
	let mut builder = DynBuiltinBuilder::default();
	[print_builtin(&mut builder)].into_iter()
}
