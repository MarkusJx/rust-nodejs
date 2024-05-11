use fs_extra::dir::CopyOptions;
use nodejs::neon::{
    context::Context,
    reflect::eval,
    types::{JsFunction, JsNumber, JsString},
};
use std::path::PathBuf;

#[chazi::test(check_reach)]
fn test_require_builtin() {
    let mut script_result = String::new();
    let res = unsafe {
        nodejs::raw::run_neon(
            |mut cx| {
                let script = cx.string("require('http').STATUS_CODES[418]");
                let status_text = eval(&mut cx, script)?;
                script_result = status_text
                    .downcast_or_throw::<JsString, _>(&mut cx)?
                    .value(&mut cx);
                Ok(())
            },
            None,
        )
    };

    assert!(res.is_ok());
    assert_eq!(script_result, "I'm a Teapot");
    chazi::reached::last()
}

#[chazi::test(check_reach, timeout_ms = 600000)]
fn test_require_external_napi() {
    let test_tmpdir = env!("CARGO_TARGET_TMPDIR");
    let napi_module_src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("napi_module");

    let napi_module_installed_dir = PathBuf::from(test_tmpdir)
        .join("napi_module")
        .to_str()
        .unwrap()
        .to_string();

    fs_extra::dir::copy(
        napi_module_src_dir,
        &napi_module_installed_dir,
        &CopyOptions {
            copy_inside: true,
            overwrite: true,
            ..Default::default()
        },
    )
    .unwrap();

    let mut npm_install_cmd = if cfg!(target_os = "windows") {
        let mut cmd = std::process::Command::new("cmd");
        cmd.arg("/c").arg("npm");
        cmd
    } else {
        std::process::Command::new("npm")
    };
    npm_install_cmd
        .current_dir(&napi_module_installed_dir)
        .arg("install");
    if cfg!(target_arch = "x86") {
        npm_install_cmd.arg("--target_arch=ia32");
    }
    let npm_install_status = npm_install_cmd.status().unwrap();
    assert!(npm_install_status.success());

    let mut add_result = 0;
    let res = unsafe {
        nodejs::raw::run_neon(
            |mut cx| {
                let module_path = cx.string(napi_module_installed_dir);
                let js_fn_script = cx.string(
                    "module_path => { \
            const m = require(module_path);\
            return m.add(40, 2);\
        }",
                );
                let js_fn = eval(&mut cx, js_fn_script)?;
                let js_fn = js_fn.downcast_or_throw::<JsFunction, _>(&mut cx)?;
                let js_undefined = cx.undefined();
                let js_fn_result = js_fn.call(&mut cx, js_undefined, [module_path.upcast()])?;
                let js_fn_result = js_fn_result.downcast_or_throw::<JsNumber, _>(&mut cx)?;
                add_result = js_fn_result.value(&mut cx) as _;
                Ok(())
            },
            None,
        )
    };

    assert_eq!(add_result, 42);
    assert!(res.is_ok());
    chazi::reached::last();
}
