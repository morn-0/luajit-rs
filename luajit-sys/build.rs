use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    let embed = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("embed");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    let luajit = out.join("luajit");
    if luajit.exists() {
        fs::remove_dir_all(&luajit).unwrap();
    }

    copy_dir::copy_dir(embed.join("luajit"), &luajit).unwrap();

    let host = env::var("HOST").unwrap();
    let target = env::var("TARGET").unwrap();

    let mut make = Command::new("make");
    make.current_dir(luajit.join("src"));
    make.arg("-e");

    if target.contains("linux") {
        make.env("TARGET_SYS", "Linux");
    }

    if target.contains("windows") {
        make.env("TARGET_SYS", "Windows");
    }

    let target_pointer_width = env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap();
    if target_pointer_width == "32" && env::var_os("HOST_CC").is_none() {
        let host_cc = cc::Build::new().target(&host).get_compiler();
        make.env("HOST_CC", format!("{} -m32", host_cc.path().display()));
    }

    make.env("BUILDMODE", "static");
    make.env("XCFLAGS", "-fPIC -DLUAJIT_ENABLE_LUA52COMPAT");

    make.status().unwrap();

    let src = luajit.join("src");
    println!("cargo:rustc-link-search=native={}", src.display());
    println!("cargo:rustc-link-lib=static=luajit");

    bindgen::builder()
        .header(src.join("lauxlib.h").to_string_lossy())
        .header(src.join("lua.h").to_string_lossy())
        .header(src.join("luaconf.h").to_string_lossy())
        .header(src.join("luajit.h").to_string_lossy())
        .header(src.join("lualib.h").to_string_lossy())
        .generate()
        .unwrap()
        .write_to_file(out.join("bindings.rs"))
        .unwrap();
}
