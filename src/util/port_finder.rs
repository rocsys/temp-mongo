use std::net::TcpListener;

/// Represents a Port Generator to find available ports within a specified range.
pub struct PortGenerator {
	selected_port: Option<u16>,
}

impl PortGenerator {
	/// Constructs a new `PortGenerator`.
	///
	/// Initially, no port is selected (`selected_port` is `None`).
	/// The `generate` method must be called to select a port.
	///
	pub fn new() -> Self {
		PortGenerator {
			selected_port: None,
		}
	}

	/// Generates and selects an available port.
	///
	/// This method requests the operating system to provide an available ephemeral port
	/// by binding a `TcpListener` to port `0`. The OS assigns a free port, which is then
	/// retrieved and stored in `selected_port`.
	///
	/// # Returns
	///
	/// Returns a mutable reference to itself, allowing for method chaining.
	pub fn generate(&mut self) -> &mut Self {
		if let Ok(listener) = TcpListener::bind(("127.0.0.1", 0)) {
			if let Ok(addr) = listener.local_addr() {
				self.selected_port = Some(addr.port());
			}
		}
		self
	}

	/// Returns the selected port, if available.
	pub fn selected_port(&self) -> Option<u16> {
		self.selected_port
	}
}
