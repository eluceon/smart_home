//! Smart thermometer — local mock and UDP-receiving variants.

use crate::error::NetworkError;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Background UDP listener that stores the last received temperature.
#[derive(Debug)]
struct UdpReceiver {
    temperature: Arc<RwLock<Option<f32>>>,
    shutdown: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    bound_addr: SocketAddr,
}

impl UdpReceiver {
    fn new(bind_addr: &str) -> Result<Self, NetworkError> {
        let temperature: Arc<RwLock<Option<f32>>> = Arc::new(RwLock::new(None));
        let shutdown = Arc::new(AtomicBool::new(false));

        let socket = UdpSocket::bind(bind_addr)?;
        // Short timeout so the thread can check the shutdown flag periodically.
        socket.set_read_timeout(Some(Duration::from_millis(200)))?;
        let bound_addr = socket.local_addr()?;

        let temp_ref = temperature.clone();
        let shut_ref = shutdown.clone();

        let thread = thread::spawn(move || {
            let mut buf = [0u8; 64];
            loop {
                if shut_ref.load(Ordering::Acquire) {
                    break;
                }
                match socket.recv_from(&mut buf) {
                    Ok((len, _)) => {
                        if let Ok(s) = std::str::from_utf8(&buf[..len]) {
                            if let Ok(temp) = s.trim().parse::<f32>() {
                                if let Ok(mut guard) = temp_ref.write() {
                                    *guard = Some(temp);
                                    log::debug!("UDP thermometer received {temp:.1} °C");
                                }
                            }
                        }
                    }
                    // Timeout / would-block: loop again to check shutdown flag.
                    Err(ref e)
                        if e.kind() == std::io::ErrorKind::WouldBlock
                            || e.kind() == std::io::ErrorKind::TimedOut =>
                    {
                        continue
                    }
                    Err(e) => {
                        log::error!("UDP receive error: {e}");
                        break;
                    }
                }
            }
        });

        Ok(Self {
            temperature,
            shutdown,
            thread: Some(thread),
            bound_addr,
        })
    }

    fn temperature(&self) -> Result<f32, NetworkError> {
        self.temperature
            .read()
            .map_err(|_| NetworkError::Protocol("RwLock poisoned".into()))?
            .ok_or(NetworkError::NoDataReceived)
    }

    fn bound_addr(&self) -> SocketAddr {
        self.bound_addr
    }
}

impl Drop for UdpReceiver {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

#[derive(Debug)]
enum ThermometerBackend {
    Local(f32),
    Udp(UdpReceiver),
}

/// A smart thermometer that can operate locally (mock) or by receiving UDP
/// packets from a running `thermo_emulator` binary.
///
/// When created with [`Thermometer::new_udp`], a background thread is spawned
/// that listens for incoming UDP datagrams and stores the latest temperature.
/// The thread is joined when the `Thermometer` is dropped.
#[derive(Debug)]
pub struct Thermometer {
    name: String,
    backend: ThermometerBackend,
}

impl Thermometer {
    /// Creates a local (in-process) thermometer with the given initial temperature.
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::Thermometer;
    ///
    /// let thermometer = Thermometer::new("Living room", 22.5);
    /// assert_eq!(thermometer.temperature().unwrap(), 22.5);
    /// ```
    pub fn new(name: impl Into<String>, temperature: f32) -> Self {
        Self {
            name: name.into(),
            backend: ThermometerBackend::Local(temperature),
        }
    }

    /// Creates a UDP-receiving thermometer that binds to `bind_addr` and
    /// accepts temperature datagrams from a `thermo_emulator` binary.
    ///
    /// A background thread is started immediately and runs until the
    /// `Thermometer` is dropped. Pass `"host:0"` to let the OS pick a free port,
    /// then retrieve the actual address with [`Thermometer::bound_addr`].
    ///
    /// # Errors
    ///
    /// Returns [`NetworkError::Io`] if the UDP socket cannot be bound.
    pub fn new_udp(name: impl Into<String>, bind_addr: &str) -> Result<Self, NetworkError> {
        Ok(Self {
            name: name.into(),
            backend: ThermometerBackend::Udp(UdpReceiver::new(bind_addr)?),
        })
    }

    /// Returns the thermometer name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the local address the UDP socket is bound to, or `None` for
    /// local (mock) thermometers.
    pub fn bound_addr(&self) -> Option<SocketAddr> {
        match &self.backend {
            ThermometerBackend::Udp(r) => Some(r.bound_addr()),
            ThermometerBackend::Local(_) => None,
        }
    }

    /// Returns the current temperature in °C.
    ///
    /// # Errors
    ///
    /// - [`NetworkError::NoDataReceived`] if the UDP thermometer has not yet
    ///   received any packet.
    #[must_use = "check the temperature value or handle the error"]
    pub fn temperature(&self) -> Result<f32, NetworkError> {
        match &self.backend {
            ThermometerBackend::Local(t) => Ok(*t),
            ThermometerBackend::Udp(recv) => recv.temperature(),
        }
    }

    /// Updates the temperature of a local thermometer.
    ///
    /// Has no effect on UDP thermometers (their temperature is set by incoming
    /// UDP packets).
    pub fn set_temperature(&mut self, temperature: f32) {
        if let ThermometerBackend::Local(t) = &mut self.backend {
            *t = temperature;
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thermometer_creation() {
        let thermometer = Thermometer::new("Living room thermometer", 20.0);
        assert_eq!(thermometer.temperature().unwrap(), 20.0);
        assert_eq!(thermometer.name(), "Living room thermometer");
    }

    #[test]
    fn test_thermometer_update_temperature() {
        let mut thermometer = Thermometer::new("Test thermometer", 18.0);
        thermometer.set_temperature(25.5);
        assert_eq!(thermometer.temperature().unwrap(), 25.5);
    }

    #[test]
    fn test_udp_thermometer_no_data() {
        // Bind to an ephemeral port; no sender → NoDataReceived.
        let thermo = Thermometer::new_udp("Sensor", "127.0.0.1:0").unwrap();
        assert!(matches!(
            thermo.temperature(),
            Err(NetworkError::NoDataReceived)
        ));
    }

    #[test]
    fn test_udp_thermometer_receives_data() {
        // Bind to an ephemeral port; retrieve the actual address to avoid TOCTOU.
        let thermo = Thermometer::new_udp("Sensor", "127.0.0.1:0").unwrap();
        let bind_addr = thermo.bound_addr().unwrap();

        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        sender.send_to(b"23.7", bind_addr).unwrap();

        thread::sleep(Duration::from_millis(400));
        let temp = thermo.temperature().unwrap();
        assert!((temp - 23.7).abs() < 0.01);
    }
}
