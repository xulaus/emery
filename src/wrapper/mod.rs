#[allow(warnings)]
mod bindings;
#[allow(warnings)]
mod encoding;

use std::{convert::From, error::Error, ffi::CString, mem, result::Result, slice, str};

use bindings::VALUE;

pub type CallbackPtr = unsafe extern "C" fn() -> VALUE;
pub type RubyEncoding = *const libc::c_void;

pub trait TryFromRuby: Sized {
    type Error;

    fn try_from(value: RubyValue) -> Result<Self, Self::Error>;
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
        std::mem::transmute(&(*rstring).as_)
    } else {
        std::mem::transmute((*rstring).as_.heap.ptr)
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

    pub fn infer_type(&self) -> Option<bindings::ruby_value_type> {
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

    fn test(&self) -> bool {
        (self.0 & !(bindings::ruby_special_consts_RUBY_Qnil as VALUE)) != 0
    }

    fn is_special_const(&self) -> bool {
        self.is_immediate() || !self.test()
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
    type Error = ();

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

impl<T> TryFromRuby for Option<T>
where
    T: TryFromRuby<Error = ()>,
{
    type Error = ();

    fn try_from(value: RubyValue) -> Result<Option<T>, ()> {
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

impl TryFromRuby for &str {
    type Error = ();

    fn try_from(value: RubyValue) -> Result<Self, Self::Error> {
        unsafe {
            if value.infer_type() != Some(bindings::ruby_value_type_RUBY_T_STRING) {
                return Err(());
            }
            let utf8 = encoding::rb_utf8_encoding();
            let ascii = encoding::rb_usascii_encoding();
            let str_enc = encoding::rb_enc_get(value.0);

            let data = if str_enc != utf8 && str_enc != ascii {
                encoding::rb_str_export_to_enc(value.0, utf8)
            } else {
                value.0
            };
            let len = rb_str_len(data);
            let slice = slice::from_raw_parts(rb_str_ptr(data), len as usize);
            Ok(str::from_utf8_unchecked(slice))
        }
    }
}

impl TryFromRuby for &[u8] {
    type Error = ();

    fn try_from(value: RubyValue) -> Result<Self, Self::Error> {
        unsafe {
            if value.infer_type() != Some(bindings::ruby_value_type_RUBY_T_STRING) {
                return Err(());
            }

            let data = value.0;
            let len = rb_str_len(data);
            Ok(slice::from_raw_parts(rb_str_ptr(data), len as usize))
        }
    }
}

#[allow(dead_code)]
pub fn rb_float_new(value: f64) -> RubyValue {
    RubyValue(unsafe { bindings::rb_float_new(value) })
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
pub fn rb_num2dbl(value: RubyValue) -> f64 {
    unsafe { bindings::rb_num2dbl(value.0) }
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
