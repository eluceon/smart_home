//! Smart home room with observer-pattern subscribers.

use crate::report::Report;
use crate::smart_device::SmartDevice;
use std::collections::HashMap;

/// Observer that is notified whenever a device is added to a room.
///
/// Implementors receive a reference to the newly added device. Closures
/// `FnMut(&SmartDevice)` automatically implement this trait.
pub trait Subscriber {
    /// Called after a device has been added to the room.
    fn on_event(&mut self, device: &SmartDevice);
}

/// Blanket implementation so closures can be used as subscribers.
impl<F: FnMut(&SmartDevice)> Subscriber for F {
    fn on_event(&mut self, device: &SmartDevice) {
        self(device);
    }
}

/// A room that holds a named collection of smart devices and notifies
/// registered [`Subscriber`]s when a device is added.
pub struct Room {
    name: String,
    devices: HashMap<String, SmartDevice>,
    subscribers: Vec<Box<dyn Subscriber>>,
}

impl std::fmt::Debug for Room {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Room")
            .field("name", &self.name)
            .field("devices", &self.devices)
            .field("subscribers", &self.subscribers.len())
            .finish()
    }
}

impl Room {
    /// Creates a new empty room.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            devices: HashMap::new(),
            subscribers: Vec::new(),
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
    ///
    /// All registered [`Subscriber`]s are notified only when a **new** key is
    /// inserted. If the key already exists, the device is replaced and the
    /// previous [`SmartDevice`] is returned.
    ///
    /// Returns `Some(old_device)` if the key was already present, `None`
    /// otherwise.
    pub fn add_device(
        &mut self,
        name: impl Into<String>,
        device: impl Into<SmartDevice>,
    ) -> Option<SmartDevice> {
        use std::collections::hash_map::Entry;
        let key: String = name.into();
        let device: SmartDevice = device.into();
        match self.devices.entry(key) {
            Entry::Occupied(mut e) => Some(e.insert(device)),
            Entry::Vacant(e) => {
                let d = e.insert(device);
                for sub in &mut self.subscribers {
                    sub.on_event(d);
                }
                None
            }
        }
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

    /// Registers a subscriber that will be notified on every [`Room::add_device`].
    ///
    /// Accepts both structs implementing [`Subscriber`] and closures
    /// `FnMut(&SmartDevice)`.
    pub fn subscribe(&mut self, subscriber: impl Subscriber + 'static) {
        self.subscribers.push(Box::new(subscriber));
    }
}

// ── Default ────────────────────────────────────────────────────────────────────

impl Default for Room {
    fn default() -> Self {
        Self::new("Unnamed room")
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
            .turn_on()
            .unwrap();

        assert!(room
            .get_device("lamp")
            .unwrap()
            .as_socket()
            .unwrap()
            .is_on()
            .unwrap());
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

    // ── Observer tests ──────────────────────────────────────────────────────

    #[test]
    fn test_subscriber_notified_on_add() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let mut room = Room::new("Test");
        let count = Rc::new(RefCell::new(0usize));
        let count_clone = count.clone();

        room.subscribe(move |_device: &SmartDevice| {
            *count_clone.borrow_mut() += 1;
        });

        room.add_device("lamp", Socket::default());
        room.add_device("sensor", Thermometer::default());

        assert_eq!(*count.borrow(), 2);
        assert_eq!(room.device_count(), 2);
    }

    #[test]
    fn test_closure_subscriber() {
        use std::cell::RefCell;

        let events: std::rc::Rc<RefCell<Vec<String>>> = std::rc::Rc::new(RefCell::new(Vec::new()));
        let events_clone = events.clone();

        let mut room = Room::new("Test");
        room.subscribe(move |device: &SmartDevice| {
            events_clone.borrow_mut().push(device.report().to_string());
        });

        room.add_device("lamp", Socket::new("Desk lamp", 60.0));
        room.add_device("sensor", Thermometer::new("Sensor", 22.5));

        let logged = events.borrow();
        assert_eq!(logged.len(), 2);
        assert!(logged[0].contains("Desk lamp"));
        assert!(logged[1].contains("Sensor"));
    }

    #[test]
    fn test_subscriber_default_room() {
        let mut room = Room::default();
        room.subscribe(|_device: &SmartDevice| {
            // no-op
        });
        room.add_device("test", Socket::default());
        assert_eq!(room.device_count(), 1);
    }
}
