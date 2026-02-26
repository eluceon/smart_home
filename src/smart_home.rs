//! Smart home.

use crate::room::Room;
use std::io::{self, Write};

/// Represents a smart home with a list of rooms.
#[derive(Debug, Clone)]
pub struct SmartHome {
    name: String,
    rooms: Vec<Room>,
}

impl SmartHome {
    /// Creates a new smart home with the given rooms.
    ///
    /// # Arguments
    ///
    /// * `name` - Home name
    /// * `rooms` - Rooms in the home
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{SmartHome, Room};
    ///
    /// let rooms = vec![];
    /// let house = SmartHome::new("My home", rooms);
    /// assert_eq!(house.name(), "My home");
    /// ```
    pub fn new(name: impl Into<String>, rooms: Vec<Room>) -> Self {
        Self {
            name: name.into(),
            rooms,
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

    /// Returns a reference to a room by index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn get_room(&self, index: usize) -> &Room {
        &self.rooms[index]
    }

    /// Returns a mutable reference to a room by index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn get_room_mut(&mut self, index: usize) -> &mut Room {
        &mut self.rooms[index]
    }

    /// Writes a full report of the home and all rooms.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `writer` fails.
    pub fn write_full_report<W: Write>(&self, mut writer: W) -> io::Result<()> {
        let separator = "=".repeat(50);
        writeln!(writer, "\n{}", separator)?;
        writeln!(writer, "Smart home: {}", self.name)?;
        writeln!(writer, "Rooms: {}", self.rooms.len())?;
        writeln!(writer, "{}", separator)?;

        for room in &self.rooms {
            room.write_report(&mut writer)?;
        }

        writeln!(writer, "\n{}", separator)?;
        Ok(())
    }

    /// Prints a full report of the home and all rooms to stdout.
    pub fn print_full_report(&self) {
        if let Err(e) = self.write_full_report(io::stdout()) {
            eprintln!("Failed to write home report: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devices::{Socket, Thermometer};
    use crate::smart_device::SmartDevice;

    #[test]
    fn test_smart_home_creation() {
        let rooms = vec![];
        let house = SmartHome::new("Apartment".to_string(), rooms);

        assert_eq!(house.name(), "Apartment");
        assert_eq!(house.room_count(), 0);
    }

    #[test]
    fn test_smart_home_get_room() {
        let devices = vec![SmartDevice::Thermometer(Thermometer::new(
            "Sensor".to_string(),
            20.0,
        ))];
        let rooms = vec![Room::new("Bedroom".to_string(), devices)];
        let house = SmartHome::new("Home".to_string(), rooms);

        assert_eq!(house.get_room(0).name(), "Bedroom");
    }

    #[test]
    fn test_smart_home_get_room_mut() {
        let devices = vec![SmartDevice::Socket(Socket::new("Lamp".to_string(), 100.0))];
        let rooms = vec![Room::new("Kitchen".to_string(), devices)];
        let mut house = SmartHome::new("Home".to_string(), rooms);

        let room = house.get_room_mut(0);
        if let Some(socket) = room.get_device_mut(0).as_socket_mut() {
            socket.turn_on();
        }

        assert!(house.get_room(0).get_device(0).as_socket().unwrap().is_on());
    }

    #[test]
    #[should_panic]
    fn test_smart_home_get_room_out_of_bounds() {
        let house = SmartHome::new("Home", vec![]);
        let _ = house.get_room(0);
    }

    #[test]
    #[should_panic]
    fn test_smart_home_get_room_mut_out_of_bounds() {
        let mut house = SmartHome::new("Home", vec![]);
        let _ = house.get_room_mut(0);
    }
}
