extern crate emery;

use emery::*;

fn fnv1a (arg: RubyValue) -> Result<String, RubyConversionError>{
    let string: RubyStringLike = TryFromRuby::try_from(&arg)?;
    const PRIME: u64 = 1099511628211;
    const BASIS: u64 = 14695981039346656037;
    let hash = string.bytes().iter().fold(
        BASIS,
        |hash, byte| {
            (hash ^ (*byte as u64)).wrapping_mul(PRIME)
        }
    );

    Ok(format!("{:x}", hash))
}

#[no_mangle]
pub extern "C" fn ruby_str_fnv1a(_module: RubyValue, arg: RubyValue) -> RubyValue {
    fnv1a(arg).into()
}

#[no_mangle]
pub extern "C" fn Init_fnv() {
    let mut emery_module = rb_define_module("EMERY").expect("invalid module name");
    let mut fnv1a = rb_define_module_under(&mut emery_module, "FNV1a").expect("invalid module name");
    rb_define_module_function(
        &mut fnv1a,
        "hexdigest",
        ruby_str_fnv1a as extern "C" fn(RubyValue, RubyValue) -> RubyValue,
    )
    .expect("invalid function name");
}
