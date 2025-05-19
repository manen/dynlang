use iter_read_until::{IntoReader, Reader};
use langlib::Expr;

mod error;
pub use error::*;
use readuntil_ext::ReadExt;

#[derive(Clone, Debug)]
pub struct Parser<'a> {
	reader: iter_read_until::StrReader<'a>,
}
impl<'a> Parser<'a> {
	pub fn new(src: &'a str) -> Self {
		Self {
			reader: src.reader(),
		}
	}

	pub fn parse_value(&mut self) -> Result<Expr> {
		let word = self.reader.read_until_item(b' ').ok()?;
		todo!("{word}")
	}
}
