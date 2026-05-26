//! Demo application that **statically links** the C ABI socket library.
//!
//! The `socket-c` crate is a Cargo dependency.  Its code is compiled into this
//! binary — no `libsocket_c.so` is needed at runtime.  Verify with:
//! ```bash
//! ldd target/debug/app-static | grep socket_c   # should return nothing
//! ```
//!
//! All operations use the C ABI functions directly.  The concrete `Socket` type
//! is imported for the cast from `*mut c_void`, but its fields are never accessed
//! — this keeps the code compatible with a pure-C consumer.

use std::ffi::{c_int, c_void, CStr, CString};

use socket_c::{
    socket_free, socket_is_on, socket_name, socket_new, socket_power, socket_turn_off,
    socket_turn_on,
};

fn check_rc(rc: c_int, fn_name: &str) -> Result<(), String> {
    if rc != 0 {
        return Err(format!("{fn_name} failed with code {rc}"));
    }
    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Static linking demo ===\n");

    // ── Create a socket ────────────────────────────────────────────────────
    let name = CString::new("Coffee machine").expect("no null bytes");
    // SAFETY: `name.as_ptr()` points to a valid null-terminated C string.
    // `socket_new` copies the name into the Socket, so `name` may be dropped
    // immediately after this call.
    let s: *mut c_void = unsafe { socket_new(name.as_ptr(), 1200.0) };
    if s.is_null() {
        return Err("Failed to create socket".into());
    }
    println!("Created socket via C ABI (statically linked).");

    // ── Inspect initial state ──────────────────────────────────────────────
    // SAFETY: `s` is a valid, non-null socket handle.
    let name_ptr = unsafe { socket_name(s) };
    // SAFETY: `name_ptr` is a valid null-terminated C string owned by the
    // socket, guaranteed live until `socket_free(s)` below.
    let rname = unsafe { CStr::from_ptr(name_ptr) }.to_string_lossy();
    let mut power: f32 = 0.0;
    let mut is_on: c_int = 0;
    // SAFETY: `s` is a valid handle.
    check_rc(unsafe { socket_power(s, &mut power) }, "socket_power")?;
    check_rc(unsafe { socket_is_on(s, &mut is_on) }, "socket_is_on")?;
    println!(
        "State: name='{rname}', on={}, power={power:.1} W",
        if is_on != 0 { "yes" } else { "no" }
    );

    // ── Turn on ────────────────────────────────────────────────────────────
    // SAFETY: `s` is a valid socket handle.
    let rc = unsafe { socket_turn_on(s) };
    println!("turn_on()  → rc={rc}");
    check_rc(rc, "socket_turn_on")?;
    check_rc(unsafe { socket_power(s, &mut power) }, "socket_power")?;
    check_rc(unsafe { socket_is_on(s, &mut is_on) }, "socket_is_on")?;
    println!("  is_on={is_on}, power={power:.1} W");

    // ── Turn off ───────────────────────────────────────────────────────────
    // SAFETY: `s` is a valid socket handle.
    let rc = unsafe { socket_turn_off(s) };
    println!("turn_off() → rc={rc}");
    check_rc(rc, "socket_turn_off")?;
    check_rc(unsafe { socket_power(s, &mut power) }, "socket_power")?;
    check_rc(unsafe { socket_is_on(s, &mut is_on) }, "socket_is_on")?;
    println!("  is_on={is_on}, power={power:.1} W");

    // ── Clean up ───────────────────────────────────────────────────────────
    // SAFETY: `s` was returned by `socket_new`, has not been freed yet, and
    // is not used after this call.
    unsafe { socket_free(s) };
    println!("\nSocket freed. Done.");
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
    }
}
