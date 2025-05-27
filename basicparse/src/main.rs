use std::{env, io};

use basicparse::Tokenizer;

fn main() {
	if from_args().is_some() {
		return;
	}
	from_stdin();
}

fn from_args() -> Option<()> {
	let mut args = env::args();
	let _arg0 = args.next()?;
	let rest = args.collect::<Vec<_>>().join(" ");

	let file = std::fs::read_to_string(&rest).expect("failed to open file");
	let tokenizer = Tokenizer::new(&file);

	let tokens = tokenizer
		.collect::<Result<Vec<_>, _>>()
		.expect("tokenizing failed");

	println!("{tokens:#?}");
	Some(())
}

fn from_stdin() {
	loop {
		let mut line = String::new();
		io::stdin().read_line(&mut line).unwrap();

		let tokenizer = Tokenizer::new(&line);
		print!("[ ");
		for token in tokenizer {
			match token {
				Ok(token) => print!("{token:?} "),
				Err(err) => {
					println!("\n{err}\n")
				}
			}
		}
		print!("]");
	}
}
