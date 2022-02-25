// SPDX-License-Identifier: GPL-2.0

//! TODOABK: finish doc

use core::{
    marker::{PhantomData, Sync},
    ptr,
};

use crate::{bindings, c_types, error, str::CStr, types::PointerWrapper, Result};

/// TODOABK: finish doc
pub struct DebugFsDirEntry<T: PointerWrapper> {
    dentry: *mut bindings::dentry,
    data: *mut c_types::c_void,
    _wrapper: PhantomData<T>,
}

// TODOABK: safety
unsafe impl<T: PointerWrapper> Sync for DebugFsDirEntry<T> {}

impl<T: PointerWrapper> DebugFsDirEntry<T> {
    pub(crate) unsafe fn create_file(
        name: &CStr,
        data: T,
        fops: &'static bindings::file_operations,
    ) -> Result<Self> {
        let name = name.as_char_ptr();
        let data = data.into_pointer() as *mut _;
        let dentry_ptr = error::from_kernel_err_ptr(unsafe {
            bindings::debugfs_create_file(name, 0, ptr::null_mut(), data, fops)
        });
        match dentry_ptr {
            Err(err) => {
                drop(unsafe { T::from_pointer(data) });
                Err(err)
            }
            Ok(dentry) => Ok(DebugFsDirEntry {
                dentry,
                data,
                _wrapper: PhantomData,
            }),
        }
    }
}

impl<T: PointerWrapper> Drop for DebugFsDirEntry<T> {
    fn drop(&mut self) {
        // TODOABK: safety
        unsafe {
            bindings::debugfs_remove(self.dentry);
        }
        // SAFETY: `self.data` was created by a call to `T::into_pointer`.
        unsafe { drop(T::from_pointer(self.data)) }
    }
}
