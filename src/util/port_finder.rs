use std::net::TcpListener;

/// `PortGenerator` is a utility class for generating an available network port.
///
/// This class is particularly useful for scenarios where an application needs to
/// bind to an available port without specifying a particular one. It leverages the
/// operating system's ability to select an ephemeral port, ensuring that the chosen
/// port is free at the time of selection.
///
/// The main use case of this class is in situations where a service or a server
/// instance (like a database server in testing environments) needs to be started
/// dynamically on a free port to avoid conflicts or for parallel testing.

pub struct PortGenerator {
    /// The port number selected by the operating system.
    /// This is `None` until `generate()` successfully assigns a port.
    selected_port: Option<u16>,
}

impl PortGenerator {
    /// Constructs a new `PortGenerator`.
    ///
    /// Initially, no port is selected (`selected_port` is `None`).
    /// The `generate` method must be called to select a port.
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

    /// Retrieves the selected port number.
    ///
    /// This method returns the port number selected by the `generate` method.
    /// If `generate` has not been called or was unsuccessful, this will return `None`.
    ///
    /// # Returns
    ///
    /// An `Option<u16>` representing the selected port.
    ///

    pub fn selected_port(&self) -> Option<u16> {
        self.selected_port
    }
}
