// SPDX-License-Identifier: GPL-2.0

//! Example of using a [`seq_file`] in Rust.
//!
//! C header: [`include/linux/seq_file.h`](../../../include/linux/seq_file.h)
//! C header: [`include/linux/proc_fs.h`](../../../include/linux/proc_fs.h)
//!
//! Reference: <https://www.kernel.org/doc/html/latest/filesystems/seq_file.html>

#![no_std]
#![feature(allocator_api, global_asm, try_reserve)]

use alloc::{boxed::Box, sync::Arc};
use core::{
    convert::TryInto,
    fmt::Write,
    iter::{repeat, Peekable, Repeat, Take},
    pin::Pin,
};
use kernel::{
    cstr,
    file_operations::{File, FileOpener, FileOperations},
    miscdev, mutex_init, seq_file,
    sync::Mutex,
    user_ptr::UserSlicePtrWriter,
};

use kernel::prelude::*;

module! {
    type: RustSeqFileDev,
    name: b"rust_seq_file",
    author: b"Adam Bratschi-Kaye",
    description: b"Rust sample using a seq_file",
    license: b"GPL v2",
    params: {
    },
}

#[derive(Clone)]
struct SharedState(Arc<Mutex<u32>>);

impl SharedState {
    fn try_new() -> KernelResult<Self> {
        let state = Arc::try_new(
            // SAFETY: `mutex_init!` is called below.
            unsafe { Mutex::new(0) },
        )?;
        // SAFETY: Mutex is pinned behind `Arc`.
        let pin_state = unsafe { Pin::new_unchecked(state.as_ref()) };
        mutex_init!(pin_state, "SharedState::0");
        Ok(SharedState(state))
    }
}

impl seq_file::SeqOperations for SharedState {
    type Item = String;
    type Iterator = Take<Repeat<String>>;
    type Wrapper = Box<Peekable<Take<Repeat<String>>>>;

    fn start(arg: &SharedState) -> Option<Self::Wrapper> {
        let count = arg.0.lock();
        let mut message = String::new();

        let template = "rust_seq_file: device opened this many times: ";
        message.try_reserve_exact(template.len() + 4).ok()?;
        // NO PANIC: We reserved space for `template` above.
        message.push_str(template);
        if *count < 1000 {
            // NO PANIC: There are 4 characters remaining in the string which
            // leaves space for a 3 digit number and the newline.
            write!(&mut message, "{}\n", *count).ok()?;
        }

        Box::try_new(repeat(message).take((*count).try_into().ok()?).peekable()).ok()
    }
}

impl FileOpener<SharedState> for SharedState {
    fn open(ctx: &SharedState) -> KernelResult<Self::Wrapper> {
        pr_info!("rust seq_file was opened!\n");
        Ok(Box::try_new(ctx.clone())?)
    }
}

impl FileOperations for SharedState {
    type Wrapper = Box<Self>;

    kernel::declare_file_operations!(read);

    fn read(&self, _: &File, data: &mut UserSlicePtrWriter, offset: u64) -> KernelResult<usize> {
        let message = b"incremented read count\n";
        if offset != 0 {
            return Ok(0);
        }

        {
            let mut count = self.0.lock();
            *count += 1;
        }

        data.write_slice(message)?;
        Ok(message.len())
    }
}

struct RustSeqFileDev {
    _seq: Pin<Box<seq_file::SeqFile<SharedState>>>,
    _dev: Pin<Box<miscdev::Registration<SharedState>>>,
}

impl KernelModule for RustSeqFileDev {
    fn init() -> KernelResult<Self> {
        pr_info!("Rust seq_file sample (init)\n");

        let state = SharedState::try_new()?;

        let seq_reg =
            kernel::seq_file::proc_create::<SharedState>(cstr!("rust_seq_file"), state.clone())?;

        let dev_reg =
            miscdev::Registration::new_pinned::<SharedState>(cstr!("rust_seq_file"), None, state)?;

        let dev = RustSeqFileDev {
            _seq: seq_reg,
            _dev: dev_reg,
        };

        Ok(dev)
    }
}

impl Drop for RustSeqFileDev {
    fn drop(&mut self) {
        pr_info!("Rust seq_file sample (exit)\n");
    }
}
