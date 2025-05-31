use std::{
	env, fs,
	io::{self, Write},
	path::PathBuf,
};

use anyhow::Context as _;
use basicparse::{Parser, preproc};
use interpret::{Context, IValue};
use langlib::{Block, Statement, Value};

mod std_builtins;

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
		eprintln!(
			"welcome to dynlang repl\nuse .import <file path> to import .dl files into the context\n"
		);

		let mut ctx = Context::default();
		ctx.builtins(std_builtins::builtins());
		loop {
			print!(" > ");
			io::stdout().flush().unwrap();
			let line = io::stdin().lines().next().unwrap().unwrap();

			let line = if line.starts_with(".import ") {
				let path = line.replace(".import ", "");
				let file = match std::fs::read_to_string(&path) {
					Ok(a) => a,
					Err(err) => {
						eprintln!("failed to read file: {err}");
						continue;
					}
				};
				file
			} else {
				line
			};

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

fn parse(src: &str) -> basicparse::Result<Vec<Statement>> {
	let parser = Parser::new(src);
	let parser = preproc(parser.statements());

	let parsed = parser.collect::<Result<Vec<_>, _>>()?;

	Ok(parsed)
}
fn eval(src: &str) -> anyhow::Result<IValue> {
	let parsed = parse(src)?;

	let mut ctx = interpret::Context::new::<IValue, _>([]);
	ctx.builtins(std_builtins::builtins());
	ctx.exec(parsed).with_context(|| "execution failed")
}
