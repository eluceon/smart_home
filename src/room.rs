//! Smart home room.

use crate::report::Report;
use crate::smart_device::SmartDevice;
use std::collections::HashMap;

/// A room that holds a named collection of smart devices.
#[derive(Debug, Clone)]
pub struct Room {
    name: String,
    devices: HashMap<String, SmartDevice>,
}

impl Room {
    /// Creates a new empty room.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            devices: HashMap::new(),
        }
    }

    /// Returns the room name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the number of devices in the room.
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// Adds a device to the room under the given key.
    ///
    /// Accepts any type that converts into [`SmartDevice`] (e.g. [`Socket`][crate::Socket]
    /// or [`Thermometer`][crate::Thermometer]).
    pub fn add_device(&mut self, name: impl Into<String>, device: impl Into<SmartDevice>) {
        self.devices.insert(name.into(), device.into());
    }

    /// Removes and returns the device with the given key, or `None` if absent.
    pub fn remove_device(&mut self, name: &str) -> Option<SmartDevice> {
        self.devices.remove(name)
    }

    /// Returns a shared reference to the device with the given key, or `None`.
    pub fn get_device(&self, name: &str) -> Option<&SmartDevice> {
        self.devices.get(name)
    }

    /// Returns a mutable reference to the device with the given key, or `None`.
    pub fn get_device_mut(&mut self, name: &str) -> Option<&mut SmartDevice> {
        self.devices.get_mut(name)
    }
}

// ── Report ────────────────────────────────────────────────────────────────────

impl Report for Room {
    fn report(&self) -> String {
        let mut s = format!("Room '{}' ({} device(s)):\n", self.name, self.devices.len());
        let mut keys: Vec<&String> = self.devices.keys().collect();
        keys.sort();
        for key in keys {
            s.push_str(&format!("  [{}] {}\n", key, self.devices[key].report()));
        }
        s
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devices::{Socket, Thermometer};

    #[test]
    fn test_room_creation() {
        let room = Room::new("Living room");
        assert_eq!(room.name(), "Living room");
        assert_eq!(room.device_count(), 0);
    }

    #[test]
    fn test_add_and_count() {
        let mut room = Room::new("Bedroom");
        room.add_device("sensor", Thermometer::new("Sensor", 20.0));
        room.add_device("lamp", Socket::new("Lamp", 100.0));
        assert_eq!(room.device_count(), 2);
    }

    #[test]
    fn test_get_device() {
        let mut room = Room::new("Bedroom");
        room.add_device("sensor", Thermometer::new("Sensor", 20.0));

        assert!(room
            .get_device("sensor")
            .unwrap()
            .as_thermometer()
            .is_some());
        assert!(room.get_device("nonexistent").is_none());
    }

    #[test]
    fn test_get_device_mut() {
        let mut room = Room::new("Kitchen");
        room.add_device("lamp", Socket::new("Lamp", 100.0));

        room.get_device_mut("lamp")
            .and_then(|d| d.as_socket_mut())
            .unwrap()
            .turn_on();

        assert!(room
            .get_device("lamp")
            .unwrap()
            .as_socket()
            .unwrap()
            .is_on());
    }

    #[test]
    fn test_remove_device() {
        let mut room = Room::new("Bathroom");
        room.add_device("light", Socket::new("Light", 60.0));
        assert_eq!(room.device_count(), 1);

        assert!(room.remove_device("light").is_some());
        assert_eq!(room.device_count(), 0);
        assert!(room.remove_device("light").is_none());
    }

    #[test]
    fn test_report_contains_name_and_key() {
        let mut room = Room::new("Hall");
        room.add_device("sensor", Thermometer::new("Sensor", 22.5));
        let r = room.report();
        assert!(r.contains("Hall"));
        assert!(r.contains("sensor"));
        assert!(r.contains("22.5"));
    }
}
