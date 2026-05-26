//! Smart socket library with C ABI.
//!
//! Provides an opaque-handle C API for creating and controlling a smart socket.
//! All handles are exposed as `*mut c_void` / `*const c_void` — the concrete
//! `Socket` type is never visible across the C boundary.
//!
//! Build artefacts:
//! - `libsocket_c.rlib` — Rust library (for Rust consumers)
//! - `libsocket_c.a`   — static C library
//! - `libsocket_c.so`  — dynamic C library
//!
//! # Thread safety
//!
//! The socket handle (`*mut c_void`) is **not** thread-safe.  Concurrent
//! access from multiple threads without external synchronization (e.g. a
//! mutex) is undefined behavior.  The caller is responsible for ensuring
//! that only one thread operates on a given handle at any time.
#![warn(missing_docs)]

use std::borrow::Cow;
use std::ffi::{c_char, c_int, c_void, CStr, CString};

/// Opaque handle representing a smart socket.
///
/// Created via [`socket_new`], destroyed via [`socket_free`].
///
/// `#[repr(C)]` is not needed — this type is only accessed through opaque
/// `*mut c_void` / `*const c_void` handles across the C ABI boundary and
/// is never passed by value.
pub struct Socket {
    name: CString,
    is_on: bool,
    power_watts: f32,
}

impl Socket {
    /// Creates a new socket in the **off** state.
    ///
    /// Returns `None` if `name` contains a null byte (`\0`) or if `power_watts`
    /// is negative or NaN.  While a `&str` is always valid UTF-8, Rust strings
    /// may legally contain embedded nulls, which are invalid in C strings.
    ///
    /// For C consumers the validation is performed by [`socket_new`] before
    /// this constructor is ever reached.
    pub fn new(name: &str, power_watts: f32) -> Option<Self> {
        if power_watts.is_nan() || power_watts < 0.0 {
            return None;
        }
        let name = CString::new(name).ok()?;
        Some(Self {
            name,
            is_on: false,
            power_watts,
        })
    }

    /// Turns the socket on.
    pub fn turn_on(&mut self) {
        self.is_on = true;
    }

    /// Turns the socket off.
    pub fn turn_off(&mut self) {
        self.is_on = false;
    }

    /// Returns `true` if the socket is currently on.
    pub fn is_on(&self) -> bool {
        self.is_on
    }

    /// Returns the current power draw in watts (0.0 when off).
    pub fn power(&self) -> f32 {
        if self.is_on {
            self.power_watts
        } else {
            0.0
        }
    }

    /// Returns the socket name, replacing invalid UTF-8 with `U+FFFD`.
    ///
    /// Non-UTF-8 names can occur when the socket was created via the C ABI
    /// function [`socket_new`] with a byte string in a non-UTF-8 encoding
    /// (e.g. Latin-1, CP1251).  For the raw C string, use [`Socket::name_cstr`].
    pub fn name(&self) -> Cow<'_, str> {
        self.name.to_string_lossy()
    }

    /// Returns the socket name as a `CStr` reference.
    ///
    /// This is the same pointer returned by the C ABI function [`socket_name`].
    pub fn name_cstr(&self) -> &CStr {
        &self.name
    }
}

// ── errno helper ────────────────────────────────────────────────────────────

/// Invalid argument (POSIX `EINVAL`).
const EINVAL: i32 = 22;

/// Sets the C `errno` to `err` via the glibc `__errno_location()` intrinsic.
///
/// On non-Linux platforms (macOS, FreeBSD, …) this is a **no-op** — `errno` is
/// not modified.  C consumers on those platforms must check the function return
/// value and cannot rely on `errno` for error details.  Adding portable `errno`
/// support via the `libc` crate is tracked separately.
#[cfg(target_os = "linux")]
fn set_errno(err: i32) {
    extern "C" {
        fn __errno_location() -> *mut i32;
    }
    unsafe {
        *__errno_location() = err;
    }
}

#[cfg(not(target_os = "linux"))]
fn set_errno(_err: i32) {}

// ── C ABI (opaque `*mut c_void` handles) ─────────────────────────────────────

/// Creates a new socket. Returns null on invalid arguments.
///
/// The socket starts in the **off** state.  `power_watts` is the nominal power
/// draw when the socket is on (must be ≥ 0).
///
/// Returns an opaque `*mut c_void` handle.
///
/// # Safety
///
/// `name` must be either null or a valid null-terminated C string.
/// The returned handle must be passed to [`socket_free`] exactly once to avoid
/// leaks.
#[no_mangle]
pub unsafe extern "C" fn socket_new(name: *const c_char, power_watts: f32) -> *mut c_void {
    if name.is_null() || power_watts.is_nan() || power_watts < 0.0 {
        set_errno(EINVAL);
        return std::ptr::null_mut();
    }
    // SAFETY: `name` is non-null and, per function contract, points to a
    // valid null-terminated C string with no interior null bytes.
    let name = unsafe { CStr::from_ptr(name) }.to_owned();
    Box::into_raw(Box::new(Socket {
        name,
        is_on: false,
        power_watts,
    })) as *mut c_void
}

