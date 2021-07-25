extern crate libc;
use std::{error::Error, ffi::CString, result::Result};

mod ruby {
    pub use libc::uintptr_t as VALUE;
    use libc::{c_double, c_int, c_void};

    pub type CallbackPtr = *const c_void;

    #[link(name = "ruby")]
    extern "C" {
        pub fn rb_define_global_const(name: *const i8, value: VALUE);
        pub fn rb_define_const(parent: VALUE, name: *const i8, value: VALUE);

        pub fn rb_float_new(value: c_double) -> VALUE;
        pub fn rb_num2dbl(value: VALUE) -> c_double;

        pub fn rb_define_module(name: *const i8) -> VALUE;
        pub fn rb_define_module_under(parent: VALUE, name: *const i8) -> VALUE;

        pub fn rb_define_method(parent: VALUE, name: *const i8, func: *const c_void, argc: c_int);
        pub fn rb_define_module_function(
            parent: VALUE,
            name: *const i8,
            func: *const c_void,
            argc: c_int,
        );
        pub fn rb_define_global_function(name: *const i8, func: *const c_void, argc: c_int);
    }
}

pub use ruby::VALUE;

pub trait RubyCallback {
    const ARGC: i32;
    fn as_ruby(&self) -> ruby::CallbackPtr;
}

impl RubyCallback for extern "C" fn() -> VALUE {
    const ARGC: i32 = 0;

    fn as_ruby(&self) -> ruby::CallbackPtr {
        *self as ruby::CallbackPtr
    }
}

impl RubyCallback for extern "C" fn(VALUE) -> VALUE {
    const ARGC: i32 = 1;

    fn as_ruby(&self) -> ruby::CallbackPtr {
        *self as ruby::CallbackPtr
    }
}

#[allow(dead_code)]
pub fn rb_float_new(value: f64) -> VALUE {
    unsafe { ruby::rb_float_new(value) }
}

#[allow(dead_code)]
pub fn rb_define_global_const(name: &str, value: VALUE) -> Result<(), Box<dyn Error>> {
    let c_name = CString::new(name)?;
    unsafe { ruby::rb_define_global_const(c_name.as_ptr(), value) };
    Ok(())
}

#[allow(dead_code)]
pub fn rb_define_const(parent: VALUE, name: &str, value: VALUE) -> Result<(), Box<dyn Error>> {
    let c_name = CString::new(name)?;
    unsafe { ruby::rb_define_const(parent, c_name.as_ptr(), value) };
    Ok(())
}

#[allow(dead_code)]
pub fn rb_num2dbl(value: VALUE) -> f64 {
    unsafe { ruby::rb_num2dbl(value) }
}

#[allow(dead_code)]
pub fn rb_define_module(name: &str) -> Result<VALUE, Box<dyn Error>> {
    let c_name = CString::new(name)?;
    Ok(unsafe { ruby::rb_define_module(c_name.as_ptr()) })
}

#[allow(dead_code)]
pub fn rb_define_module_under(parent: VALUE, name: &str) -> Result<VALUE, Box<dyn Error>> {
    let c_name = CString::new(name)?;
    Ok(unsafe { ruby::rb_define_module_under(parent, c_name.as_ptr()) })
}

#[allow(dead_code)]
pub fn rb_define_method<F: RubyCallback>(
    parent: VALUE,
    name: &str,
    func: F,
) -> Result<(), Box<dyn Error>> {
    let c_name = CString::new(name)?;
    unsafe { ruby::rb_define_method(parent, c_name.as_ptr(), func.as_ruby(), F::ARGC) };
    Ok(())
}

#[allow(dead_code)]
pub fn rb_define_module_function<F: RubyCallback>(
    parent: VALUE,
    name: &str,
    func: F,
) -> Result<(), Box<dyn Error>> {
    let c_name = CString::new(name)?;
    unsafe { ruby::rb_define_module_function(parent, c_name.as_ptr(), func.as_ruby(), F::ARGC) };
    Ok(())
}

#[allow(dead_code)]
pub fn rb_define_global_function<F: RubyCallback>(
    name: &str,
    func: F,
) -> Result<(), Box<dyn Error>> {
    let c_name = CString::new(name)?;
    unsafe { ruby::rb_define_global_function(c_name.as_ptr(), func.as_ruby(), F::ARGC) };
    Ok(())
}
