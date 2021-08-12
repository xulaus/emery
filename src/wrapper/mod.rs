#[allow(warnings)]
mod bindings;
#[allow(warnings)]
mod encoding;

use std::{convert::From, error::Error, ffi::CString, mem, result::Result, slice, str};

use bindings::VALUE;

pub type CallbackPtr = unsafe extern "C" fn() -> VALUE;
pub type RubyConversionError = ();

pub trait TryFromRuby: Sized {
    fn try_from(value: RubyValue) -> Result<Self, RubyConversionError>;
}

unsafe fn rb_str_len(value: VALUE) -> i64 {
    let rstring: *const bindings::RString = std::mem::transmute(value);
    let flags = (*rstring).basic.flags;

    if flags & (bindings::ruby_rstring_flags_RSTRING_NOEMBED as u64) == 0 {
        ((flags & (bindings::ruby_rstring_flags_RSTRING_EMBED_LEN_MASK as u64))
            >> bindings::ruby_rstring_flags_RSTRING_EMBED_LEN_SHIFT as u64) as i64
    } else {
        (*rstring).as_.heap.len
    }
}

unsafe fn rb_str_ptr(value: VALUE) -> *const u8 {
    let rstring: *const bindings::RString = std::mem::transmute(value);
    let flags = (*rstring).basic.flags;

    if flags & (bindings::ruby_rstring_flags_RSTRING_NOEMBED as u64) == 0 {
        (*rstring).as_.ary.as_ptr() as *const u8
    } else {
        (*rstring).as_.heap.ptr as *const u8
    }
}

pub trait RubyCallback {
    const ARGC: i32;
    fn as_ruby(&self) -> CallbackPtr;
}

impl RubyCallback for extern "C" fn(RubyValue) -> RubyValue {
    const ARGC: i32 = 0;

    fn as_ruby(&self) -> CallbackPtr {
        unsafe { std::mem::transmute(*self) }
    }
}

impl RubyCallback for extern "C" fn(RubyValue, RubyValue) -> RubyValue {
    const ARGC: i32 = 1;

