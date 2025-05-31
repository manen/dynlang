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

fn obj_keys(builder: &mut BuiltinBuilder) -> BuiltinFn {
	fn obj_keys(val: IValue) -> IValue {
		macro_rules! implementation {
			($obj:expr) => {{ $obj.keys().cloned().map(IValue::String).collect() }};
		}
		let keys: Vec<_> = match val {
			IValue::Object(obj) => implementation!(obj),
			IValue::Value(Value::Object(obj)) => implementation!(obj),
			_ => {
				return IValue::Value(Value::String(
					"value passed to builtin obj_keys fn isn't an object".into(),
				));
			}
		};
		IValue::Array(keys)
	}
	builder.new_fn("obj_keys", obj_keys)
}

fn obj_merge(builder: &mut BuiltinBuilder) -> BuiltinFn {
	fn obj_merge(val: IValue) -> IValue {
		let (a, b) = match val {
			IValue::Array(arr) if arr.len() == 2 => {
				let mut arr = arr.into_iter();
				(arr.next().unwrap(), arr.next().unwrap())
			}
			_ => {
				return IValue::String(format!(
					"invalid argument to obj_merge: expected an array with two object elements"
				));
			}
		};
		match (a, b) {
			(IValue::Object(a), IValue::Object(b)) => {
				IValue::Object(a.into_iter().chain(b).collect())
			}
			_ => IValue::String(format!(
				"invalid arguments to obj_merge: expected both arguments to be objects"
			)),
		}
	}
	builder.new_fn("obj_merge", obj_merge)
}

pub fn builtins() -> impl Iterator<Item = BuiltinFn> {
	let mut builder = BuiltinBuilder::default();
	builder
		.build_fns([
			print_builtin,
			len_builtin,
			to_string_builtin,
			obj_keys,
			obj_merge,
		])
		.into_iter()
}
