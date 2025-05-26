use interpret::Context;
use langlib::{Expr, Function, Reach, Statement, Value};

fn main() {
	let mut ctx = Context::new([("num".into(), Value::f32(2.2))]);

	let expr = Expr::Add(Reach::Named("num".into()), Reach::Value(Value::i32(4)));
	let out = ctx.resolve_expr(&expr).unwrap();

	println!("num + 4 = {out:?}");

	let b = Function::new([
		Statement::SetVariable(
			"hello".into(),
			Expr::Reach(Reach::Value(Value::String("hello".into()))),
		),
		Statement::DumpContext,
		Statement::Return(Some(Expr::Reach(Reach::Named("hello".into())))),
	])
	.block;

	let out = ctx.resolve_block(&b).unwrap();
	assert_eq!(out, Some(Value::String("hello".into())));
}
