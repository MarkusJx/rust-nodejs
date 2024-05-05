use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use nodejs::neon::result::NeonResult;
use nodejs::neon::types::{JsArray, JsString};
use nodejs::neon::{context::Context, reflect::eval, types::JsNumber};

#[chazi::test(check_reach)]
fn test_simple() {
    let mut answer = 0;
    let res = unsafe {
        nodejs::raw::run_neon(|mut cx| {
            let script = cx.string("40+2");
            let forty_two = eval(&mut cx, script)?;
            answer = forty_two
                .downcast_or_throw::<JsNumber, _>(&mut cx)?
                .value(&mut cx) as _;
            Ok(())
        })
    };
    assert_eq!(answer, 42);
    assert!(res.is_ok());
    chazi::reached::last()
}

#[chazi::test(check_reach)]
fn test_simple_safe() {
    let mut answer = 0;
    let res = nodejs::run_neon(|mut cx| {
        let script = cx.string("40+2");
        let forty_two = eval(&mut cx, script)?;
        answer = forty_two
            .downcast_or_throw::<JsNumber, _>(&mut cx)?
            .value(&mut cx) as _;
        Ok(())
    });

    assert_eq!(answer, 42);
    assert!(res.is_ok());
    chazi::reached::last()
}

#[chazi::test(check_reach)]
fn test_simple_napi() {
    let mut answer = 0;
    let res = unsafe {
        nodejs::raw::run_napi(|env| {
            let res: napi::JsNumber = env.run_script("40+2")?;
            answer = res.get_int32()?;

            Ok(())
        })
    };

    assert_eq!(answer, 42);
    assert!(res.is_ok());
    chazi::reached::last()
}

#[chazi::test(check_reach)]
fn test_simple_napi_safe() {
    let mut answer = 0;
    let res = nodejs::run_napi(|env| {
        let res: napi::JsNumber = env.run_script("40+2")?;
        answer = res.get_int32()?;

        Ok(())
    });

    assert_eq!(answer, 42);
    assert!(res.is_ok());
    chazi::reached::last()
}

#[chazi::test(check_reach)]
fn test_process_exit_nonzero() {
    let res = unsafe {
        nodejs::raw::run_neon(|mut cx| {
            let script = cx.string("process.exit(40+2)");
            eval(&mut cx, script)?;
            Ok(())
        })
    };

    assert!(res.is_err());
    assert_eq!(res.err().unwrap().code(), 42);
    chazi::reached::last()
}

#[chazi::test(check_reach)]
fn test_process_exit() {
    let res = unsafe {
        nodejs::raw::run_neon(|mut cx| {
            let script = cx.string("process.exit()");
            eval(&mut cx, script)?;
            Ok(())
        })
    };

    assert!(res.is_ok());
    chazi::reached::last()
}

#[chazi::test(check_reach)]
fn test_argv() {
    let mut args = Vec::<String>::new();
    let res = unsafe {
        nodejs::raw::run_neon(|mut cx| {
            let script = cx.string("[process.argv0, ...process.argv.slice(1)]");
            let process_args = eval(&mut cx, script)?;
            let process_args = process_args
                .downcast_or_throw::<JsArray, _>(&mut cx)?
                .to_vec(&mut cx)?;
            args = process_args
                .iter()
                .map(|arg| {
                    Ok(arg
                        .downcast_or_throw::<JsString, _>(&mut cx)?
                        .value(&mut cx))
                })
                .collect::<NeonResult<Vec<String>>>()?;
            Ok(())
        })
    };
    assert_eq!(args, std::env::args().collect::<Vec<String>>());
    assert!(res.is_ok());
    chazi::reached::last()
}

#[chazi::test(check_reach)]
fn test_uncaught_error() {
    let res = unsafe {
        nodejs::raw::run_neon(|mut cx| {
            let script = cx.string("setImmediate(() => throw new Error())");
            eval(&mut cx, script)?;
            Ok(())
        })
    };

    assert!(res.is_err());
    assert_eq!(res.err().unwrap().code(), 1);
    chazi::reached::last()
}

#[chazi::test(check_reach)]
fn test_uncaught_error_napi() {
    let res = unsafe {
        nodejs::raw::run_napi(|env| {
            env.run_script("setImmediate(() => throw new Error())")?;

            Ok(())
        })
    };

    assert!(res.is_err());
    assert_eq!(res.err().unwrap().code(), 1);
    chazi::reached::last()
}

#[chazi::test(check_reach)]
fn test_manual_stop() {
    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();
    std::thread::spawn(move || {
        let mut lock = result_clone.lock().unwrap();
        let res = unsafe {
            nodejs::raw::run_neon(|mut cx| {
                let script = cx.string("setInterval(() => {}, 1000)");
                eval(&mut cx, script)?;

                Ok(())
            })
        };

        lock.replace(res);
    });

    std::thread::sleep(Duration::from_secs(1));
    let code = unsafe { nodejs::raw::stop() };
    assert!(code.is_ok(), "{}", code.err().unwrap());

    let lock = result.lock().unwrap();
    let res = lock.as_ref().unwrap();
    assert!(res.is_ok(), "{}", res.as_ref().err().unwrap());

    chazi::reached::last()
}
