// SPDX-License-Identifier: GPL-2.0

//! Example of using a [`seq_file`] in Rust.
//!
//! C header: [`include/linux/seq_file.h`](../../../include/linux/seq_file.h)
//! C header: [`include/linux/proc_fs.h`](../../../include/linux/proc_fs.h)
//!
//! Reference: <https://www.kernel.org/doc/html/latest/filesystems/seq_file.html>

#![feature(allocator_api, global_asm)]

use alloc::boxed::Box;
use core::{
    cmp::min,
    convert::TryInto,
    fmt::Write,
    iter::{repeat, Peekable, Repeat, Take},
    pin::Pin,
};
use kernel::{
    c_str,
    debugfs::DebugFsDirEntry,
    file::File,
    file_operations::FileOperations,
    io_buffer::IoBufferWriter,
    miscdev, mutex_init,
    prelude::*,
    seq_file,
    sync::{Mutex, Ref},
    Error, Result,
};

module! {
    type: RustSeqFileDev,
    name: b"rust_seq_file",
    author: b"Adam Bratschi-Kaye",
    description: b"Rust sample using a seq_file",
    license: b"GPL v2",
}

struct State(Mutex<u32>);

impl State {
    fn try_new() -> Result<Pin<Ref<Self>>> {
        Ok(Ref::pinned(Ref::try_new_and_init(
            unsafe { State(Mutex::new(0)) },
            |mut state| {
                // SAFETY: Mutex is pinned behind `Ref`.
                let pin_state = unsafe { state.as_mut().map_unchecked_mut(|s| &mut s.0) };
                mutex_init!(pin_state, "State::0");
            },
        )?))
    }
}

impl seq_file::SeqOperations for State {
    type Item = u32;
    type DataWrapper = Pin<Ref<State>>;
    type IteratorWrapper = Box<Peekable<Take<Repeat<u32>>>>;

    fn start(&self) -> Option<Self::IteratorWrapper> {
        const MAX_DIGITS: usize = 3;
        const MAX_LENGTH: usize = MAX_DIGITS + 1;
        const MAX_COUNT: u32 = 10u32.pow(MAX_DIGITS as u32) - 1;

        let count = self.0.lock();
        let mut message = String::new();

        let template = if *count <= MAX_COUNT {
            "rust_seq_file: device opened this many times: "
        } else {
            "rust_seq_file: device opened at least this many times: "
        };
        message.try_reserve_exact(template.len() + MAX_LENGTH)?;
        // NOPANIC: We reserved space for `template` above.
        message.push_str(template);
        let message_count = min(*count, MAX_COUNT);
        // NOPANIC: There are `MAX_LENGTH` characters remaining in the string which
        // leaves space for a `MAX_DIGITS` digit number and the newline.
        // `message_count` is `<= MAX_COUNT` means it has less than `MAX_DIGITS`
        // digits.
        writeln!(&mut message, "{}", message_count).map_err(|_| Error::ENOMEM)?;

        Box::try_new(repeat(message).take((*count).try_into()?).peekable())
            .map_err(|_| Error::ENOMEM)
    }
}

struct Token;

impl FileOperations for Token {
    type Wrapper = Pin<Ref<State>>;
    type OpenData = Pin<Ref<State>>;

    kernel::declare_file_operations!(read);

    fn open(state: &Pin<Ref<State>>, _file: &File) -> Result<Self::Wrapper> {
        Ok(state.clone())
    }

    fn read<T: IoBufferWriter>(shared: &Ref<State>, _: &File, _: &mut T, _: u64) -> Result<usize> {
        *(shared.0.lock()) += 1;
        Ok(0)
    }
}

struct RustSeqFileDev {
    _debugfs: DebugFsDirEntry<Pin<Ref<State>>>,
    _dev: Pin<Box<miscdev::Registration<Token>>>,
}

impl KernelModule for RustSeqFileDev {
    fn init() -> Result<Self> {
        pr_info!("Rust seq_file sample (init)\n");

        let state = State::try_new()?;

        let debugfs_dir_entry =
            seq_file::debugfs_create_file(c_str!("rust_seq_file"), state.clone())?;

        let dev_reg =
            miscdev::Registration::new_pinned::<Token>(c_str!("rust_seq_file"), None, state)?;

        let dev = RustSeqFileDev {
            _debugfs: debugfs_dir_entry,
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
