//! Security subsystem — filesystem sandbox and permission management.
//!
//! OpenBee is designed to be **absolutely safe** for users. The engine
//! cannot access any directory the user hasn't explicitly approved.
//! All file I/O goes through the [`SandboxedFs`] which validates every
//! path against the allowed directory set before performing any operation.

pub mod sandbox;

pub use sandbox::{
    global_sandbox, init_sandbox, Permission, SandboxError, SandboxResult, SandboxedFs,
};
