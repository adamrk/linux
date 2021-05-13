// SPDX-License-Identifier: GPL-2.0

//! Type for defining `proc` files.
//!
//! This module allows Rust devices to create entries in `/proc` from a
//! [`bindings::proc_ops`] vtable.
//!
//! C header: [`include/linux/proc_fs.h`](../../../include/linux/proc_fs.h)
//!
//! Reference: <https://www.kernel.org/doc/html/latest/filesystems/proc.html>

use core::{
    marker::{PhantomData, Sync},
    ptr,
};

use crate::{
    bindings, c_types,
    seq_file::{SeqFileOperationsVTable, SeqOperations},
    str::CStr,
    types::PointerWrapper,
    Error, Result,
};

/// An entry under `/proc` containing data of type `T`.
///
/// This is the Rust equivalent to [`proc_dir_entry`] on the C side.
///
/// # Invariants
///
/// The [`ProcDirEntry::proc_dir_entry`] is a valid pointer.
/// [`ProcDirEntry::data`] points to the PDE data of
/// [`ProcDirEntry::proc_dir_entry`].
/// [`ProcDirEntry::data`] was created by a call to `T::into_pointer`.
///
/// [`proc_dir_entry`]: ../../../fs/proc/internal.h
pub struct ProcDirEntry<T: PointerWrapper> {
    proc_dir_entry: *mut bindings::proc_dir_entry,
    data: *const c_types::c_void,
    _wrapper: PhantomData<T>,
}

// SAFETY: The `proc_dir_entry` and `data` raw pointers aren't accessible.
unsafe impl<T: PointerWrapper> Sync for ProcDirEntry<T> {}

impl<T: PointerWrapper> Drop for ProcDirEntry<T> {
    fn drop(&mut self) {
        // SAFETY: Calling a C function. `proc_dir_entry` is a valid pointer to
        // a `bindings::proc_dir_entry` because it was created by a call to
        // `proc_create_data` which only returns valid pointers.
        unsafe {
            bindings::proc_remove(self.proc_dir_entry);
        }
        // SAFETY: `self.data` was created by a call to `T::into_pointer`.
        unsafe { drop(T::from_pointer(self.data)) }
    }
}

impl<T: PointerWrapper> ProcDirEntry<T> {
    /// Create a seq_file entry in `/proc` containing data of type `S`.
    ///
    /// Corresponds to [`proc_create_seq_private`] on the C side.
    ///
    /// [`proc_create_seq_private`]: ../../../fs/proc/generic.c
    pub fn new_seq_private<S>(name: &CStr, data: T) -> Result<Self>
    where
        S: SeqOperations<DataWrapper = T>,
    {
        let data = data.into_pointer();
        let name = name.as_char_ptr();

        // SAFETY: Calling a C function. The vtable for `S` expects a
        // `S::DataWrapper = T` pointer in the data field of the associated
        // `proc_dir_entry`.  `name` is guaranteed to be null terminated
        // because it is of type `CStr`.
        let proc_dir_entry = unsafe {
            bindings::proc_create_seq_private(
                name,
                0,
                ptr::null_mut(),
                SeqFileOperationsVTable::<S>::build(),
                0,
                data as *mut c_types::c_void,
            )
        };
        if proc_dir_entry.is_null() {
            // SAFETY: `data` was created with a call to `T::into_pointer`.
            drop(unsafe { T::from_pointer(data) });
            Err(Error::ENOMEM)
        } else {
            // INVARIANT: `proc_dir_entry` is a valid pointer.
            // The `data` points to the data stored in `proc_dir_entry`, and
            // `data` was created by `T::into_pointer`.
            Ok(ProcDirEntry {
                proc_dir_entry,
                data,
                _wrapper: PhantomData,
            })
        }
    }
}
