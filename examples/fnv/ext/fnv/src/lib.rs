extern crate emery;

use emery::*;

#[no_mangle]
pub extern "C" fn ruby_str_fnv1a(_module: RubyValue, arg: RubyValue) -> RubyValue {
    let casted: Result<&[u8], RubyConversionError> = TryFromRuby::try_from(arg);
    if let Ok(string) = casted {
        const PRIME: u64 = 1099511628211;
        const BASIS: u64 = 14695981039346656037;

        Ok(string.iter().fold(
            BASIS,
            |hash, byte| {
                (hash ^ (*byte as u64)).wrapping_mul(PRIME)
            }
        ))
    } else {
        Err("FNV only works on things that can be converted to bytes")
    }
    .into()
}

#[no_mangle]
pub extern "C" fn Init_fnv() {
    let emery_module = rb_define_module("EMERY").expect("invalid module name");
    let fnv1a = rb_define_module_under(emery_module, "FNV1a").expect("invalid module name");
    rb_define_module_function(
        fnv1a,
        "hexdigest",
        ruby_str_fnv1a as extern "C" fn(RubyValue, RubyValue) -> RubyValue,
    )
    .expect("invalid function name");
}
