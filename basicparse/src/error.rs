#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
	#[error("reader error while tokenizing:\n{0}")]
	Read(#[from] readuntil_ext::Error),
	#[error("{context}:\n{err}")]
	Context { context: String, err: Box<Self> },
}
impl Error {
	pub fn with_context(self, context: String) -> Self {
		Self::Context {
			context,
			err: Box::new(self),
		}
	}
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

pub trait ResultExt {
	type T;
	type E: Into<Error>;

	fn with_context(self, f: impl FnOnce() -> String) -> Result<Self::T>;
}
impl<T, E: Into<Error>> ResultExt for Result<T, E> {
	type T = T;
	type E = E;

	fn with_context(self, f: impl FnOnce() -> String) -> Result<Self::T> {
		self.map_err(Into::into)
			.map_err(|err| err.with_context(f()))
	}
}