    fn as_ruby(&self) -> CallbackPtr {
        unsafe { std::mem::transmute(*self) }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RubyValue(pub VALUE);

impl RubyValue {
    pub fn is_true(&self) -> bool {
        self.0 == (bindings::ruby_special_consts_RUBY_Qtrue as VALUE)
    }

    pub fn is_false(&self) -> bool {
        self.0 == (bindings::ruby_special_consts_RUBY_Qfalse as VALUE)
    }

    pub fn is_nil(&self) -> bool {
        self.0 == (bindings::ruby_special_consts_RUBY_Qnil as VALUE)
    }

    pub fn is_undef(&self) -> bool {
        self.0 == (bindings::ruby_special_consts_RUBY_Qundef as VALUE)
    }

    pub fn is_symbol(&self) -> bool {
        (self.0 & !((!0) << 8)) == (bindings::ruby_special_consts_RUBY_SYMBOL_FLAG as VALUE)
    }

    pub fn is_fixnum(&self) -> bool {
        (self.0 & (bindings::ruby_special_consts_RUBY_FIXNUM_FLAG as VALUE)) != 0
    }

    pub fn is_numeric(&self) -> bool {
        match self.infer_type() {
            Some(bindings::ruby_value_type_RUBY_T_FLOAT)
            | Some(bindings::ruby_value_type_RUBY_T_FIXNUM) => true,
            _ => false,
        }
    }

    fn infer_type(&self) -> Option<bindings::ruby_value_type> {
        if !self.is_special_const() {
            self.builtin_type()
        } else if self.is_false() {
            Some(bindings::ruby_value_type_RUBY_T_FALSE)
        } else if self.is_nil() {
            Some(bindings::ruby_value_type_RUBY_T_NIL)
        } else if self.is_true() {
            Some(bindings::ruby_value_type_RUBY_T_TRUE)
        } else if self.is_undef() {
            Some(bindings::ruby_value_type_RUBY_T_UNDEF)
        } else if self.is_fixnum() {
            Some(bindings::ruby_value_type_RUBY_T_FIXNUM)
        } else if self.is_symbol() {
            Some(bindings::ruby_value_type_RUBY_T_SYMBOL)
        } else {
            Some(bindings::ruby_value_type_RUBY_T_FLOAT)
        }
    }

    fn is_immediate(&self) -> bool {
        (self.0 & (bindings::ruby_special_consts_RUBY_IMMEDIATE_MASK as VALUE)) != 0
    }

    pub fn truthy(&self) -> bool {
        (self.0 & !(bindings::ruby_special_consts_RUBY_Qnil as VALUE)) != 0
    }

    fn is_special_const(&self) -> bool {
        self.is_immediate() || !self.truthy()
    }

    fn builtin_type(&self) -> Option<bindings::ruby_value_type> {
        let basic: *const bindings::RBasic = unsafe { mem::transmute(self.0) };
        let masked = unsafe { (*basic).flags } & (bindings::ruby_value_type_RUBY_T_MASK as VALUE);
        if masked < 0x1f {
            Some(unsafe { mem::transmute(masked as u32) })
        } else {
            None
        }
    }
}

impl TryFromRuby for bool {
    fn try_from(value: RubyValue) -> Result<bool, ()> {
        match value.infer_type() {
            Some(bindings::ruby_value_type_RUBY_T_TRUE) => Ok(true),
            Some(bindings::ruby_value_type_RUBY_T_FALSE) => Ok(false),
            _ => Err(()),
        }
    }
}

impl From<bool> for RubyValue {
    fn from(value: bool) -> RubyValue {
        let wrapped = if value {
            bindings::ruby_special_consts_RUBY_Qtrue as VALUE
        } else {
            bindings::ruby_special_consts_RUBY_Qfalse as VALUE
        };
        RubyValue(wrapped)
    }
}

impl From<&str> for RubyValue {
    fn from(value: &str) -> RubyValue {
        RubyValue(unsafe {
            bindings::rb_utf8_str_new(std::mem::transmute(value.as_ptr()), value.len() as i64)
        })
    }
}

impl From<f64> for RubyValue {
    fn from(value: f64) -> RubyValue {
        RubyValue(unsafe { bindings::rb_float_new(value) })
    }
}

impl<T: TryFromRuby> TryFromRuby for Option<T> {
    fn try_from(value: RubyValue) -> Result<Option<T>, RubyConversionError> {
        if (bindings::ruby_special_consts_RUBY_Qnil as VALUE) == value.0 {
            Ok(None)
        } else {
            Ok(Some(T::try_from(value)?))
        }
    }
}

impl<T: Into<RubyValue>> From<Option<T>> for RubyValue {
    fn from(opt: Option<T>) -> RubyValue {
        opt.map(|x| x.into())
            .unwrap_or(RubyValue(bindings::ruby_special_consts_RUBY_Qnil as VALUE))
    }
}

#[derive(Copy, Clone)]
pub struct RubyString(VALUE);

impl RubyString {
    pub fn is_utf8(&self) -> bool {
        let utf8 = unsafe { encoding::rb_utf8_encoding() };
        let ascii_7bit = unsafe { encoding::rb_usascii_encoding() };

        let str_enc = unsafe { encoding::rb_enc_get(self.0) };
        str_enc == utf8 || str_enc == ascii_7bit
    }

    pub fn is_ascii(&self) -> bool {
        let ascii_7bit = unsafe { encoding::rb_usascii_encoding() };

        let str_enc = unsafe { encoding::rb_enc_get(self.0) };
        str_enc == ascii_7bit
    }

    pub fn len(&self) -> usize {
        unsafe { rb_str_len(self.0) as usize }
    }

    pub fn bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) }
    }

    pub fn try_str(&self) -> Result<&str, RubyConversionError> {
        if self.is_utf8() {
            Ok(unsafe { str::from_utf8_unchecked(self.bytes()) })
        } else {
            Err(())
        }
    }

    pub fn to_owned(&self) -> String {
        if let Ok(s) = self.try_str() {
            s.to_owned()
        } else {
            RubyString(unsafe {
                encoding::rb_str_export_to_enc(self.0, encoding::rb_utf8_encoding())
            })
            .try_str()
            .unwrap()
            .to_owned()
        }
    }

    fn as_ptr(&self) -> *const u8 {
        unsafe { rb_str_ptr(self.0) }
    }
}

