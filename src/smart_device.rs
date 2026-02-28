//! Smart device — enum wrapper around concrete device types.

use crate::devices::{Socket, Thermometer};
use crate::report::Report;
use std::fmt;

/// A smart device: either a [`Thermometer`] or a [`Socket`].
#[derive(Debug, Clone)]
pub enum SmartDevice {
    /// Thermometer variant.
    Thermometer(Thermometer),
    /// Socket variant.
    Socket(Socket),
}

impl SmartDevice {
    /// Returns a shared reference to the inner [`Socket`], or `None`.
    pub fn as_socket(&self) -> Option<&Socket> {
        match self {
            SmartDevice::Socket(s) => Some(s),
            SmartDevice::Thermometer(_) => None,
        }
    }

    /// Returns a mutable reference to the inner [`Socket`], or `None`.
    pub fn as_socket_mut(&mut self) -> Option<&mut Socket> {
        match self {
            SmartDevice::Socket(s) => Some(s),
            SmartDevice::Thermometer(_) => None,
        }
    }

    /// Returns a shared reference to the inner [`Thermometer`], or `None`.
    pub fn as_thermometer(&self) -> Option<&Thermometer> {
        match self {
            SmartDevice::Thermometer(t) => Some(t),
            SmartDevice::Socket(_) => None,
        }
    }

    /// Returns a mutable reference to the inner [`Thermometer`], or `None`.
    pub fn as_thermometer_mut(&mut self) -> Option<&mut Thermometer> {
        match self {
            SmartDevice::Thermometer(t) => Some(t),
            SmartDevice::Socket(_) => None,
        }
    }
}

// ── From conversions ──────────────────────────────────────────────────────────

impl From<Socket> for SmartDevice {
    fn from(socket: Socket) -> Self {
        SmartDevice::Socket(socket)
    }
}

impl From<Thermometer> for SmartDevice {
    fn from(therm: Thermometer) -> Self {
        SmartDevice::Thermometer(therm)
    }
}

// ── Report ────────────────────────────────────────────────────────────────────

impl Report for SmartDevice {
    fn report(&self) -> String {
        match self {
            SmartDevice::Thermometer(t) => {
                format!("Thermometer '{}': {} °C", t.name(), t.temperature())
            }
            SmartDevice::Socket(s) => {
                let status = if s.is_on() { "on" } else { "off" };
                format!("Socket '{}': {} (power: {} W)", s.name(), status, s.power())
            }
        }
    }
}

impl fmt::Display for SmartDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.report())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_thermometer_and_socket() {
        let therm = Thermometer::new("Living room", 22.0);
        let device = SmartDevice::Thermometer(therm);
        assert!(device.as_thermometer().is_some());
        assert!(device.as_socket().is_none());

        let socket = Socket::new("Lamp", 100.0);
        let device = SmartDevice::Socket(socket);
        assert!(device.as_socket().is_some());
        assert!(device.as_thermometer().is_none());
    }

    #[test]
    fn test_as_socket_mut() {
        let socket = Socket::new("Lamp", 100.0);
        let mut device = SmartDevice::Socket(socket);
        device.as_socket_mut().unwrap().turn_on();
        assert!(device.as_socket().unwrap().is_on());
    }

    #[test]
    fn test_from_socket() {
        let device: SmartDevice = Socket::new("Lamp", 60.0).into();
        assert!(device.as_socket().is_some());
    }

    #[test]
    fn test_from_thermometer() {
        let device: SmartDevice = Thermometer::new("Sensor", 22.0).into();
        assert!(device.as_thermometer().is_some());
    }

    #[test]
    fn test_report_thermometer() {
        let device: SmartDevice = Thermometer::new("Sensor", 22.5).into();
        let r = device.report();
        assert!(r.contains("Sensor"));
        assert!(r.contains("22.5"));
    }

    #[test]
    fn test_report_socket_off() {
        let device: SmartDevice = Socket::new("Lamp", 60.0).into();
        let r = device.report();
        assert!(r.contains("Lamp"));
        assert!(r.contains("off"));
    }

    #[test]
    fn test_report_socket_on() {
        let mut device: SmartDevice = Socket::new("Lamp", 60.0).into();
        device.as_socket_mut().unwrap().turn_on();
        let r = device.report();
        assert!(r.contains("on"));
        assert!(r.contains("60"));
    }
}
