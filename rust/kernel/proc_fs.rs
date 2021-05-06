// SPDX-License-Identifier: GPL-2.0

//! Type for defining `proc` files.
//!
//! This module allows Rust devices to create entries in `/proc` from a
//! [`bindings::proc_ops`] vtable.
//!
//! C header: [`include/linux/proc_fs.h`](../../../include/linux/proc_fs.h)
//!
//! Reference: <https://www.kernel.org/doc/html/latest/filesystems/proc.html>

use alloc::boxed::Box;
use core::{
    marker::{PhantomData, Sync},
    ops::Deref,
    ptr,
};

use crate::{bindings, c_types, CStr, Error, KernelResult};

/// An entry under `/proc` containing data of type `T`.
///
/// This is the Rust equivalent to [`proc_dir_entry`] on the C side.
///
/// # Invariants
///
/// The pointer [`ProcDirEntry::proc_dir_entry`] is a valid pointer and
/// it's field [`bindings::proc_dir_entry::data`] is a valid pointer to
/// `T`.
///
/// [`proc_dir_entry`]: ../../../fs/proc/internal.h
pub struct ProcDirEntry<T> {
    proc_dir_entry: *mut bindings::proc_dir_entry,
    data: PhantomData<T>,
}

// SAFETY: The `proc_dir_entry` raw pointer isn't accessible.
unsafe impl<T> Sync for ProcDirEntry<T> {}

impl<T> Drop for ProcDirEntry<T> {
    fn drop(&mut self) {
        // SAFETY: `ProcDirEntry` is guaranteed to have a valid pointer to `T`
        // in the `data` field of `proc_dir_entry`.
        let data = unsafe { Box::from_raw((*self.proc_dir_entry).data as *mut T) };
        // SAFETY: Calling a C function. `proc_dir_entry` is a valid pointer to
        // a `bindings::proc_dir_entry` because it was created by a call to
        // `proc_create_data` which only returns valid pointers.
        unsafe {
            bindings::proc_remove(self.proc_dir_entry);
        }
        drop(data);
    }
}

/// Create an entry in `/proc` containing data of type `T`.
///
/// Corresponds to [`proc_create_data`] on the C side.
///
/// [`proc_create_data]: ../../../fs/proc/generic.c
pub(crate) fn proc_create_data<T>(
    name: CStr<'static>,
    proc_ops: &'static bindings::proc_ops,
    data: T,
) -> KernelResult<ProcDirEntry<T>> {
    let data_ptr = Box::into_raw(Box::try_new(data)?) as *mut c_types::c_void;
    let name = name.deref().as_ptr() as *const u8 as *const c_types::c_char;

    // SAFETY: Calling a C function. `name` is guaranteed to be null terminated
    // because it is of type `CStr`.
    let proc_dir_entry =
        unsafe { bindings::proc_create_data(name, 0, ptr::null_mut(), proc_ops, data_ptr) };
    if proc_dir_entry.is_null() {
        Err(Error::ENOMEM)
    } else {
        // INVARIANT: `proc_dir_entry` is a valid pointer and it's data field
        // is a valid pointer to `T`.
        Ok(ProcDirEntry {
            proc_dir_entry,
            data: PhantomData,
        })
    }
}
