use std::path::PathBuf;

/// An error that can occur when creating or cleaning a MongDB instance.
pub struct Error {
	/// The actual error.
	inner: ErrorInner,
}

#[derive(Debug)]
pub enum ErrorInner {
	/// Failed to create the temporary directory.
	MakeTempDir(std::io::Error),

	/// Failed to create the database directory.
	MakeDbDir(PathBuf, std::io::Error),

	/// Failed to spawn the server.
	SpawnServer(String, std::io::Error),

	/// Failed to kill the server.
	KillServer(std::io::Error),

	/// Failed to clean up the temporary directory.
	CleanDir(PathBuf, std::io::Error),

	/// Failed to connect to the server.
	Connect(String, mongodb::error::Error),

	Port,
}

impl std::error::Error for Error {}

impl std::fmt::Debug for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		std::fmt::Debug::fmt(&self.inner, f)
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		std::fmt::Display::fmt(&self.inner, f)
	}
}

impl std::fmt::Display for ErrorInner {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::MakeTempDir(e) => write!(f, "Failed to create temporary directory: {e}"),
			Self::MakeDbDir(path, e) => {
				write!(f, "Failed to create data directory {}: {e}", path.display())
			}
			Self::SpawnServer(name, e) => write!(f, "Failed to run server command: {name}: {e}"),
			Self::KillServer(e) => write!(f, "Failed to terminate spanwed server: {e}"),
			Self::CleanDir(path, e) => write!(
				f,
				"Failed to clean up temporary state directory {}: {e}",
				path.display()
			),
			Self::Connect(address, e) => write!(f, "Failed to connect to server at {address}: {e}"),
			Self::Port => write!(f, "Failed to select a free port by the os "),
		}
	}
}

impl From<ErrorInner> for Error {
	fn from(inner: ErrorInner) -> Self {
		Self { inner }
	}
}
