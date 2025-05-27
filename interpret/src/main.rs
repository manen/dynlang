use interpret::Context;
use langlib::{Expr, Function, Reach, Statement, Value};

fn main() {
	let out = Context::new([]).exec(
		[
			Statement::SetVariable(
				"num".into(),
				Expr::Add(Reach::Value(Value::i32(2)), Reach::Value(Value::f32(4.2))),
			),
			Statement::SetVariable(
				"grow".into(),
				Expr::Reach(Reach::Value(Value::Function(Function::new([
					Statement::ModifyVariable(
						"num".into(),
						Expr::Add(Reach::Named("num".into()), Reach::Value(Value::i32(2))),
					),
					Statement::DumpContext,
				])))),
			),
		]
		.into_iter()
		.chain(
			std::iter::repeat_n(
				[
					Statement::Pause,
					Statement::Expr(Expr::CallFn(Reach::Named("grow".into()))),
				],
				4,
			)
			.flatten(),
		),
	);
	println!("out: {out:?}");
}
