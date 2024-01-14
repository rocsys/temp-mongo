use std::process::Child;

/// Simple wrapper around [`std::process::Child`] that kills the process when dropped.
pub struct KillOnDrop {
    /// The wrapped child process.
    child: Child,
}

impl KillOnDrop {
    /// Wrap an existing [`std::process:Child`] object.
    pub fn new(child: Child) -> Self {
        Self { child }
    }

    /// Get the PID of the child process.
    pub fn id(&self) -> u32 {
        self.child.id()
    }

    /// Kill the child process.
    pub fn kill(&mut self) -> std::io::Result<()> {
        self.child.kill()
    }
}

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        self.child.kill().ok();
        self.child.wait().ok();
    }
}
