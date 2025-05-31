use std::{
	env, fs,
	io::{self, Write},
	path::PathBuf,
};

use anyhow::{Context as _, anyhow};
use basicparse::{Parser, ResultExt, preproc};
use interpret::{Context, IValue};
use langlib::{Block, Statement, Value};
use rustyline::DefaultEditor;

mod std_builtins;

fn main() -> anyhow::Result<()> {
	let mut args = env::args();
	args.next();

	let path = args.collect::<Vec<_>>().join(" ");
	let path: PathBuf = path.into();

	if path.exists() {
		let file = fs::read_to_string(&path).unwrap();
		let out = eval(&file);
		println!("{out:?}");
		Ok(())
	} else {
		let history_path = "./.history.txt";

		let mut rl = DefaultEditor::new()?;
		if rl.load_history(history_path).is_err() {
			eprintln!("no previous history")
		}

		eprintln!(
			"welcome to dynlang repl\nuse .import <file path> to import .dl files into the context\n"
		);

		let mut ctx = Context::default();
		ctx.builtins(std_builtins::builtins());
		let a = loop {
			let line = match rl.readline(" > ") {
				Ok(a) => a,
				Err(err) => break Err(err),
			};
			rl.add_history_entry(&line)?;

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
					Ok(a) => println!("{a}"),
					Err(err) => eprintln!("failed to execute: {err}"),
				},
				Err(err) => eprintln!("failed to parse: {err}"),
			}
		};
		rl.save_history(history_path)?;
		a?
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
	ctx.exec(parsed)
		.map_err(|err| anyhow!("{err}"))
		.with_context(|| "execution failed")
}
