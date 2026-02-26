//! Smart device (enum wrapper).

use crate::devices::{Socket, Thermometer};
use std::fmt;

/// Represents a smart device.
///
/// Can be either a thermometer or a socket.
#[derive(Debug, Clone)]
pub enum SmartDevice {
    /// Thermometer variant.
    Thermometer(Thermometer),
    /// Socket variant.
    Socket(Socket),
}

impl SmartDevice {
    /// Returns a mutable reference to the socket if this device is a socket.
    pub fn as_socket_mut(&mut self) -> Option<&mut Socket> {
        match self {
            SmartDevice::Socket(socket) => Some(socket),
            SmartDevice::Thermometer(_) => None,
        }
    }

    /// Returns a reference to the socket if this device is a socket.
    pub fn as_socket(&self) -> Option<&Socket> {
        match self {
            SmartDevice::Socket(socket) => Some(socket),
            SmartDevice::Thermometer(_) => None,
        }
    }

    /// Returns a reference to the thermometer if this device is a thermometer.
    pub fn as_thermometer(&self) -> Option<&Thermometer> {
        match self {
            SmartDevice::Thermometer(therm) => Some(therm),
            SmartDevice::Socket(_) => None,
        }
    }

    /// Returns a mutable reference to the thermometer if this device is a thermometer.
    pub fn as_thermometer_mut(&mut self) -> Option<&mut Thermometer> {
        match self {
            SmartDevice::Thermometer(therm) => Some(therm),
            SmartDevice::Socket(_) => None,
        }
    }
}

impl fmt::Display for SmartDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SmartDevice::Thermometer(therm) => {
                write!(
                    f,
                    "Thermometer '{}': {} C",
                    therm.name(),
                    therm.temperature()
                )
            }
            SmartDevice::Socket(socket) => {
                let status = if socket.is_on() { "on" } else { "off" };
                let power = socket.power();
                write!(
                    f,
                    "Socket '{}': {} (power: {} W)",
                    socket.name(),
                    status,
                    power
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_device_thermometer() {
        let therm = Thermometer::new("Living room".to_string(), 22.0);
        let device = SmartDevice::Thermometer(therm);

        assert!(device.as_thermometer().is_some());
        assert!(device.as_socket().is_none());
    }

    #[test]
    fn test_smart_device_socket() {
        let socket = Socket::new("Lamp".to_string(), 100.0);
        let device = SmartDevice::Socket(socket);

        assert!(device.as_socket().is_some());
        assert!(device.as_thermometer().is_none());
    }

    #[test]
    fn test_smart_device_socket_mut() {
        let socket = Socket::new("Lamp".to_string(), 100.0);
        let mut device = SmartDevice::Socket(socket);

        if let Some(socket) = device.as_socket_mut() {
            socket.turn_on();
        }

        let socket = device.as_socket().unwrap();
        assert!(socket.is_on());
    }
}
