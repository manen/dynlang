use basicparse::*;
use std::{
	env,
	io::{self, Write},
};

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

	let file = match std::fs::read_to_string(&rest) {
		Err(err) => {
			eprintln!("{err}");
			return None;
		}
		Ok(a) => a,
	};
	let tokenizer = Tokenizer::new(&file);

	let tokens = tokenizer
		.collect::<Result<Vec<_>, _>>()
		.expect("tokenizing failed");

	println!("{tokens:#?}");

	let parser = Parser::from_iter(tokens.into_iter().map(|a| {
		println!("{a:?}");
		Ok(a)
	}));

	let parsed = parser
		.statements()
		.collect::<Result<Vec<_>, _>>()
		.expect("failed to parse");

	println!("{parsed:#?}");

	Some(())
}

fn from_stdin() {
	let mode = env::args()
		.nth(1)
		.expect("expected mode for stdin parsing: tokenize or parse");

	let handle = |line: &str| {
		if mode == "tokenize" {
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
		} else if mode == "parse" {
			let mut parser = Parser::new(&line);
			print!("[ ");
			loop {
				match parser.read_statement() {
					Ok(a) => println!("{a:?}"),
					Err(Error::EOFStatement) => break,
					Err(err) => eprintln!("{err}"),
				}
			}
		} else {
			panic!("incorrect mode: expected tokenize or parse");
		}
	};

	loop {
		let mut line = String::new();

		io::stdout().flush().unwrap();
		io::stdin().read_line(&mut line).unwrap();
		println!();

		handle(&line);
		io::stdout().flush().unwrap();
	}
}
