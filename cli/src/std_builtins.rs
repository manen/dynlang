use interpret::{BuiltinBuilder, BuiltinFn, Error, IValue, Result};
use langlib::Value;

fn print_noln(value: IValue) -> Result<IValue> {
	match value {
		IValue::Value(Value::bool(a)) => print!("{a}"),
		IValue::Value(Value::i32(a)) => print!("{a}"),
		IValue::Value(Value::f32(a)) => print!("{a}"),
		IValue::Value(Value::String(a)) => print!("{a}"),
		IValue::Value(Value::Array(a)) => print!("{a:?}"),
		IValue::Value(Value::Object(a)) => print!("{a:?}"),
		IValue::Value(a) => print!("{a:?}"),

		IValue::Object(a) => print!("{a:?}"),
		IValue::Array(a) => print!("{a:?}"),

		_ => print!("{value:?}"),
	}
	Ok(IValue::None())
}
fn print(value: IValue) -> Result<IValue> {
	let ret = print_noln(value);
	println!();
	ret
}

fn print_builtin(builder: &mut BuiltinBuilder) -> BuiltinFn {
	builder.new_fn("print", print)
}
fn print_noln_builtin(builder: &mut BuiltinBuilder) -> BuiltinFn {
	builder.new_fn("print_noln", print_noln)
}

fn len_builtin(builder: &mut BuiltinBuilder) -> BuiltinFn {
	fn len(val: IValue) -> Result<IValue> {
		let len = interpret::utils::len(&val);
		Ok(if let Some(len) = len {
			IValue::i32(len as _)
		} else {
			IValue::None()
		})
	}
	builder.new_fn("len", len)
}

fn to_string_builtin(builder: &mut BuiltinBuilder) -> BuiltinFn {
	fn to_string(val: IValue) -> Result<IValue> {
		Ok(IValue::String(format!("{val}")))
	}
	builder.new_fn("to_string", to_string)
}

fn obj_keys(builder: &mut BuiltinBuilder) -> BuiltinFn {
	fn obj_keys(val: IValue) -> Result<IValue> {
		macro_rules! implementation {
			($obj:expr) => {{ $obj.keys().cloned().map(IValue::String).collect() }};
		}
		let keys: Vec<_> = match val {
			IValue::Object(obj) => implementation!(obj),
			IValue::Value(Value::Object(obj)) => implementation!(obj),
			_ => {
				return Err(Error::Runtime(format!(
					"value ({val:?}) passed to builtin obj_keys fn isn't an object"
				)));
			}
		};
		Ok(IValue::Array(keys))
	}
	builder.new_fn("obj_keys", obj_keys)
}

fn obj_merge(builder: &mut BuiltinBuilder) -> BuiltinFn {
	fn obj_merge(val: IValue) -> Result<IValue> {
		let (a, b) = match val {
			IValue::Array(arr) if arr.len() == 2 => {
				let mut arr = arr.into_iter();
				(arr.next().unwrap(), arr.next().unwrap())
			}
			_ => {
				return Err(Error::Runtime(format!(
					"invalid argument to obj_merge: expected an array with two object elements"
				)));
			}
		};
		match (a, b) {
			(IValue::Object(a), IValue::Object(b)) => {
				Ok(IValue::Object(a.into_iter().chain(b).collect()))
			}
			_ => Err(Error::Runtime(format!(
				"invalid arguments to obj_merge: expected both arguments to be objects"
			))),
		}
	}
	builder.new_fn("obj_merge", obj_merge)
}

fn throw_error(builder: &mut BuiltinBuilder) -> BuiltinFn {
	fn throw_error(val: IValue) -> Result<IValue> {
		Err(Error::Runtime(format!("program threw error:\n{val}")))
	}
	builder.new_fn("throw_error", throw_error)
}

pub fn builtins() -> impl Iterator<Item = BuiltinFn> {
	let mut builder = BuiltinBuilder::default();
	builder
		.build_fns([
			print_builtin,
			print_noln_builtin,
			len_builtin,
			to_string_builtin,
			obj_keys,
			obj_merge,
			throw_error,
		])
		.into_iter()
}
