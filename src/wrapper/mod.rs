#[allow(warnings)]
mod bindings;
#[allow(warnings)]
mod encoding;

use std::{convert::From, error::Error, ffi::CString, mem, result::Result, slice, str};

use bindings::VALUE;

pub type CallbackPtr = unsafe extern "C" fn() -> VALUE;
#[derive(Debug)]
pub struct RubyConversionError{
    value: String,
    into_type: String
}

impl std::fmt::Display for RubyConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Invalid Conversion of \"{}\" into {}", &self.value, &self.into_type)
    }
}

pub trait TryFromRuby<'a>: Sized {
    fn try_from(value: &'a RubyValue) -> Result<Self, RubyConversionError>;
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

// Conversions for booleans

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

impl TryFromRuby<'_> for bool {
    fn try_from(value: &RubyValue) -> Result<bool, RubyConversionError> {
        match value.infer_type() {
            Some(bindings::ruby_value_type_RUBY_T_TRUE) => Ok(true),
            Some(bindings::ruby_value_type_RUBY_T_FALSE) => Ok(false),
            _ => Err(RubyConversionError{
                value: "".to_owned(), // TODO
                into_type: "bool".to_owned()
            }),
        }
    }
}

// Conversions for numbers

impl From<f64> for RubyValue {
    fn from(value: f64) -> RubyValue {
        RubyValue(unsafe { bindings::rb_float_new(value) })
    }
}
impl From<f32> for RubyValue {
    fn from(value: f32) -> RubyValue {
        RubyValue(unsafe { bindings::rb_float_new(value as f64) })
    }
}

impl From<u64> for RubyValue {
    fn from(value: u64) -> RubyValue {
        RubyValue(unsafe { bindings::rb_ull2inum(value) })
    }
}

impl From<i64> for RubyValue {
    fn from(value: i64) -> RubyValue {
        RubyValue(unsafe { bindings::rb_ll2inum(value) })
    }
}

impl From<isize> for RubyValue {
    fn from(value: isize) -> RubyValue {
        RubyValue(unsafe { bindings::rb_int2inum(value) })
    }
}

impl From<usize> for RubyValue {
    fn from(value: usize) -> RubyValue {
        RubyValue(unsafe { bindings::rb_uint2inum(value) })
    }
}

impl From<u32> for RubyValue {
    fn from(value: u32) -> RubyValue {
        RubyValue(unsafe { bindings::rb_uint2inum(value as usize) })
    }
}

impl From<i32> for RubyValue {
    fn from(value: i32) -> RubyValue {
        RubyValue(unsafe { bindings::rb_int2inum(value as isize) })
    }
}

impl TryFromRuby<'_> for f64 {
    fn try_from(value: &RubyValue) -> Result<Self, RubyConversionError> {
        if !value.is_numeric() {
            Err(RubyConversionError{
                value: "".to_owned(), // TODO
                into_type: "64 bit float".to_owned()
            })
        } else {
            Ok(unsafe { bindings::rb_num2dbl(value.0) })
        }
    }
}

impl TryFromRuby<'_> for i64 {
    fn try_from(value: &RubyValue) -> Result<i64, RubyConversionError> {
        if value.is_fixnum() {
            Ok(unsafe { bindings::rb_fix2int(value.0) })
        } else {
            Err(RubyConversionError{
                value: "".to_owned(), // TODO
                into_type: "64 bit integer".to_owned()
            })
        }
    }
}

// Conversions for Options

impl<'a, T: TryFromRuby<'a>> TryFromRuby<'a> for Option<T> {
    fn try_from(value: &'a RubyValue) -> Result<Option<T>, RubyConversionError> {
        if (bindings::ruby_special_consts_RUBY_Qnil as VALUE) == value.0 {
            Ok(None)
        } else {
            Ok(Some(T::try_from(value)?))
        }
    }
}

impl<T: Into<RubyValue>> From<Option<T>> for RubyValue {
    fn from(value: Option<T>) -> RubyValue {
        match value {
            Some(inner) => inner.into(),
            None => RubyValue(bindings::ruby_special_consts_RUBY_Qnil as VALUE),
        }
    }
}

// Result to exception conversion

