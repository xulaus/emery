extern crate libc;

mod ffi {
    use libc::c_double;
    pub use libc::uintptr_t as VALUE;

    #[link(name = "ruby")]
    extern "C" {
        pub fn rb_define_global_const(name: *const u8, value: VALUE);
        pub fn rb_float_new(value: c_double) -> VALUE;
    }
}

pub use ffi::VALUE;

pub fn rb_float_new(value: f64) -> VALUE {
    unsafe { ffi::rb_float_new(value) }
}

pub fn rb_define_global_const(name: &str, value: VALUE) {
    unsafe { ffi::rb_define_global_const(name.as_bytes().as_ptr(), value) }
}
