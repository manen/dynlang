use iter_read_until::{IntoReader, Reader};
use langlib::Value;

mod error;
pub use error::*;
use readuntil_ext::ReadExt;

mod tokens;
pub use tokens::*;

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

	pub fn parse_value(&mut self) -> Result<Value> {
		let word = self.reader.read_until_item(b' ').ok()?;
		todo!("{word}")
	}
}
