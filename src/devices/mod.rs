//! Device module for the smart home.

pub mod socket;
pub mod thermometer;

pub use socket::Socket;
pub use thermometer::Thermometer;
