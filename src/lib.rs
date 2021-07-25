mod bindings;
use bindings::*;

#[no_mangle]
pub extern "C" fn emery_fn(_a: VALUE) -> VALUE {
    rb_float_new(2.0)
}

#[no_mangle]
pub extern "C" fn Init_libemery() {
    let emery_module = rb_define_module("EMERY").expect("invalid module name");

    rb_define_const(emery_module, "EMERY", rb_float_new(1.0)).expect("invalid function name");
    rb_define_module_function(
        emery_module,
        "fn",
        emery_fn as extern "C" fn(VALUE) -> VALUE,
    )
    .expect("invalid function name");
}
