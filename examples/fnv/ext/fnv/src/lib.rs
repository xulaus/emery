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

extern "C" fn ruby_str_fnv1a64(_module: RubyValue, arg: RubyValue) -> RubyValue {
    const PRIME: u64 = 1099511628211;
    const BASIS: u64 = 14695981039346656037;
    TryFromRuby::try_from(&arg)
        .map(|string: RubyStringLike| format!("{:x}", fnv1a(PRIME, BASIS, string.bytes())))
        .into()
}
extern "C" fn ruby_str_fnv1a128(_module: RubyValue, arg: RubyValue) -> RubyValue {
    const PRIME: u128 = 309485009821345068724781371;
    const BASIS: u128 = 144066263297769815596495629667062367629;
    TryFromRuby::try_from(&arg)
        .map(|string: RubyStringLike| format!("{:x}", fnv1a(PRIME, BASIS, string.bytes())))
        .into()
}

#[no_mangle]
pub extern "C" fn Init_fnv() {
    RubyModule::new("Digest")
        .add_sub_module("FNV1a64", |fnv1a64| {
            fnv1a64.add_method(
                "hexdigest",
                ruby_str_fnv1a64 as extern "C" fn(RubyValue, RubyValue) -> RubyValue,
            );
        })
        .add_sub_module("FNV1a128", |fnv1a128| {
            fnv1a128.add_method(
                "hexdigest",
                ruby_str_fnv1a128 as extern "C" fn(RubyValue, RubyValue) -> RubyValue,
            );
        });
}
