#[cfg(feature = "neon")]
use neon;
#[cfg(feature = "neon")]
use neon::context::ModuleContext;
#[cfg(feature = "neon")]
use neon::result::NeonResult;
#[cfg(feature = "napi")]
use napi::bindgen_prelude::napi_register_module_v1;
#[cfg(feature = "napi")]
use napi::sys::{napi_env, napi_value};
#[cfg(feature = "napi")]
use napi::Env;

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
#[cfg(feature = "napi")]
use napi::JsError;

use crate::error::NodeError;
use crate::sys;

/// Starts a Node.js instance and immediately run the provided N-API module init function.
/// Blocks until the event loop stops, and returns the exit code.
///
/// # Safety
/// This function can only be called at most once.
pub unsafe fn run_raw(napi_reg_func: *mut std::os::raw::c_void) -> crate::Result<()> {
    let args: Vec<CString> = std::env::args()
        .map(|arg| CString::new(arg).unwrap_or_default())
        .collect();
    let mut argc_c = Vec::<*const c_char>::with_capacity(args.len());
    for arg in &args {
        argc_c.push(arg.as_ptr() as *const c_char)
    }

    let result = sys::node_run(sys::node_options_t {
        process_argc: argc_c.len() as c_int,
        process_argv: argc_c.as_ptr(),
        napi_reg_func,
    });

    if !result.error.is_null() {
        let result_error_string = CString::from(CStr::from_ptr(result.error));
        libc::free(result.error as _);

        Err(NodeError::new(result_error_string.to_string_lossy().into_owned(), result.exit_code as i32))
    } else if result.exit_code != 0 {
        Err(NodeError::new("Node.js exited with a non-zero exit code".to_string(), result.exit_code as i32))
    } else {
        Ok(())
    }
}

/// Stops the running Node.js instance.
/// Returns an error if Node.js is not running.
/// Returns Ok(()) if Node.js is stopped successfully.
/// 
/// # Safety
/// This function should be safe as long as it is called after [`run_raw`] or [`run_neon`].
pub unsafe fn stop() -> crate::Result<()> {
    let code = sys::node_stop();
    if code != 0 {
        let error_str = if code == -1 {
            ": Node.js is not running"
        } else {
            ""
        };

        Err(NodeError::new(format!("Node.js failed to stop{error_str}"), code as i32))
    } else {
        Ok(())
    }
}

/// Starts a Node.js instance and immediately run the provided N-API module init function.
/// Blocks until the event loop stops, and returns the exit code.
///
/// # Safety
/// This function can only be called at most once.
#[cfg(feature = "neon")]
pub unsafe fn run_neon<F: for<'a> FnOnce(ModuleContext<'a>) -> NeonResult<()>>(f: F) -> crate::Result<()> {
    use std::ptr::null_mut;
    use std::sync::Once;
    static mut MODULE_INIT_FN: *mut std::ffi::c_void = null_mut(); // *mut Option<F>

    let mut module_init_fn = Some(f);
    MODULE_INIT_FN = (&mut module_init_fn) as *mut Option<F> as _;

    unsafe extern "C" fn napi_reg_func<F: for<'a> FnOnce(ModuleContext<'a>) -> NeonResult<()>>(
        env: neon::macro_internal::runtime::raw::Env,
        m: neon::macro_internal::runtime::raw::Local,
    ) -> neon::macro_internal::runtime::raw::Local {
        neon::macro_internal::initialize_module(env, std::mem::transmute(m), |ctx| {
            static ONCE: Once = Once::new();
            let mut result = Ok(());
            ONCE.call_once(|| {
                let module_init_fn = (MODULE_INIT_FN as *mut Option<F>).as_mut().unwrap();
                let module_init_fn = module_init_fn.take().unwrap();
                MODULE_INIT_FN = null_mut();
                result = module_init_fn(ctx)
            });
            result
        });
        m
    }

    run_raw(napi_reg_func::<F> as _)
}

#[cfg(feature = "napi")]
pub unsafe fn run_napi<F: FnOnce(Env) -> napi::Result<()>>(f: F) -> crate::Result<()> {
    use std::ptr::null_mut;
    use std::sync::Once;

    static mut MODULE_INIT_FN: *mut std::os::raw::c_void = null_mut(); // *mut Option<F>

    let mut module_init_fn = Some(f);
    MODULE_INIT_FN = (&mut module_init_fn) as *mut Option<F> as _;

    unsafe extern "C" fn napi_reg_func<F: FnOnce(Env) -> napi::Result<()>>(
        env: napi_env,
        exports: napi_value,
    ) -> napi_value {
        static ONCE: Once = Once::new();
        ONCE.call_once(|| {
            napi_register_module_v1(env, exports);
        });

        let module_init_fn = (MODULE_INIT_FN as *mut Option<F>).as_mut().unwrap();
        let module_init_fn = module_init_fn.take().unwrap();
        MODULE_INIT_FN = null_mut();
        let env = Env::from_raw(env);
        let res = module_init_fn(env);
        res.unwrap_or_else(|err| {
            let _ = env.throw(JsError::from(err).into_unknown(env));
        });

        exports
    }

    run_raw(napi_reg_func::<F> as _)
}