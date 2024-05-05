#![doc = include_str!("../README.md")]

pub mod error;
pub mod raw;
mod sys;

#[cfg(feature = "napi")]
use napi::Env;
#[cfg(feature = "neon")]
use neon::context::ModuleContext;
#[cfg(feature = "neon")]
use neon::result::NeonResult;

pub use crate::error::Result;
#[cfg(feature = "neon")]
pub use neon;

static NODE_EXECUTED: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

#[cfg(any(feature = "neon", feature = "napi"))]
fn run_inner<F: FnOnce() -> Result<()>>(f: F) -> Result<()> {
    let mut executed = NODE_EXECUTED
        .lock()
        .map_err(|_| error::NodeError::new("Mutex lock failed".to_string(), 1))?;

    if *executed {
        Err(error::NodeError::new(
            "Node.js is already running".to_string(),
            1,
        ))
    } else {
        *executed = true;
        f()
    }
}

#[cfg(feature = "neon")]
pub fn run_neon<F: for<'a> FnOnce(ModuleContext<'a>) -> NeonResult<()>>(f: F) -> Result<()> {
    run_inner(|| unsafe { raw::run_neon(f) })
}

#[cfg(feature = "napi")]
pub fn run_napi<F: FnOnce(Env) -> napi::Result<()>>(f: F) -> Result<()> {
    run_inner(|| unsafe { raw::run_napi(f) })
}
