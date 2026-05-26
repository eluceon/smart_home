//! Reporter — statically-typed heterogeneous list of [`Report`] implementors.
//!
//! Uses nested tuples for compile-time composition. Each [`Reporter::add`] call
//! wraps the previous type, building a recursive type-level list.
//!
//! # Examples
//!
//! ```
//! use smart_home::{Reporter, Room, SmartDevice, Socket, Thermometer};
//!
//! let room = Room::default();
//! let device = SmartDevice::default();
//! let socket = Socket::default();
//!
//! let report = Reporter::new()
//!     .add(&room)
//!     .add(&device)
//!     .add(&socket)
//!     .report();
//! ```

use crate::report::Report;

/// A statically-composed collection of references to [`Report`] implementors.
///
/// `T` is a nested tuple of references — e.g. `(&Room, (&Socket, ()))`.
pub struct Reporter<T> {
    items: T,
}

impl Reporter<()> {
    /// Creates an empty reporter.
    pub fn new() -> Self {
        Self { items: () }
    }
}

impl Default for Reporter<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Reporter<T> {
    /// Adds another item to the reporter.
    ///
    /// The returned `Reporter` has a different concrete type, enabling
    /// heterogeneous collections without trait objects.
    #[allow(clippy::should_implement_trait)]
    pub fn add<U: Report>(self, item: &U) -> Reporter<(&U, T)> {
        Reporter {
            items: (item, self.items),
        }
    }
}

// `ReportAll` is `pub(crate)`, so this impl block is only usable when `T`
// satisfies the bound — which callers from outside the crate cannot name.
// The `#[allow]` attributes are intentional:
//   - `private_bounds`   — `ReportAll` is a pub(crate) trait exposed in a
//     public impl (the bound is deliberately hidden from the public API).
//   - `unknown_lints`    — guard for future / older compilers where
//     `private_bounds` may not yet be recognised (forward-compat).
#[allow(unknown_lints)]
#[allow(private_bounds)]
impl<T: ReportAll> Reporter<T> {
    /// Produces a combined report string from all added items.
    pub fn report(&self) -> String {
        self.items.report_all()
    }
}

// ── ReportAll — recursive trait for nested tuples ─────────────────────────────

/// Helper trait for recursively reporting through nested tuples.
pub(crate) trait ReportAll {
    fn report_all(&self) -> String;
}

/// Base case: empty tuple.
impl ReportAll for () {
    fn report_all(&self) -> String {
        String::new()
    }
}

/// Recursive case: a reference to a head item plus a tail tuple.
impl<Head: Report, Tail: ReportAll> ReportAll for (&Head, Tail) {
    fn report_all(&self) -> String {
        let (head, tail) = self;
        format!("{}{}", head.report(), tail.report_all())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devices::{Socket, Thermometer};
    use crate::room::Room;
    use crate::smart_device::SmartDevice;

    #[test]
    fn test_reporter_single_item() {
        let socket = Socket::new("Lamp", 60.0);
        let r = Reporter::new().add(&socket).report();
        assert!(r.contains("Lamp"));
    }

    #[test]
    fn test_reporter_multiple_items() {
        let room = Room::new("Kitchen");
        let socket = Socket::new("Lamp", 60.0);
        let thermo = Thermometer::new("Sensor", 22.5);
        let device: SmartDevice = Thermometer::default().into();

        let r = Reporter::new()
            .add(&room)
            .add(&socket)
            .add(&thermo)
            .add(&device)
            .report();

        assert!(r.contains("Kitchen"));
        assert!(r.contains("Lamp"));
        assert!(r.contains("Sensor"));
        assert!(r.contains("22.5"));
    }

    #[test]
    fn test_reporter_empty() {
        let r = Reporter::new().report();
        assert!(r.is_empty());
    }
}
