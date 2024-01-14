use rand::seq::SliceRandom;
use rand::thread_rng;
use std::net::TcpListener;

/// Represents a Port Generator to find available ports within a specified range.
pub struct PortGenerator {
    port_range: String,
    selected_port: Option<u16>,
}

impl PortGenerator {
    /// Creates a new PortGenerator with the specified port range.
    ///
    /// # Arguments
    /// * `start_range` - The starting port number of the range.
    /// * `end_range` - The ending port number of the range.
    pub fn new(start_range: u16, end_range: u16) -> Self {
        PortGenerator {
            port_range: format!("{}-{}", start_range, end_range),
            selected_port: None,
        }
    }

    /// Attempts to find a free port within the specified range.
    ///
    /// # Returns
    /// `Some(port)` if a free port is found, or `None` if no free port is available.
    //TODO: Maybe we need to return the random number as a Result and implement proper error handeling
    pub fn generate(&mut self) -> &mut Self {
        let mut rng = thread_rng();
        let port_range: Vec<u16> = (self.port_range_start()..=self.port_range_end()).collect();
        let mut shuffled_ports = port_range;
        shuffled_ports.shuffle(&mut rng);

        for port in shuffled_ports {
            if TcpListener::bind(("127.0.0.1", port)).is_ok() {
                self.selected_port = Some(port);
                break; // Exit the loop once a port is found
            }
        }

        self // Return a mutable reference to the object itself
    }

    /// Gets the start of the port range.
    fn port_range_start(&self) -> u16 {
        self.port_range.split('-').next().unwrap().parse().unwrap()
    }

    /// Gets the end of the port range.
    fn port_range_end(&self) -> u16 {
        self.port_range.split('-').last().unwrap().parse().unwrap()
    }

    /// Returns the selected port, if available.
    pub fn selected_port(&self) -> Option<u16> {
        self.selected_port
    }
}
