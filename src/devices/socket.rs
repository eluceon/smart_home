//! Smart socket.

/// Represents a smart socket.
#[derive(Debug, Clone)]
pub struct Socket {
    name: String,
    is_on: bool,
    power_consumption: f32,
}

impl Socket {
    /// Creates a new smart socket.
    ///
    /// # Arguments
    ///
    /// * `name` - Socket name
    /// * `power_consumption` - Power consumption in watts when turned on
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::Socket;
    ///
    /// let socket = Socket::new("Desk lamp", 60.0);
    /// assert!(!socket.is_on());
    /// ```
    pub fn new(name: impl Into<String>, power_consumption: f32) -> Self {
        Self {
            name: name.into(),
            is_on: false,
            power_consumption,
        }
    }

    /// Turns the socket on.
    pub fn turn_on(&mut self) {
        self.is_on = true;
    }

    /// Turns the socket off.
    pub fn turn_off(&mut self) {
        self.is_on = false;
    }

    /// Returns whether the socket is on.
    pub fn is_on(&self) -> bool {
        self.is_on
    }

    /// Returns the current power draw.
    ///
    /// Returns 0.0 when the socket is off, otherwise the nominal power value.
    pub fn power(&self) -> f32 {
        if self.is_on {
            self.power_consumption
        } else {
            0.0
        }
    }

    /// Returns the socket name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the nominal power consumption.
    pub fn power_consumption(&self) -> f32 {
        self.power_consumption
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_creation() {
        let socket = Socket::new("Air conditioner".to_string(), 1500.0);
        assert!(!socket.is_on());
        assert_eq!(socket.power(), 0.0);
        assert_eq!(socket.name(), "Air conditioner");
    }

    #[test]
    fn test_socket_turn_on_off() {
        let mut socket = Socket::new("Kettle".to_string(), 2000.0);

        socket.turn_on();
        assert!(socket.is_on());
        assert_eq!(socket.power(), 2000.0);

        socket.turn_off();
        assert!(!socket.is_on());
        assert_eq!(socket.power(), 0.0);
    }

    #[test]
    fn test_socket_power_consumption() {
        let socket = Socket::new("Fridge".to_string(), 800.0);
        assert_eq!(socket.power_consumption(), 800.0);
    }
}
