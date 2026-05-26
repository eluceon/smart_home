//! Demo application that **dynamically loads** the C ABI socket library at runtime.
//!
//! Requires `libsocket_c.so` to be present (built by `cargo build -p socket-c`).
//!
//! Build: `cargo build -p app-dynamic`
//! Run:   `cargo run -p app-dynamic`
//!
//! ## Library search order
//!
//! 1. `SOCKET_C_LIB` environment variable (exact path)
//! 2. Directory of the running executable
//! 3. `target/debug/` and `target/release/` relative to workspace root
//! 4. `libsocket_c.so` in current directory
//! 5. System paths (`/usr/lib/`, `/usr/local/lib/`)

use libloading::{Library, Symbol};
use std::error::Error;
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::path::PathBuf;

type SocketNew = unsafe extern "C" fn(*const c_char, f32) -> *mut c_void;
type SocketFree = unsafe extern "C" fn(*mut c_void);
type SocketTurnOn = unsafe extern "C" fn(*mut c_void) -> c_int;
type SocketTurnOff = unsafe extern "C" fn(*mut c_void) -> c_int;
type SocketIsOn = unsafe extern "C" fn(*const c_void, out: *mut c_int) -> c_int;
type SocketPower = unsafe extern "C" fn(*const c_void, out: *mut f32) -> c_int;
type SocketName = unsafe extern "C" fn(*const c_void) -> *const c_char;

/// Loads a symbol from the library.
///
/// # Safety
///
/// `lib` must be a valid `Library` handle.  `T` must match the function
/// signature of the symbol identified by `name`.
unsafe fn load_symbol<'lib, T>(
    lib: &'lib Library,
    name: &[u8],
) -> Result<Symbol<'lib, T>, Box<dyn Error>> {
    lib.get(name)
        .map_err(|e| format!("symbol '{}' not found: {e}", String::from_utf8_lossy(name)).into())
}

/// Locates `libsocket_c.so` by checking (in order):
///
/// 1. `SOCKET_C_LIB` environment variable
/// 2. Directory of the running executable (covers `cargo run`)
/// 3. One level above the executable (covers `target/debug/examples/` layout)
/// 4. Current directory, then system paths
///
/// All paths are resolved at runtime — no compile-time paths are embedded.
fn find_lib() -> PathBuf {
    if let Ok(p) = std::env::var("SOCKET_C_LIB") {
        let path = PathBuf::from(&p);
        if path.exists() {
            return path;
        }
        eprintln!("warning: SOCKET_C_LIB='{p}' does not exist, falling back to default search");
    }

    // Search relative to the running executable.
    if let Ok(exe) = std::env::current_exe() {
        let exe_dir = exe.parent().map(|d| d.to_path_buf()).unwrap_or_default();

        // Same directory as the executable (e.g. `target/debug/`).
        let next_to_exe = exe_dir.join("libsocket_c.so");
        if next_to_exe.exists() {
            return next_to_exe;
        }

        // One level up (e.g. from `target/debug/examples/` → `target/debug/`).
        if let Some(parent) = exe_dir.parent() {
            let one_up = parent.join("libsocket_c.so");
            if one_up.exists() {
                return one_up;
            }
        }
    }

    // Fallback candidates.
    let candidates = [
        PathBuf::from("libsocket_c.so"),
        PathBuf::from("/usr/lib/libsocket_c.so"),
        PathBuf::from("/usr/local/lib/libsocket_c.so"),
    ];

    for cand in &candidates {
        if cand.exists() {
            return cand.clone();
        }
    }

    // Return the first fallback as the default to try; let libloading report
    // the actual error.
    candidates[0].clone()
}

fn check_rc(rc: c_int, fn_name: &str) -> Result<(), Box<dyn Error>> {
    if rc != 0 {
        return Err(format!("{fn_name} failed with code {rc}").into());
    }
    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    println!("=== Dynamic loading demo (libloading) ===\n");

    let lib_path = find_lib();
    println!("Loading library: {}", lib_path.display());

    // Library::new defaults to RTLD_LAZY | RTLD_LOCAL on Linux.
    //
    // SAFETY: The library at `lib_path` is libsocket_c.so built by the same
    // Cargo workspace with a known, trusted set of exported symbols.
    let lib = unsafe { Library::new(&lib_path) }.map_err(|e| {
        format!(
            "Failed to load {}: {e}\n\
             Hint: build the library first with `cargo build -p socket-c`",
            lib_path.display()
        )
    })?;

    // SAFETY: Each symbol name matches a `#[no_mangle]` function exported by
    // socket-c.  The type annotations match the function signatures 1:1.
    let socket_new: Symbol<SocketNew> = unsafe { load_symbol(&lib, b"socket_new") }?;
    let socket_free: Symbol<SocketFree> = unsafe { load_symbol(&lib, b"socket_free") }?;
    let socket_turn_on: Symbol<SocketTurnOn> = unsafe { load_symbol(&lib, b"socket_turn_on") }?;
    let socket_turn_off: Symbol<SocketTurnOff> = unsafe { load_symbol(&lib, b"socket_turn_off") }?;
    let socket_is_on: Symbol<SocketIsOn> = unsafe { load_symbol(&lib, b"socket_is_on") }?;
    let socket_power: Symbol<SocketPower> = unsafe { load_symbol(&lib, b"socket_power") }?;
    let socket_name: Symbol<SocketName> = unsafe { load_symbol(&lib, b"socket_name") }?;

    println!("All symbols resolved successfully.\n");

    // ── Create a socket ────────────────────────────────────────────────────
    let name = CString::new("Air conditioner").expect("no null bytes");
    // SAFETY: `name.as_ptr()` is a valid null-terminated C string. The
    // returned handle lives until `socket_free` is called below.
    let s: *mut c_void = unsafe { socket_new(name.as_ptr(), 2500.0) };
    if s.is_null() {
        return Err("Failed to create socket".into());
    }
    println!("Created socket via dynamically-loaded C ABI.");

    // ── Inspect initial state ──────────────────────────────────────────────
    // SAFETY: `s` is a valid, non-null socket handle.
    let name_ptr = unsafe { socket_name(s) };
    // SAFETY: `name_ptr` is a null-terminated C string owned by the socket,
    // valid until `socket_free(s)` below.
    let rname = unsafe { CStr::from_ptr(name_ptr) }.to_string_lossy();
    let mut power: f32 = 0.0;
    let mut is_on: c_int = 0;
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
    // SAFETY: `s` was returned by `socket_new`, has not been freed, and is
    // never used again after this call.
    unsafe { socket_free(s) };
    println!("\nSocket freed. Library auto-unloaded when it goes out of scope. Done.");
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
    }
    // `run()` returns normally — destructors run, `Library` is properly dropped.
}
