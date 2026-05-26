//! Typestate builder for [`SmartHome`].

use crate::room::Room;
use crate::smart_device::SmartDevice;
use crate::smart_home::SmartHome;
use std::marker::PhantomData;

/// Marker type: no rooms have been added yet.
pub struct NoRooms;
/// Marker type: at least one room has been added.
pub struct HasRooms;

/// A typestate builder for constructing a [`SmartHome`].
///
/// Before the first room is added only [`HomeBuilder::add_room`] is available.
/// After at least one room, devices can be added to the current room and
/// additional rooms can be appended.
///
/// # Examples
///
/// ```
/// use smart_home::{HomeBuilder, Socket, Thermometer};
///
/// let home = HomeBuilder::new()
///     .add_room("Living room")
///     .add_device("lamp", Socket::default())
///     .add_device("sensor", Thermometer::default())
///     .add_room("Bedroom")
///     .add_device("heater", Socket::default())
///     .build();
/// ```
pub struct HomeBuilder<State = NoRooms> {
    home: SmartHome,
    current_room: Option<Room>,
    current_room_name: Option<String>,
    _state: PhantomData<State>,
}

impl HomeBuilder<NoRooms> {
    /// Creates a new builder with no rooms.
    pub fn new() -> Self {
        Self {
            home: SmartHome::new("My Smart Home"),
            current_room: None,
            current_room_name: None,
            _state: PhantomData,
        }
    }

    /// Adds the first room and transitions to the [`HasRooms`] state.
    ///
    /// Subsequent calls to [`add_device`](HomeBuilder::add_device) will place
    /// devices into this room until another room is added.
    #[must_use = "builder methods consume self and return a new state"]
    pub fn add_room(mut self, name: impl Into<String>) -> HomeBuilder<HasRooms> {
        let name: String = name.into();
        self.current_room = Some(Room::new(name.clone()));
        self.current_room_name = Some(name);
        HomeBuilder {
            home: self.home,
            current_room: self.current_room,
            current_room_name: self.current_room_name,
            _state: PhantomData,
        }
    }
}

impl Default for HomeBuilder<NoRooms> {
    fn default() -> Self {
        Self::new()
    }
}

impl HomeBuilder<HasRooms> {
    /// Adds a device to the *current* room (the most recently added room).
    #[must_use = "builder methods consume self and return a new state"]
    pub fn add_device(mut self, key: impl Into<String>, device: impl Into<SmartDevice>) -> Self {
        if let Some(ref mut room) = self.current_room {
            room.add_device(key, device);
        }
        self
    }

    /// Finalises the current room (if any), then begins a new room.
    ///
    /// Subsequent [`add_device`](HomeBuilder::add_device) calls will place
    /// devices into this new room.
    #[must_use = "builder methods consume self and return a new state"]
    pub fn add_room(mut self, name: impl Into<String>) -> Self {
        // Commit the previous room to the home.
        if let (Some(room), Some(room_name)) =
            (self.current_room.take(), self.current_room_name.take())
        {
            self.home.add_room(room_name, room);
        }

        let name: String = name.into();
        self.current_room = Some(Room::new(name.clone()));
        self.current_room_name = Some(name);
        self
    }

    /// Consumes the builder and returns the assembled [`SmartHome`].
    ///
    /// If a room was being built, it is committed first.
    #[must_use = "build() consumes the builder and returns the SmartHome"]
    pub fn build(mut self) -> SmartHome {
        if let (Some(room), Some(room_name)) =
            (self.current_room.take(), self.current_room_name.take())
        {
            self.home.add_room(room_name, room);
        }
        self.home
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devices::{Socket, Thermometer};

    #[test]
    fn test_builder_creates_home() {
        let home = HomeBuilder::new()
            .add_room("Living room")
            .add_device("lamp", Socket::default())
            .add_device("sensor", Thermometer::default())
            .build();

        assert_eq!(home.room_count(), 1);
        assert!(home.get_room("Living room").is_some());
        assert_eq!(home.get_room("Living room").unwrap().device_count(), 2);
    }

    #[test]
    fn test_builder_multiple_rooms() {
        let home = HomeBuilder::new()
            .add_room("Room A")
            .add_device("dev_a", Socket::default())
            .add_room("Room B")
            .add_device("dev_b", Thermometer::default())
            .build();

        assert_eq!(home.room_count(), 2);
        assert!(home.get_room("Room A").is_some());
        assert!(home.get_room("Room B").is_some());
    }

    #[test]
    fn test_builder_empty_room_then_second() {
        let home = HomeBuilder::new()
            .add_room("Empty room")
            .add_room("Room with device")
            .add_device("lamp", Socket::default())
            .build();

        assert_eq!(home.room_count(), 2);
        assert_eq!(home.get_room("Empty room").unwrap().device_count(), 0);
        assert_eq!(home.get_room("Room with device").unwrap().device_count(), 1);
    }
}
