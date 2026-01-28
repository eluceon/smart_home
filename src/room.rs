//! Smart home room.

use crate::smart_device::SmartDevice;
use std::io::{self, Write};

/// Represents a room with a list of smart devices.
#[derive(Debug, Clone)]
pub struct Room {
    name: String,
    devices: Vec<SmartDevice>,
}

impl Room {
    /// Creates a new room with the given devices.
    ///
    /// # Arguments
    ///
    /// * `name` - Room name
    /// * `devices` - Devices in the room
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{Room, SmartDevice, Thermometer};
    ///
    /// let devices = vec![
    ///     SmartDevice::Thermometer(Thermometer::new("Sensor".to_string(), 20.0))
    /// ];
    /// let room = Room::new("Bedroom".to_string(), devices);
    /// assert_eq!(room.name(), "Bedroom");
    /// ```
    pub fn new(name: String, devices: Vec<SmartDevice>) -> Self {
        Self { name, devices }
    }

    /// Returns the room name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the number of devices in the room.
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// Returns a reference to a device by index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn get_device(&self, index: usize) -> &SmartDevice {
        &self.devices[index]
    }

    /// Returns a mutable reference to a device by index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn get_device_mut(&mut self, index: usize) -> &mut SmartDevice {
        &mut self.devices[index]
    }

    /// Writes a report for all devices in the room.
    pub fn write_report<W: Write>(&self, mut writer: W) -> io::Result<()> {
        writeln!(writer, "\nRoom: {}", self.name)?;
        writeln!(writer, "  Devices: {}", self.devices.len())?;
        for device in &self.devices {
            writeln!(writer, "  {}", device)?;
        }
        Ok(())
    }

    /// Prints a report for all devices in the room to stdout.
    pub fn print_report(&self) {
        let _ = self.write_report(io::stdout());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devices::{Socket, Thermometer};

    #[test]
    fn test_room_creation() {
        let devices = vec![
            SmartDevice::Thermometer(Thermometer::new("Sensor".to_string(), 20.0)),
            SmartDevice::Socket(Socket::new("Lamp".to_string(), 100.0)),
        ];
        let room = Room::new("Living room".to_string(), devices);

        assert_eq!(room.name(), "Living room");
        assert_eq!(room.device_count(), 2);
    }

    #[test]
    fn test_room_get_device() {
        let devices = vec![SmartDevice::Thermometer(Thermometer::new(
            "Sensor".to_string(),
            20.0,
        ))];
        let room = Room::new("Bedroom".to_string(), devices);

        let device = room.get_device(0);
        assert!(device.as_thermometer().is_some());
    }

    #[test]
    fn test_room_get_device_mut() {
        let devices = vec![SmartDevice::Socket(Socket::new("Lamp".to_string(), 100.0))];
        let mut room = Room::new("Kitchen".to_string(), devices);

        if let Some(socket) = room.get_device_mut(0).as_socket_mut() {
            socket.turn_on();
        }

        assert!(room.get_device(0).as_socket().unwrap().is_on());
    }

    #[test]
    #[should_panic]
    fn test_room_get_device_out_of_bounds() {
        let devices = vec![];
        let room = Room::new("Bathroom".to_string(), devices);
        let _ = room.get_device(0);
    }
}
