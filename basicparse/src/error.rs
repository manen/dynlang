#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
	#[error("yoo")]
	Read(#[from] readuntil_ext::Error),
}
pub type Result<T, E = Error> = core::result::Result<T, E>;
