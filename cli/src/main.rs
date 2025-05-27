use std::{
	env, fs,
	io::{self, Write},
	path::PathBuf,
};

use anyhow::Context as _;
use basicparse::{Parser, preproc};
use interpret::Context;
use langlib::{Block, Statement, Value};

fn main() {
	let mut args = env::args();
	args.next();

	let path = args.collect::<Vec<_>>().join(" ");
	let path: PathBuf = path.into();

	if path.exists() {
		let file = fs::read_to_string(&path).unwrap();
		let out = eval(&file);
		println!("{out:?}");
	} else {
		eprintln!("welcome to dynlang repl\n");

		let mut ctx = Context::default();
		loop {
			print!(" > ");
			io::stdout().flush().unwrap();
			let line = io::stdin().lines().next().unwrap().unwrap();

			match parse(&line) {
				Ok(parsed) => match ctx.resolve_block(&Block(parsed)) {
					Ok(a) => println!("{a:?}"),
					Err(err) => eprintln!("failed to execute: {err}"),
				},
				Err(err) => eprintln!("failed to parse: {err}"),
			}
		}
	}
}

fn parse(src: &str) -> anyhow::Result<Vec<Statement>> {
	let parser = Parser::new(src);
	let parser = preproc(parser.statements());

	let parsed = parser
		.collect::<Result<Vec<_>, _>>()
		.with_context(|| "parser failed")?;

	Ok(parsed)
}
fn eval(src: &str) -> anyhow::Result<Value> {
	let parsed = parse(src)?;

	interpret::Context::new([])
		.exec(parsed)
		.with_context(|| "execution failed")
}
