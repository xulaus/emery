mod wrapper;
use wrapper::*;

use std::convert::*;

#[no_mangle]
pub extern "C" fn ruby_str_all_whitespace(_module: RubyValue, arg: RubyValue) -> RubyValue {
    let casted: Result<RubyString, RubyConversionError> = TryFromRuby::try_from(arg);
    match casted {
        Ok(string) => {
            if string.len() == 0 {
                Some(true)
            } else if let Ok(utf8) = string.try_str() {
                if utf8.is_ascii() {
                    Some(utf8.bytes().all(|c| (c as char).is_ascii_whitespace()))
                } else {
                    Some(utf8.trim_start().is_empty())
                }
            } else {
                let utf8: String = string.to_owned();
                Some(utf8.trim_start().is_empty())
            }
        }
        _ => None,
    }
    .into()
}

#[no_mangle]
pub extern "C" fn ruby_new_string(_module: RubyValue, arg: RubyValue) -> RubyValue {
    let inp: Option<&str> = wrapper::TryFromRuby::try_from(arg).unwrap_or(None);
    inp.map(|_| "asd").into()
}

#[no_mangle]
pub extern "C" fn Init_libemery() {
    let emery_module = rb_define_module("EMERY").expect("invalid module name");

    rb_define_const(emery_module, "EMERY", 1.0.into()).expect("invalid function name");
    rb_define_module_function(
        emery_module,
        "all_whitespace?",
        ruby_str_all_whitespace as extern "C" fn(RubyValue, RubyValue) -> RubyValue,
    )
    .expect("invalid function name");
    rb_define_module_function(
        emery_module,
        "new_string",
        ruby_new_string as extern "C" fn(RubyValue, RubyValue) -> RubyValue,
    )
    .expect("invalid function name");
}
