mod wrapper;
use wrapper::*;

use std::convert::*;

#[no_mangle]
pub extern "C" fn ruby_str_all_whitespace(_module: RubyValue, arg: RubyValue) -> RubyValue {
    let string: &str = <&str>::try_from(arg).unwrap_or("");
    if string.is_ascii() {
        string.bytes().all(|c| (c as char).is_ascii_whitespace())
    } else {
        string.trim_start().is_empty()
    }
    .into()
}

#[no_mangle]
pub extern "C" fn ruby_new_string(_module: RubyValue) -> RubyValue {
    "lfg".into()
}

#[no_mangle]
pub extern "C" fn Init_libemery() {
    let emery_module = rb_define_module("EMERY").expect("invalid module name");

    rb_define_const(emery_module, "EMERY", rb_float_new(1.0)).expect("invalid function name");
    rb_define_module_function(
        emery_module,
        "all_whitespace?",
        ruby_str_all_whitespace as extern "C" fn(RubyValue, RubyValue) -> RubyValue,
    )
    .expect("invalid function name");
    rb_define_module_function(
        emery_module,
        "new_string",
        ruby_new_string as extern "C" fn(RubyValue) -> RubyValue,
    )
    .expect("invalid function name");
}
