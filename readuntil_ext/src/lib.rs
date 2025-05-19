use iter_read_until::Read;

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
	#[error("expected {expected:?}, got {got:?}")]
	ExpectedGot {
		expected: Read<()>,
		got: Read<String>,
	},
	#[error("expected ok or end, got {0:?}")]
	ExpectedOkOrEndGot(Read<String>),
}
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub trait ReadExt {
	type T;

	/// errors if read isn't [Read::Finished], meaning the reader
	/// was already exhausted before this read was issued
	fn finished(self) -> Result<()>;

	/// started to read successfully but read until the end of the
	/// text and didn't find the item we were searching for
	fn end(self) -> Result<Self::T>;
	/// started to read and found the item we were looking for
	fn ok(self) -> Result<Self::T>;

	fn ok_or_end(self) -> Result<Self::T>;
}

impl<'a> ReadExt for Read<&'a str> {
	type T = &'a str;

	fn ok(self) -> Result<Self::T> {
		match self {
			Read::Condition(a) => Ok(a),
			read => Err(Error::ExpectedGot {
				expected: Read::Condition(()),
				got: collect(read),
			}),
		}
	}
	fn end(self) -> Result<Self::T> {
		match self {
			Read::End(a) => Ok(a),
			read => Err(Error::ExpectedGot {
				expected: Read::End(()),
				got: collect(read),
			}),
		}
	}
	fn finished(self) -> Result<()> {
		match self {
			Self::Finished => Ok(()),
			read => Err(Error::ExpectedGot {
				expected: Read::Finished,
				got: collect(read),
			}),
		}
	}

	fn ok_or_end(self) -> Result<Self::T> {
		match self {
			Self::End(a) => Ok(a),
			Self::Condition(a) => Ok(a),
			Self::Finished => Err(Error::ExpectedOkOrEndGot(Read::Finished)),
		}
	}
}

fn collect<T: ToString>(i: Read<T>) -> Read<String> {
	match i {
		Read::Condition(a) => Read::Condition(a.to_string()),
		Read::End(a) => Read::End(a.to_string()),
		Read::Finished => Read::Finished,
	}
}
