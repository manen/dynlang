use interpret::Context;
use langlib::{Expr, Function, Reach, Statement, Value};

fn main() {
	let out = Context::new([]).exec([]);
	println!("out: {out:?}");
}
