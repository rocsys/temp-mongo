use std::mem::ManuallyDrop;
use std::path::{Path, PathBuf};

/// Wrapper around [`tempfile::TempDir`], but with option to disable clean-on-drop behavior.
pub struct TempDir {
	/// The wrapped [`tempfile::TempDir`].
	inner: ManuallyDrop<tempfile::TempDir>,

	/// Determine to clean the temporary directory on drop.
	///
	/// If true, clean the temporary directory on drop.
	/// If false, leave the temporary directory on the filesystem.
	clean_on_drop: bool,
}

impl TempDir {
	/// Create a new temporary directory in the system tempdir
	pub fn new(clean_on_drop: bool) -> std::io::Result<Self> {
		Ok(Self {
			inner: ManuallyDrop::new(tempfile::tempdir()?),
			clean_on_drop,
		})
	}

	/// Create a new temporary directory in the given parent folder.
	pub fn new_in(parent: impl AsRef<Path>, clean_on_drop: bool) -> std::io::Result<Self> {
		Ok(Self {
			inner: ManuallyDrop::new(tempfile::tempdir_in(parent)?),
			clean_on_drop,
		})
	}

	/// Get the path of the temporary directory.
	pub fn path(&self) -> &Path {
		self.inner.path()
	}

	/// Enable or disable clean-on-drop behavior.
	pub fn set_clean_on_drop(&mut self, clean_on_drop: bool) {
		self.clean_on_drop = clean_on_drop;
	}

	/// Convert `self` into the wrapper [`tempfile::TempDir`].
	fn into_inner(mut self) -> tempfile::TempDir {
		let inner = unsafe { ManuallyDrop::take(&mut self.inner) };
		std::mem::forget(self);
		inner
	}

	/// Persist the temporary directory and return the path.
	///
	/// This ignore the value of `clean_on_drop`.
	/// The directory will not be cleaned up.
	pub fn into_path(self) -> PathBuf {
		self.into_inner().into_path()
	}

	/// Close the temporary directory, removing it from the filesystem unconditionally.
	///
	/// This ignore the value of `clean_on_drop`.
	/// The directory will be cleaned up immediately.
	pub fn close(self) -> std::io::Result<()> {
		self.into_inner().close()
	}
}

impl Drop for TempDir {
	fn drop(&mut self) {
		if self.clean_on_drop {
			unsafe { ManuallyDrop::drop(&mut self.inner) }
		}
	}
}
