mod bindings;
use bindings::{rb_define_global_const, rb_float_new, VALUE};

#[no_mangle]
pub extern "C" fn Init_libemery() {
    rb_define_global_const("EMERY", rb_float_new(1.0));
}