/// Frees a socket previously created with [`socket_new`].
///
/// Passing null is a safe no-op.
///
/// # Safety
///
/// `s` must be either null or a valid handle returned by [`socket_new`] that
/// has not yet been freed.
#[no_mangle]
pub unsafe extern "C" fn socket_free(s: *mut c_void) {
    if s.is_null() {
        return;
    }
    // SAFETY: `s` is non-null, was created by `Box::into_raw` in
    // `socket_new`, has the correct alignment for `Socket`, and has not
    // been freed before.
    drop(Box::from_raw(s as *mut Socket));
}

/// Turns the socket on. Returns 0 on success, -1 if `s` is null.
///
/// # Safety
///
/// `s` must be either null or a valid handle returned by [`socket_new`] that
/// has not yet been freed.
#[no_mangle]
pub unsafe extern "C" fn socket_turn_on(s: *mut c_void) -> c_int {
    if s.is_null() {
        set_errno(EINVAL);
        return -1;
    }
    // SAFETY: `s` is non-null, was created by `Box::into_raw` in
    // `socket_new`, has correct alignment for `Socket`, and has not been
    // freed before.
    (*(s as *mut Socket)).turn_on();
    0
}

/// Turns the socket off. Returns 0 on success, -1 if `s` is null.
///
/// # Safety
///
/// `s` must be either null or a valid handle returned by [`socket_new`] that
/// has not yet been freed.
#[no_mangle]
pub unsafe extern "C" fn socket_turn_off(s: *mut c_void) -> c_int {
    if s.is_null() {
        set_errno(EINVAL);
        return -1;
    }
    // SAFETY: `s` is non-null, was created by `Box::into_raw` in
    // `socket_new`, has correct alignment for `Socket`, and has not been
    // freed before.
    (*(s as *mut Socket)).turn_off();
    0
}

/// Queries whether the socket is on.
///
/// Writes `1` into `*out` if the socket is on, `0` if off.
/// Returns 0 on success, -1 if `s` or `out` is null.
///
/// This separates the error indicator from the state value, so C code like
/// `if (*out) { ... }` is unambiguous.
///
/// # Safety
///
/// `s` must be either null or a valid handle returned by [`socket_new`] that
/// has not yet been freed.  `out` must be null or a valid pointer to a `c_int`
/// that can be written to.
#[no_mangle]
pub unsafe extern "C" fn socket_is_on(s: *const c_void, out: *mut c_int) -> c_int {
    if s.is_null() || out.is_null() {
        set_errno(EINVAL);
        return -1;
    }
    // SAFETY: `s` and `out` are non-null. `s` was created by
    // `Box::into_raw` in `socket_new`, has correct alignment, and has not
    // been freed. `out` points to a writable `c_int`.
    *out = (*(s as *const Socket)).is_on() as c_int;
    0
}

/// Writes the current power draw in watts into `*out` (0.0 when off).
///
/// Returns 0 on success, -1 if either `s` or `out` is null.
///
/// # Safety
///
/// `s` must be either null or a valid handle returned by [`socket_new`] that
/// has not yet been freed.  `out` must be null or a valid pointer to an `f32`
/// that can be written to.
#[no_mangle]
pub unsafe extern "C" fn socket_power(s: *const c_void, out: *mut f32) -> c_int {
    if s.is_null() || out.is_null() {
        set_errno(EINVAL);
        return -1;
    }
    // SAFETY: `s` and `out` are non-null. `s` was created by
    // `Box::into_raw` in `socket_new`, has correct alignment, and has not
    // been freed. `out` points to a writable `f32`.
    *out = (*(s as *const Socket)).power();
    0
}

