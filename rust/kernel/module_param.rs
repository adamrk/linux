use core::fmt::Write;

/// Types that can be used for module parameters.
/// Note that displaying the type in `sysfs` will fail if `to_string` returns
/// more than 4K bytes (including an additional null terminator).
pub trait ModuleParam : core::fmt::Display + core::marker::Sized {
    fn try_from_param_arg(arg: &[u8]) -> Option<Self>;

    /// # Safety
    ///
    /// `val` must point to a valid null-terminated string. The `arg` field of
    /// `param` must be an instance of `Self`.
    unsafe extern "C" fn set_param(val: *const crate::c_types::c_char, param: *const crate::bindings::kernel_param) -> crate::c_types::c_int {
        let arg = crate::c_types::c_string_bytes(val);
        match Self::try_from_param_arg(arg) {
            Some(new_value) => {
                let old_value = (*param).__bindgen_anon_1.arg as *mut Self;
                let _ = core::ptr::replace(old_value, new_value);
                0
            }
            None => crate::error::Error::EINVAL.to_kernel_errno()
        }
    }

    /// # Safety
    ///
    /// `buf` must be a buffer of length at least `kernel::PAGE_SIZE` that is
    /// writeable. The `arg` field of `param` must be an instance of `Self`.
    unsafe extern "C" fn get_param(buf: *mut crate::c_types::c_char, param: *const crate::bindings::kernel_param) -> crate::c_types::c_int {
        let slice = core::slice::from_raw_parts_mut(buf as *mut u8, crate::PAGE_SIZE);
        let mut buf = crate::buffer::Buffer::new(slice);
        match write!(buf, "{}\0", *((*param).__bindgen_anon_1.arg as *mut Self)) {
            Err(_) => crate::error::Error::EINVAL.to_kernel_errno(),
            Ok(()) => buf.bytes_written() as crate::c_types::c_int,
        }
    }

    /// # Safety
    ///
    /// The `arg` field of `param` must be an instance of `Self`.
    unsafe extern "C" fn free(arg: *mut crate::c_types::c_void) {
        core::ptr::drop_in_place(arg as *mut Self);
    }
}

macro_rules! make_param_ops {
    ($ops:ident, $ty:ident) => {
        impl ModuleParam for $ty {
            fn try_from_param_arg(arg: &[u8]) -> Option<Self> {
                let utf8 = core::str::from_utf8(arg).ok()?;
                utf8.parse::<$ty>().ok()
            }
        }

        pub static $ops: crate::bindings::kernel_param_ops = crate::bindings::kernel_param_ops {
            flags: 0,
            set: Some(<$ty as crate::module_param::ModuleParam>::set_param),    
            get: Some(<$ty as crate::module_param::ModuleParam>::get_param),
            free: Some(<$ty as crate::module_param::ModuleParam>::free),
        };
    }
}

make_param_ops!(PARAM_OPS_I8, i8);
make_param_ops!(PARAM_OPS_I64, i64);
make_param_ops!(PARAM_OPS_USIZE, usize);
make_param_ops!(PARAM_OPS_ISIZE, isize);
