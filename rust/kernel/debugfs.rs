// SPDX-License-Identifier: GPL-2.0

//! Rust implementation of `debugfs`.
//!
//! This module allows Rust kernel modules to create directories and files in
//! `/debugfs`.
//!
//! C header: [`include/linux/debugfs.h`](../../../include/linux/debugfs.h)
//!
//! Reference: <https://www.kernel.org/doc/html/latest/filesystems/debugfs.html>

use alloc::{boxed::Box, vec::Vec};
use core::{
    marker::{PhantomData, Sync},
    ptr,
};

use crate::{bindings, c_types, error, str::CStr, types::PointerWrapper, Result};

/// An `inode` for a directory in debugfs.
pub struct DebugFsDirectory {
    dentry: *mut bindings::dentry,
    has_parent: bool,
    file_children: Vec<Box<dyn Sync>>,
    dir_children: Vec<DebugFsDirectory>,
}

// SAFETY: There are no public functions that take a shared [`DebugFsDirectory`]
// reference and all its fields are private so a thread can't actually do
// anything with a `&DebugFsDirectory`. This makes it is safe to share across
// threads. [`DebufFsDirectory::create`] does take a `&DebugFsDirectory`, but it
// is only publicly accessible through [`DebufFsDirectory::create_with_parent`]
// which requires the reference to be mutable.
unsafe impl Sync for DebugFsDirectory {}

impl DebugFsDirectory {
    /// Create a new directory in `debugfs` under `parent`. If `parent` is
    /// `None`, it will be created at the `debugfs` root. The directory will be
    /// recursively removed on drop.
    fn create(name: &CStr, parent: Option<&DebugFsDirectory>) -> Result<Self> {
        let name = name.as_char_ptr();
        let parent_ptr = match parent {
            Some(dir) => dir.dentry,
            None => ptr::null_mut(),
        };
        // SAFETY: Calling a C function. `name` is a valid null-terminated
        // string because it came from a [`CStr`] and `parent` is either null or
        // valid because it came from a [`DebugFsDirectory`].
        let dentry =
            error::from_kernel_err_ptr(unsafe { bindings::debugfs_create_dir(name, parent_ptr) })?;
        Ok(DebugFsDirectory {
            dentry,
            has_parent: parent.is_some(),
            file_children: Vec::new(),
            dir_children: Vec::new(),
        })
    }

    /// Create a new directory in `debugfs` under `parent`. The directory will
    /// be removed when the parent is dropped.
    pub fn create_with_parent<'name, 'parent>(
        name: &'name CStr,
        parent: &'parent mut DebugFsDirectory,
    ) -> Result<&'parent mut DebugFsDirectory> {
        let result = DebugFsDirectory::create(name, Some(parent))?;
        parent.dir_children.try_push(result)?;
        let index = parent.dir_children.len() - 1;
        Ok(&mut parent.dir_children[index])
    }

    /// Create a new directory at the toplevel of `debugfs`.
    /// The directory will be recursively removed on drop.
    pub fn create_toplevel(name: &CStr) -> Result<DebugFsDirectory> {
        DebugFsDirectory::create(name, None)
    }
}

impl Drop for DebugFsDirectory {
    fn drop(&mut self) {
        // If this entry has a parent, we don't need to worry about removal
        // because the parent will remove its children when dropped. Otherwise
        // we need to clean up.
        if !self.has_parent {
            // SAFETY: Calling a C function. `dentry` must have been created by
            // a call to [`DebugFsDirectory::create`] which always returns a
            // valid `dentry`. There is no parent, so the `dentry` couldn't have
            // been removed and must still be valid.
            unsafe {
                bindings::debugfs_remove(self.dentry);
            }
        }
    }
}

/// An `inode` for a file in debugfs with a `T` stored in `i_private`.
pub struct DebugFsFile<T: PointerWrapper> {
    dentry: *mut bindings::dentry,
    data: *mut c_types::c_void,
    has_parent: bool,
    _wrapper: PhantomData<T>,
}

// SAFETY: There are no public methods available on [`DebugFsFile`] so a thread
// can't actually do anything with a `&DebugFsFile`. This makes it is safe to
// share across threads.
unsafe impl<T: PointerWrapper> Sync for DebugFsFile<T> {}

impl<T: PointerWrapper + 'static> DebugFsFile<T> {
    /// Create a file in the `debugfs` directory under `parent`. If `parent` is
    /// `None` then the file will be created at the root of the `debugfs`
    /// directory.
    ///
    /// # Safety
    ///
    /// `fops` must be valid when opening an `inode` with `data::into_pointer`
    /// stored in `i_private`.
    unsafe fn create(
        name: &CStr,
        parent: Option<&DebugFsDirectory>,
        data: T,
        fops: &'static bindings::file_operations,
    ) -> Result<DebugFsFile<T>> {
        let name = name.as_char_ptr();
        let data = data.into_pointer() as *mut _;
        let parent = parent.map(|p| p.dentry).unwrap_or_else(ptr::null_mut);
        // SAFETY: Calling a C function. `name` will be a valid null-terminated
        // string because it came from a [`CStr`]. The caller guarantees that
        // `fops` is valid for an inode with `data` in `i_private`.
        let dentry_ptr = error::from_kernel_err_ptr(unsafe {
            bindings::debugfs_create_file(name, 0, parent, data, fops)
        });
        match dentry_ptr {
            Err(err) => {
                // SAFETY: `data` was created by `T::into_pointer` just above.
                drop(unsafe { T::from_pointer(data) });
                Err(err)
            }
            Ok(dentry) => Ok(DebugFsFile {
                dentry,
                data,
                has_parent: true,
                _wrapper: PhantomData,
            }),
        }
    }

    /// Create a file in the `debugfs` directory under `parent`.
    ///
    /// # Safety
    ///
    /// `fops` must be valid when opening an `inode` with `data::into_pointer`
    /// stored in `i_private`.
    pub(crate) unsafe fn create_with_parent(
        name: &CStr,
        parent: &mut DebugFsDirectory,
        data: T,
        fops: &'static bindings::file_operations,
    ) -> Result<()> {
        // SAFETY: The caller must ensure the safety conditions on `create` are
        // met.
        let file: DebugFsFile<T> = unsafe { Self::create(name, Some(parent), data, fops) }?;
        let boxed: Box<dyn Sync> = Box::try_new(file)?;
        parent.file_children.try_push(boxed)?;
        Ok(())
    }

    /// Create a file at the top level of the `debugfs` directory.
    ///
    /// # Safety
    ///
    /// `fops` must be valid when opening an `inode` with `data::into_pointer`
    /// stored in `i_private`.
    pub(crate) unsafe fn create_toplevel(
        name: &CStr,
        data: T,
        fops: &'static bindings::file_operations,
    ) -> Result<DebugFsFile<T>> {
        // SAFETY: The caller must ensure the safety conditions on `create` are
        // met.
        unsafe { Self::create(name, None, data, fops) }
    }
}

impl<T: PointerWrapper> Drop for DebugFsFile<T> {
    fn drop(&mut self) {
        // If there is a parent then the parent is responsible for dropping the
        // dentries.
        if !self.has_parent {
            // SAFETY: Calling a C function. `dentry` must have been created by
            // a call to [`DebugFsFile::create`] which always returns a valid
            // `dentry`. Since there is no parent that can remove the `dentry`
            // it must still exist.
            unsafe { bindings::debugfs_remove(self.dentry) };
        }
        // SAFETY: `self.data` was created by a call to `T::into_pointer` in
        // [`DebugFsFile::create`].
        unsafe { T::from_pointer(self.data) };
    }
}
