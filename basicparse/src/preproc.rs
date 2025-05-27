use langlib::{Expr, Reach, Statement, Value};

pub fn preproc<E>(
	iter: impl IntoIterator<Item = Result<Statement, E>>,
) -> impl Iterator<Item = Result<Statement, E>> {
	let iter = iter.into_iter();
	iter.map(|a| match a {
		Ok(Statement::Expr(Expr::Reach(Reach::Value(Value::String(s))))) => {
			// inline unused strings get converted into hidden debug statements
			Ok(match s.as_ref() {
				"__pause" => Statement::Pause,
				"__dump_ctx" => Statement::DumpContext,
				_ => Statement::Expr(Expr::Reach(Reach::Value(Value::String(s)))),
			})
		}
		_ => a,
	})
}
