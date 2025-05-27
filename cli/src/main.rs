use std::{
	env, fs,
	io::{self, Write},
	path::PathBuf,
};

use anyhow::Context;
use basicparse::{Parser, preproc};
use langlib::Value;

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
		eprintln!("welcome to dynlang repl\nyour code is ran fully isolated");

		loop {
			print!(" > ");
			io::stdout().flush().unwrap();
			let line = io::stdin().lines().next().unwrap().unwrap();
			match eval(&line) {
				Ok(a) => println!("{a:?}"),
				Err(err) => eprintln!("{err}"),
			}
		}
	}
}

fn eval(src: &str) -> anyhow::Result<Value> {
	let parser = Parser::new(src);
	let parser = preproc(parser.statements());

	let parsed = parser
		.collect::<Result<Vec<_>, _>>()
		.with_context(|| "parser failed")?;

	interpret::Context::new([])
		.exec(parsed)
		.with_context(|| "execution failed")
}
