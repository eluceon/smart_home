//! "Smart Home" library.
//!
//! Provides basic building blocks for a smart home, including devices
//! (thermometers, sockets) organized into rooms and a house.

pub mod devices;
pub mod room;
pub mod smart_device;
pub mod smart_home;

pub use devices::{Socket, Thermometer};
pub use room::Room;
pub use smart_device::SmartDevice;
pub use smart_home::SmartHome;