impl<T: Into<RubyValue>, Err: std::fmt::Display> From<Result<T, Err>> for RubyValue {
    fn from(value: Result<T, Err>) -> RubyValue {
        match value {
            Ok(inner) => inner.into(),
            Err(e) => {
                let error = format!("{}", e);
                unsafe {
                    bindings::rb_exc_raise(
                        bindings::rb_exc_new(
                         bindings::rb_eRuntimeError,
                         error.as_ptr() as *const std::os::raw::c_char,
                         error.len() as std::os::raw::c_long
                        )
                    );
                }
                RubyValue(bindings::ruby_special_consts_RUBY_Qnil as VALUE)
            }
        }
    }
}

// String conversions and wrappers

impl From<&str> for RubyValue {
    fn from(value: &str) -> RubyValue {
        RubyValue(unsafe {
            bindings::rb_utf8_str_new(std::mem::transmute(value.as_ptr()), value.len() as i64)
        })
    }
}

#[derive(Copy, Clone)]
pub struct RubyString<'a>(&'a VALUE);

impl<'a> RubyString<'a> {
    pub fn is_utf8(&self) -> bool {
        let utf8 = unsafe { encoding::rb_utf8_encoding() };
        let ascii_7bit = unsafe { encoding::rb_usascii_encoding() };

        let str_enc = unsafe { encoding::rb_enc_get(*self.0) };
        str_enc == utf8 || str_enc == ascii_7bit
    }

    pub fn is_ascii(&self) -> bool {
        let ascii_7bit = unsafe { encoding::rb_usascii_encoding() };

        let str_enc = unsafe { encoding::rb_enc_get(*self.0) };
        str_enc == ascii_7bit
    }

    pub fn len(&self) -> usize {
        unsafe { rb_str_len(*self.0) as usize }
    }

    pub fn bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) }
    }

    pub fn try_str(&'a self) -> Result<&'a str, RubyConversionError> {
        if self.is_utf8() {
            Ok(unsafe { str::from_utf8_unchecked(self.bytes()) })
        } else {
            Err(RubyConversionError{
                value: "".to_owned(), // TODO
                into_type: "utf8 string".to_owned()
            })
        }
    }

    pub fn to_owned(&self) -> String {
        if let Ok(s) = self.try_str() {
            s.to_owned()
        } else {
            let converted = unsafe { encoding::rb_str_export_to_enc(*(self.0), encoding::rb_utf8_encoding()) };
            RubyString(&converted)
            .try_str()
            .unwrap()
            .to_owned()
        }
    }

    unsafe fn as_ptr(&self) -> *const u8 {
        rb_str_ptr(*self.0)
    }
}

impl<'a> TryFromRuby<'a> for RubyString<'a> {
    fn try_from(value: &'a RubyValue) -> Result<Self, RubyConversionError> {
        if value.infer_type() != Some(bindings::ruby_value_type_RUBY_T_STRING) {
            Err(RubyConversionError{
                value: "".to_owned(), // TODO
                into_type: "string".to_owned()
            })
        } else {
            Ok(RubyString(&value.0))
        }
    }
}

// Add hacky direct binding

#[allow(dead_code)]
pub fn rb_define_global_const(name: &str, value: RubyValue) -> Result<(), Box<dyn Error>> {
    let c_name = CString::new(name)?;
    unsafe { bindings::rb_define_global_const(c_name.as_ptr(), value.0) };
    Ok(())
}

#[allow(dead_code)]
pub fn rb_define_const(
    parent: &mut RubyValue,
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
pub fn rb_define_module_under(parent: &mut RubyValue, name: &str) -> Result<RubyValue, Box<dyn Error>> {
    let c_name = CString::new(name)?;
    Ok(RubyValue(unsafe {
        bindings::rb_define_module_under(parent.0, c_name.as_ptr())
    }))
}

#[allow(dead_code)]
pub fn rb_define_method<F: RubyCallback>(
    parent: &mut RubyValue,
    name: &str,
    func: F,
) -> Result<(), Box<dyn Error>> {
    let c_name = CString::new(name)?;
    unsafe { bindings::rb_define_method(parent.0, c_name.as_ptr(), Some(func.as_ruby()), F::ARGC) };
    Ok(())
}

#[allow(dead_code)]
pub fn rb_define_module_function<F: RubyCallback>(
    parent: &mut RubyValue,
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
