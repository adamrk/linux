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
    io_buffer::IoBufferWriter,
    miscdev, mutex_init,
    prelude::*,
    proc_fs, seq_file,
    sync::Mutex,
};

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

    fn display(item: &Self::Item) -> &str {
        &item[..]
    }

    fn start(arg: &SharedState) -> Option<Box<Peekable<Self::Iterator>>> {
        const MAX_DIGITS: usize = 3;
        const MAX_LENGTH: usize = MAX_DIGITS + 1;
        const MAX_COUNT: u32 = 10u32.pow(MAX_DIGITS as u32);

        let count = arg.0.lock();
        let mut message = String::new();

        let template = "rust_seq_file: device opened this many times: ";
        message
            .try_reserve_exact(template.len() + MAX_LENGTH)
            .ok()?;
        // NOPANIC: We reserved space for `template` above.
        message.push_str(template);
        if *count < MAX_COUNT {
            // NOPANIC: There are MAX_LENGTH characters remaining in the string which
            // leaves space for a MAX_DIGITS digit number and the newline.
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

    fn read<T: IoBufferWriter>(&self, _: &File, data: &mut T, offset: u64) -> KernelResult<usize> {
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
    #[cfg(CONFIG_PROC_FS)]
    _proc: proc_fs::ProcDirEntry<SharedState>,
    _dev: Pin<Box<miscdev::Registration<SharedState>>>,
}

impl KernelModule for RustSeqFileDev {
    fn init() -> KernelResult<Self> {
        pr_info!("Rust seq_file sample (init)\n");

        let state = SharedState::try_new()?;

        #[cfg(CONFIG_PROC_FS)]
        let proc_dir_entry =
            seq_file::proc_create_seq::<SharedState>(cstr!("rust_seq_file"), state.clone())?;

        let dev_reg =
            miscdev::Registration::new_pinned::<SharedState>(cstr!("rust_seq_file"), None, state)?;

        let dev = RustSeqFileDev {
            #[cfg(CONFIG_PROC_FS)]
            _proc: proc_dir_entry,
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
