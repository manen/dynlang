use crate::*;

use iter_read_until::{IntoReader, StrReader};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Signal {
	Word,

	Eq,
	Plus,
	StrStart,
	ParensStart,
	CurlyStart,
	BracketStart,
}
fn token_letters(c: u8) -> Option<Signal> {
	match c {
		b'=' => Some(Signal::Eq),
		b'+' => Some(Signal::Plus),
		b'"' => Some(Signal::StrStart),
		b'(' => Some(Signal::ParensStart),
		b'{' => Some(Signal::CurlyStart),
		b'[' => Some(Signal::BracketStart),
		b' ' => Some(Signal::Word),
		_ => None,
	}
}

#[derive(Clone, Debug)]
pub enum Token {
	/// `let`
	Let,
	/// `=`
	Eq,
	/// `fn`
	Fn,
	/// `+`
	Plus,

	Ident(String),
	/// number literal
	NumLit(String),
	/// string literal,
	StrLit(String),

	/// everything between `(` and `)`, tokenized
	Parens(Vec<Token>),
	/// everything between `{` and `}`, tokenized
	Curly(Vec<Token>),
	/// everything between `[` and `]`, tokenized
	Brackets(Vec<Token>),
}

#[derive(Copy, Clone, Debug)]
pub struct Tokenizer<'a> {
	reader: StrReader<'a>,
	/// we use signal to store any additional information we need to take care of before reading from the reader
	signal: Option<Signal>,
}
impl<'a> Tokenizer<'a> {
	pub fn new(src: &'a str) -> Self {
		Tokenizer {
			reader: src.reader(),
			signal: None,
		}
	}

	pub fn next_token(&mut self) -> Result<Token> {
		if let Some(signal) = self.signal.take() {
			/// shared macro for all bracket types
			macro_rules! encapsulating {
				($op:expr, $cl:expr => $name:ident) => {{
					let mut i = 1;
					let s = self
						.reader
						.read_until(|c| match *c {
							$op => {
								i += 1;
								false
							}
							$cl => {
								i -= 1;
								if i <= 0 { true } else { false }
							}
							_ => false,
						})
						.ok()
						.with_context(|| {
							format!(
								"couldn't find matching closing parens for {} ('{}')",
								stringify!($name),
								$cl
							)
						})?;
					let tokenizer = Tokenizer::new(s);
					let tokens = tokenizer.collect::<Result<Vec<_>, _>>()?;
					Token::Parens(tokens)
				}};
			}

			return Ok(match signal {
				Signal::Word => return self.next_token(),
				Signal::Eq => Token::Eq,
				Signal::Plus => Token::Plus,
				Signal::StrStart => {
					let s = self
						.reader
						.read_until_item(b'"')
						.ok()
						.with_context(|| "you didn't close a string literal".into())?;
					Token::StrLit(s.into())
				}
				Signal::ParensStart => {
					encapsulating!(b'(', b')' => Parens)
				}
				Signal::CurlyStart => {
					encapsulating!(b'{', b'}' => Curly)
				}
				Signal::BracketStart => {
					encapsulating!(b'[', b']' => Brackets)
				}
			});
		}

		let word = self
			.reader
			.read_until(|c| match token_letters(*c) {
				Some(Signal::Word) => true,
				Some(sig) => {
					self.signal = Some(sig);
					true
				}
				_ => false,
			})
			.ok_or_end()?;
		if word.len() == 0 {
			return Err(Error::TokenizerFinished);
		}

		match word.trim() {
			"let" => Ok(Token::Let),
			"fn" => Ok(Token::Fn),
			ident => Ok(Token::Ident(ident.into())),
		}
	}
}
impl<'a> Iterator for Tokenizer<'a> {
	type Item = Result<Token>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.next_token() {
			Ok(a) => Some(Ok(a)),
			Err(Error::Read(readuntil_ext::Error::ExpectedOkOrEndGot(
				iter_read_until::Read::Finished,
			))) => None,
			Err(Error::Read(readuntil_ext::Error::ExpectedGot {
				expected: _,
				got: iter_read_until::Read::Finished,
			})) => None,
			Err(Error::TokenizerFinished) => None,
			Err(err) => Some(Err(err)),
		}
	}
}
