//! Smart home — top-level container for rooms.

use crate::error::SmartHomeError;
use crate::report::Report;
use crate::room::Room;
use crate::smart_device::SmartDevice;
use std::collections::HashMap;

/// A smart home that holds a named collection of rooms.
#[derive(Debug, Clone)]
pub struct SmartHome {
    name: String,
    rooms: HashMap<String, Room>,
}

impl SmartHome {
    /// Creates a new empty smart home.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            rooms: HashMap::new(),
        }
    }

    /// Returns the home name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the number of rooms.
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    /// Adds a room under the given key.
    pub fn add_room(&mut self, name: impl Into<String>, room: Room) {
        self.rooms.insert(name.into(), room);
    }

    /// Removes and returns the room with the given key, or `None` if absent.
    pub fn remove_room(&mut self, name: &str) -> Option<Room> {
        self.rooms.remove(name)
    }

    /// Returns a shared reference to the room with the given key, or `None`.
    pub fn get_room(&self, name: &str) -> Option<&Room> {
        self.rooms.get(name)
    }

    /// Returns a mutable reference to the room with the given key, or `None`.
    pub fn get_room_mut(&mut self, name: &str) -> Option<&mut Room> {
        self.rooms.get_mut(name)
    }

    /// Returns a shared reference to a device identified by room and device keys.
    ///
    /// # Errors
    ///
    /// - [`SmartHomeError::RoomNotFound`] if `room_name` does not exist.
    /// - [`SmartHomeError::DeviceNotFound`] if `device_name` does not exist in the room.
    pub fn get_device(
        &self,
        room_name: &str,
        device_name: &str,
    ) -> Result<&SmartDevice, SmartHomeError> {
        let room = self
            .rooms
            .get(room_name)
            .ok_or_else(|| SmartHomeError::RoomNotFound(room_name.to_string()))?;
        room.get_device(device_name)
            .ok_or_else(|| SmartHomeError::DeviceNotFound(device_name.to_string()))
    }
}

// ── Report ────────────────────────────────────────────────────────────────────

impl Report for SmartHome {
    fn report(&self) -> String {
        let sep = "=".repeat(50);
        let mut s = format!(
            "\n{}\nSmart Home '{}' ({} room(s)):\n{}\n",
            sep,
            self.name,
            self.rooms.len(),
            sep
        );
        let mut keys: Vec<&String> = self.rooms.keys().collect();
        keys.sort();
        for key in keys {
            s.push_str(&format!("\n[Room: {}]\n", key));
            s.push_str(&self.rooms[key].report());
        }
        s.push_str(&format!("\n{}\n", sep));
        s
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devices::{Socket, Thermometer};

    fn make_home() -> SmartHome {
        let mut home = SmartHome::new("Apartment");

        let mut living_room = Room::new("Living room");
        living_room.add_device("sensor", Thermometer::new("Sensor", 20.0));
        living_room.add_device("lamp", Socket::new("Lamp", 60.0));

        let mut bedroom = Room::new("Bedroom");
        bedroom.add_device("sensor", Thermometer::new("Sensor", 18.0));
        bedroom.add_device("heater", Socket::new("Space heater", 2000.0));

        home.add_room("living_room", living_room);
        home.add_room("bedroom", bedroom);
        home
    }

    #[test]
    fn test_creation() {
        let home = SmartHome::new("Apartment");
        assert_eq!(home.name(), "Apartment");
        assert_eq!(home.room_count(), 0);
    }

    #[test]
    fn test_add_remove_room() {
        let mut home = SmartHome::new("Home");
        home.add_room("kitchen", Room::new("Kitchen"));
        assert_eq!(home.room_count(), 1);

        assert!(home.remove_room("kitchen").is_some());
        assert_eq!(home.room_count(), 0);
        assert!(home.remove_room("kitchen").is_none());
    }

    #[test]
    fn test_get_room() {
        let home = make_home();
        assert!(home.get_room("living_room").is_some());
        assert!(home.get_room("nonexistent").is_none());
    }

    #[test]
    fn test_get_room_mut_and_toggle_socket() {
        let mut home = make_home();
        home.get_room_mut("bedroom")
            .unwrap()
            .get_device_mut("heater")
            .unwrap()
            .as_socket_mut()
            .unwrap()
            .turn_on();

        assert!(home
            .get_room("bedroom")
            .unwrap()
            .get_device("heater")
            .unwrap()
            .as_socket()
            .unwrap()
            .is_on());
    }

    #[test]
    fn test_get_device_ok() {
        let home = make_home();
        assert!(home.get_device("living_room", "lamp").is_ok());
    }

    #[test]
    fn test_get_device_room_not_found() {
        let home = make_home();
        assert!(matches!(
            home.get_device("no_such_room", "lamp"),
            Err(SmartHomeError::RoomNotFound(_))
        ));
    }

    #[test]
    fn test_get_device_device_not_found() {
        let home = make_home();
        assert!(matches!(
            home.get_device("living_room", "no_such_device"),
            Err(SmartHomeError::DeviceNotFound(_))
        ));
    }

    #[test]
    fn test_report_contains_home_and_rooms() {
        let home = make_home();
        let r = home.report();
        assert!(r.contains("Apartment"));
        assert!(r.contains("living_room"));
        assert!(r.contains("bedroom"));
    }
}
