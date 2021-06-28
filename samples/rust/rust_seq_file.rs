// SPDX-License-Identifier: GPL-2.0

//! Example of using a [`seq_file`] in Rust.
//!
//! C header: [`include/linux/seq_file.h`](../../../include/linux/seq_file.h)
//! C header: [`include/linux/proc_fs.h`](../../../include/linux/proc_fs.h)
//!
//! Reference: <https://www.kernel.org/doc/html/latest/filesystems/seq_file.html>

#![no_std]
#![feature(allocator_api, global_asm, try_reserve)]

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
    file::File,
    file_operations::{FileOpener, FileOperations},
    io_buffer::IoBufferWriter,
    miscdev, mutex_init,
    prelude::*,
    proc_fs, seq_file,
    sync::{Mutex, Ref, RefBorrow},
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
    type Item = String;
    type Iterator = Take<Repeat<String>>;
    type DataWrapper = Pin<Ref<State>>;
    type IteratorWrapper = Box<Peekable<Self::Iterator>>;

    fn display(item: &Self::Item) -> &str {
        &item[..]
    }

    fn start(state: RefBorrow<State>) -> Result<Self::IteratorWrapper> {
        const MAX_DIGITS: usize = 3;
        const MAX_LENGTH: usize = MAX_DIGITS + 1;
        const MAX_COUNT: u32 = 10u32.pow(MAX_DIGITS as u32) - 1;

        let count = state.0.lock();
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

impl FileOpener<Pin<Ref<State>>> for Token {
    fn open(ctx: &Pin<Ref<State>>) -> Result<Self::Wrapper> {
        pr_info!("rust seq_file was opened!\n");
        Ok(ctx.clone())
    }
}

impl FileOperations for Token {
    kernel::declare_file_operations!(read);

    type Wrapper = Pin<Ref<State>>;

    fn read<T: IoBufferWriter>(shared: &Ref<State>, _: &File, _: &mut T, _: u64) -> Result<usize> {
        *(shared.0.lock()) += 1;
        Ok(0)
    }
}

struct RustSeqFileDev {
    _proc: proc_fs::ProcDirEntry<Pin<Ref<State>>>,
    _dev: Pin<Box<miscdev::Registration<Pin<Ref<State>>>>>,
}

impl KernelModule for RustSeqFileDev {
    fn init() -> Result<Self> {
        pr_info!("Rust seq_file sample (init)\n");

        let state = State::try_new()?;

        let proc_dir_entry = proc_fs::ProcDirEntry::new_seq_private::<State>(
            c_str!("rust_seq_file"),
            state.clone(),
        )?;

        let dev_reg =
            miscdev::Registration::new_pinned::<Token>(c_str!("rust_seq_file"), None, state)?;

        let dev = RustSeqFileDev {
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