/// Returns a pointer to the null-terminated socket name.
///
/// The pointer is valid until the socket is freed.  The caller **must not**
/// free the returned pointer — it is owned by the socket.  Returns null if
/// `s` is null.
///
/// # Safety
///
/// `s` must be either null or a valid handle returned by [`socket_new`] that
/// has not yet been freed.
#[no_mangle]
pub unsafe extern "C" fn socket_name(s: *const c_void) -> *const c_char {
    if s.is_null() {
        set_errno(EINVAL);
        return std::ptr::null();
    }
    // SAFETY: `s` is non-null, was created by `Box::into_raw` in
    // `socket_new`, has correct alignment, and has not been freed. The
    // returned pointer references the `name` field of the `Socket`, which
    // is live as long as the socket is not freed.
    (*(s as *const Socket)).name.as_ptr()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn create_and_free() {
        let name = CString::new("TestSocket").unwrap();
        let s = unsafe { socket_new(name.as_ptr(), 100.0) };
        assert!(!s.is_null());
        let sock = s as *mut Socket;
        // Verify C ABI and Rust API return consistent results for initial state.
        assert!(!unsafe { (*sock).is_on() });
        let mut is_on: c_int = -1;
        assert_eq!(unsafe { socket_is_on(s, &mut is_on) }, 0);
        assert_eq!(is_on, 0);
        unsafe { socket_free(s) };
    }

    #[test]
    fn null_name_returns_null() {
        let s = unsafe { socket_new(std::ptr::null(), 100.0) };
        assert!(s.is_null());
    }

    #[test]
    fn nan_power_returns_null() {
        let name = CString::new("NaN Test").unwrap();
        let s = unsafe { socket_new(name.as_ptr(), f32::NAN) };
        assert!(s.is_null());
    }

    #[test]
    fn negative_power_returns_null() {
        let name = CString::new("Bad").unwrap();
        let s = unsafe { socket_new(name.as_ptr(), -1.0) };
        assert!(s.is_null());
    }

    #[test]
    fn free_null_is_noop() {
        unsafe { socket_free(std::ptr::null_mut()) };
    }

    #[test]
    fn turn_on_off() {
        let name = CString::new("Kettle").unwrap();
        let s = unsafe { socket_new(name.as_ptr(), 2000.0) };

        let mut power: f32 = -1.0;

        let mut is_on: c_int = -1;
        assert_eq!(unsafe { socket_is_on(s, &mut is_on) }, 0);
        assert_eq!(is_on, 0);
        assert_eq!(unsafe { socket_power(s, &mut power) }, 0);
        assert_eq!(power, 0.0);

        assert_eq!(unsafe { socket_turn_on(s) }, 0);
        assert_eq!(unsafe { socket_is_on(s, &mut is_on) }, 0);
        assert_eq!(is_on, 1);
        assert_eq!(unsafe { socket_power(s, &mut power) }, 0);
        assert_eq!(power, 2000.0);

        assert_eq!(unsafe { socket_turn_off(s) }, 0);
        assert_eq!(unsafe { socket_is_on(s, &mut is_on) }, 0);
        assert_eq!(is_on, 0);
        assert_eq!(unsafe { socket_power(s, &mut power) }, 0);
        assert_eq!(power, 0.0);

        unsafe { socket_free(s) };
    }

    #[test]
    fn name() {
        let name = CString::new("Desk Lamp").unwrap();
        let s = unsafe { socket_new(name.as_ptr(), 60.0) };

        let name_ptr = unsafe { socket_name(s) };
        assert!(!name_ptr.is_null());
        let returned = unsafe { CStr::from_ptr(name_ptr) }.to_str().unwrap();
        assert_eq!(returned, "Desk Lamp");

        unsafe { socket_free(s) };
    }

    #[test]
    fn non_utf8_name_via_c_abi() {
        // Create a socket via C ABI with a name containing invalid UTF-8
        // bytes (but no interior null — C strings can't contain those).
        let raw = b"K\xFFtchen";
        let name_cstr = CString::new(raw.as_ref()).unwrap();
        let s = unsafe { socket_new(name_cstr.as_ptr(), 100.0) };
        assert!(!s.is_null());

        // C ABI function returns the raw bytes — caller is responsible.
        let name_ptr = unsafe { socket_name(s) };
        assert!(!name_ptr.is_null());
        let cstr = unsafe { CStr::from_ptr(name_ptr) };
        // Verify the exact bytes are preserved.
        assert_eq!(cstr.to_bytes(), b"K\xFFtchen");

        // Rust `name()` replaces invalid UTF-8 with U+FFFD.
        let sock = s as *mut Socket;
        let name = unsafe { (*sock).name() };
        assert!(name.contains('\u{FFFD}'));

        unsafe { socket_free(s) };
    }

    #[test]
    fn null_pointer_handling() {
        let mut power: f32 = 0.0;
        let mut is_on: c_int = -1;

        assert_eq!(unsafe { socket_turn_on(std::ptr::null_mut()) }, -1);
        assert_eq!(unsafe { socket_turn_off(std::ptr::null_mut()) }, -1);
        assert_eq!(unsafe { socket_is_on(std::ptr::null(), &mut is_on) }, -1);
        assert_eq!(unsafe { socket_power(std::ptr::null(), &mut power) }, -1);
        assert_eq!(
            unsafe { socket_power(std::ptr::null(), std::ptr::null_mut()) },
            -1
        );
        assert!(unsafe { socket_name(std::ptr::null()) }.is_null());
    }

    #[test]
    fn power_valid_socket_null_out() {
        let name = CString::new("Valid").unwrap();
        let s = unsafe { socket_new(name.as_ptr(), 100.0) };
        assert!(!s.is_null());
        // Valid socket handle, but null output pointer — should fail gracefully.
        assert_eq!(unsafe { socket_power(s, std::ptr::null_mut()) }, -1);
        unsafe { socket_free(s) };
    }
}
