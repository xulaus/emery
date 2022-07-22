extern crate bindgen;

use std::{env, error::Error, path::PathBuf, io::Write};
use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn Error>> {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let ruby_script = br#"
        puts RbConfig::CONFIG.values_at("libdir", "rubyhdrdir", "rubyarchhdrdir")
    "#;
    let ruby_out = {
        let mut child = Command::new("ruby")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let child_stdin = child.stdin.as_mut().unwrap();
        child_stdin.write_all(ruby_script)?;
        drop(child_stdin);

        String::from_utf8(child.wait_with_output()?.stdout)?
    };
    let ruby_dirs: Vec<&str> = ruby_out.split('\n').collect();

    println!("cargo:rustc-link-search={}", ruby_dirs[0]);
    println!("cargo:rustc-link-lib=ruby");

    let mut bindings = bindgen::Builder::default()
        .header(format!("{}/ruby.h", ruby_dirs[1]));
    for &dir in ruby_dirs.iter().skip(1) {
        bindings = bindings.clang_arg(format!("-I{dir}"));
    }
     bindings.parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
    Ok(())
}
