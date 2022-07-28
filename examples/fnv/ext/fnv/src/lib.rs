extern crate emery;
extern crate num_traits;

use emery::*;

fn fnv1a<T>(prime: T, basis: T, bytes: &[u8]) -> T
where
    T: std::ops::BitXor<Output = T>
        + num_traits::ops::wrapping::WrappingMul
        + std::convert::From<u8>,
{
    bytes.iter().fold(basis, |hash, byte| {
        (hash ^ T::from(*byte)).wrapping_mul(&prime)
    })
}

#[no_mangle]
pub extern "C" fn ruby_str_fnv1a64(_module: RubyValue, arg: RubyValue) -> RubyValue {
    const PRIME: u64 = 1099511628211;
    const BASIS: u64 = 14695981039346656037;
    TryFromRuby::try_from(&arg)
        .map(|string: RubyStringLike| format!("{:x}", fnv1a(PRIME, BASIS, string.bytes())))
        .into()
}
#[no_mangle]
pub extern "C" fn ruby_str_fnv1a128(_module: RubyValue, arg: RubyValue) -> RubyValue {
    const PRIME: u128 = 309485009821345068724781371;
    const BASIS: u128 = 144066263297769815596495629667062367629;
    TryFromRuby::try_from(&arg)
        .map(|string: RubyStringLike| format!("{:x}", fnv1a(PRIME, BASIS, string.bytes())))
        .into()
}

#[no_mangle]
pub extern "C" fn Init_fnv() {
    let mut emery_module = rb_define_module("Digest").expect("invalid module name");
    let mut fnv1a64 =
        rb_define_module_under(&mut emery_module, "FNV1a64").expect("invalid module name");
    rb_define_module_function(
        &mut fnv1a64,
        "hexdigest",
        ruby_str_fnv1a64 as extern "C" fn(RubyValue, RubyValue) -> RubyValue,
    )
    .expect("invalid function name");
    let mut fnv1a128 =
        rb_define_module_under(&mut emery_module, "FNV1a128").expect("invalid module name");
    rb_define_module_function(
        &mut fnv1a128,
        "hexdigest",
        ruby_str_fnv1a128 as extern "C" fn(RubyValue, RubyValue) -> RubyValue,
    )
    .expect("invalid function name");
}
