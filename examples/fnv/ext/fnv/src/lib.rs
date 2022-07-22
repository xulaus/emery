extern crate emery;

use emery::*;

fn fnv1a64(arg: RubyValue) -> Result<String, RubyConversionError>{
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

fn fnv1a128(arg: RubyValue) -> Result<String, RubyConversionError>{
    let string: RubyStringLike = TryFromRuby::try_from(&arg)?;
    const PRIME: u128 = 309485009821345068724781371;
    const BASIS: u128 = 144066263297769815596495629667062367629;
    let hash = string.bytes().iter().fold(
        BASIS,
        |hash, byte| {
            (hash ^ (*byte as u128)).wrapping_mul(PRIME)
        }
    );
    Ok(format!("{:x}", hash))
}

#[no_mangle]
pub extern "C" fn ruby_str_fnv1a64(_module: RubyValue, arg: RubyValue) -> RubyValue {
    fnv1a64(arg).into()
}
#[no_mangle]
pub extern "C" fn ruby_str_fnv1a128(_module: RubyValue, arg: RubyValue) -> RubyValue {
    fnv1a128(arg).into()
}

#[no_mangle]
pub extern "C" fn Init_fnv() {
    let mut emery_module = rb_define_module("Digest").expect("invalid module name");
    let mut fnv1a64 = rb_define_module_under(&mut emery_module, "FNV1a64").expect("invalid module name");
    rb_define_module_function(
        &mut fnv1a64,
        "hexdigest",
        ruby_str_fnv1a64 as extern "C" fn(RubyValue, RubyValue) -> RubyValue,
    ).expect("invalid function name");
    let mut fnv1a128 = rb_define_module_under(&mut emery_module, "FNV1a128").expect("invalid module name");
    rb_define_module_function(
        &mut fnv1a128,
        "hexdigest",
        ruby_str_fnv1a128 as extern "C" fn(RubyValue, RubyValue) -> RubyValue,
    ).expect("invalid function name");
}