impl TryFromRuby for RubyString {
    fn try_from(value: RubyValue) -> Result<Self, RubyConversionError> {
        if value.infer_type() != Some(bindings::ruby_value_type_RUBY_T_STRING) {
            Err(())
        } else {
            Ok(RubyString(value.0))
        }
    }
}

impl TryFromRuby for &str {
    fn try_from(value: RubyValue) -> Result<Self, RubyConversionError> {
        let rstring: RubyString = <RubyString>::try_from(value)?;

        // Would be real nice if we could just rstring.try_str()
        // Borrow checker acts up though as we are being fast and
        // loose with saftey
        if rstring.is_utf8() {
            unsafe {
                let slice: &[u8] = slice::from_raw_parts(rstring.as_ptr(), rstring.len());
                Ok(str::from_utf8_unchecked(slice))
            }
        } else {
            Err(())
        }
    }
}

impl TryFromRuby for &[u8] {
    fn try_from(value: RubyValue) -> Result<Self, RubyConversionError> {
        let string: RubyString = <RubyString>::try_from(value)?;
        unsafe { Ok(slice::from_raw_parts(string.as_ptr(), string.len())) }
    }
}

impl TryFromRuby for f64 {
    fn try_from(value: RubyValue) -> Result<Self, RubyConversionError> {
        if !value.is_numeric() {
            Err(())
        } else {
            Ok(unsafe { bindings::rb_num2dbl(value.0) })
        }
    }
}

#[allow(dead_code)]
pub fn rb_define_global_const(name: &str, value: RubyValue) -> Result<(), Box<dyn Error>> {
    let c_name = CString::new(name)?;
    unsafe { bindings::rb_define_global_const(c_name.as_ptr(), value.0) };
    Ok(())
}

#[allow(dead_code)]
pub fn rb_define_const(
    parent: RubyValue,
    name: &str,
    value: RubyValue,
) -> Result<(), Box<dyn Error>> {
    let c_name = CString::new(name)?;
    unsafe { bindings::rb_define_const(parent.0, c_name.as_ptr(), value.0) };
    Ok(())
}

#[allow(dead_code)]
pub fn rb_define_module(name: &str) -> Result<RubyValue, Box<dyn Error>> {
    let c_name = CString::new(name)?;
    Ok(RubyValue(unsafe {
        bindings::rb_define_module(c_name.as_ptr())
    }))
}

#[allow(dead_code)]
pub fn rb_define_module_under(parent: RubyValue, name: &str) -> Result<RubyValue, Box<dyn Error>> {
    let c_name = CString::new(name)?;
    Ok(RubyValue(unsafe {
        bindings::rb_define_module_under(parent.0, c_name.as_ptr())
    }))
}

#[allow(dead_code)]
pub fn rb_define_method<F: RubyCallback>(
    parent: RubyValue,
    name: &str,
    func: F,
) -> Result<(), Box<dyn Error>> {
    let c_name = CString::new(name)?;
    unsafe { bindings::rb_define_method(parent.0, c_name.as_ptr(), Some(func.as_ruby()), F::ARGC) };
    Ok(())
}

#[allow(dead_code)]
pub fn rb_define_module_function<F: RubyCallback>(
    parent: RubyValue,
    name: &str,
    func: F,
) -> Result<(), Box<dyn Error>> {
    let c_name = CString::new(name)?;
    unsafe {
        bindings::rb_define_module_function(
            parent.0,
            c_name.as_ptr(),
            Some(func.as_ruby()),
            F::ARGC,
        )
    };
    Ok(())
}

#[allow(dead_code)]
pub fn rb_define_global_function<F: RubyCallback>(
    name: &str,
    func: F,
) -> Result<(), Box<dyn Error>> {
    let c_name = CString::new(name)?;
    unsafe { bindings::rb_define_global_function(c_name.as_ptr(), Some(func.as_ruby()), F::ARGC) };
    Ok(())
}
