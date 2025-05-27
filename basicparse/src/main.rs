use std::{
	env,
	io::{self, Write},
};

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
	let rest = rest.trim();
	if rest == "" {
		return None;
	}

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

		io::stdout().flush().unwrap();
		io::stdin().read_line(&mut line).unwrap();
		println!("line read");

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
		io::stdout().flush().unwrap();
	}
}
