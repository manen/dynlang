use interpret::Context;
use langlib::{Expr, Function, Reach, Value};

fn main() {
	let ctx = Context::new([("num".into(), Value::f32(2.2))]);

	let expr = Expr::Add(Reach::Named("num".into()), Reach::Value(Value::i32(4)));
	let out = ctx.resolve_expr(&expr).unwrap();

	println!("num + 4 = {out:?}");
}
