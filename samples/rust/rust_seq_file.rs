// SPDX-License-Identifier: GPL-2.0

//! Rust miscellaneous device sample.

use kernel::prelude::*;
use kernel::{
    file::File,
    file_operations::FileOperations,
    io_buffer::{IoBufferReader, IoBufferWriter},
    miscdev, seq_file,
    seq_file::SeqFileDebugFsDirEntry,
    sync::{CondVar, Mutex, Ref, RefBorrow, UniqueRef},
};

module! {
    type: RustMiscdev,
    name: b"rust_miscdev",
    author: b"Rust for Linux Contributors",
    description: b"Rust miscellaneous device sample",
    license: b"GPL v2",
}

const MAX_TOKENS: usize = 3;

struct SharedStateInner {
    token_count: usize,
}

struct SharedState {
    state_changed: CondVar,
    inner: Mutex<SharedStateInner>,
}

impl SharedState {
    fn try_new() -> Result<Ref<Self>> {
        let mut state = Pin::from(UniqueRef::try_new(Self {
            // SAFETY: `condvar_init!` is called below.
            state_changed: unsafe { CondVar::new() },
            // SAFETY: `mutex_init!` is called below.
            inner: unsafe { Mutex::new(SharedStateInner { token_count: 0 }) },
        })?);

        // SAFETY: `state_changed` is pinned when `state` is.
        let pinned = unsafe { state.as_mut().map_unchecked_mut(|s| &mut s.state_changed) };
        kernel::condvar_init!(pinned, "SharedState::state_changed");

        // SAFETY: `inner` is pinned when `state` is.
        let pinned = unsafe { state.as_mut().map_unchecked_mut(|s| &mut s.inner) };
        kernel::mutex_init!(pinned, "SharedState::inner");

        Ok(state.into())
    }
}

struct Token;
impl FileOperations for Token {
    type Wrapper = Ref<SharedState>;
    type OpenData = Ref<SharedState>;

    kernel::declare_file_operations!(read, write);

    fn open(shared: &Ref<SharedState>, _file: &File) -> Result<Self::Wrapper> {
        Ok(shared.clone())
    }

    fn read(
        shared: RefBorrow<'_, SharedState>,
        _: &File,
        data: &mut impl IoBufferWriter,
        offset: u64,
    ) -> Result<usize> {
        // Succeed if the caller doesn't provide a buffer or if not at the start.
        if data.is_empty() || offset != 0 {
            return Ok(0);
        }

        {
            let mut inner = shared.inner.lock();

            // Wait until we are allowed to decrement the token count or a signal arrives.
            while inner.token_count == 0 {
                if shared.state_changed.wait(&mut inner) {
                    return Err(Error::EINTR);
                }
            }

            // Consume a token.
            inner.token_count -= 1;
        }

        // Notify a possible writer waiting.
        shared.state_changed.notify_all();

        // Write a one-byte 1 to the reader.
        data.write_slice(&[1u8; 1])?;
        Ok(1)
    }

    fn write(
        shared: RefBorrow<'_, SharedState>,
        _: &File,
        data: &mut impl IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {
        {
            let mut inner = shared.inner.lock();

            // Wait until we are allowed to increment the token count or a signal arrives.
            while inner.token_count == MAX_TOKENS {
                if shared.state_changed.wait(&mut inner) {
                    return Err(Error::EINTR);
                }
            }

            // Increment the number of token so that a reader can be released.
            inner.token_count += 1;
        }

        // Notify a possible reader waiting.
        shared.state_changed.notify_all();
        Ok(data.len())
    }
}

impl seq_file::SeqOperations for Token {
    type Item = usize;
    type OpenData = Ref<SharedState>;
    type DataWrapper = Ref<SharedState>;
    type IteratorWrapper = Box<(usize, usize)>;

    fn open<'a>(open_data: RefBorrow<'a, SharedState>) -> Result<Ref<SharedState>> {
        Ok(open_data.into())
    }

    fn start(data: &Self::DataWrapper) -> Option<Self::IteratorWrapper> {
        let total = data.inner.lock().token_count;
        Box::try_new((total, 1)).ok()
    }

    fn next(iterator: &mut Self::IteratorWrapper) -> bool {
        let total = iterator.0;
        let current = iterator.1;
        if total == current {
            false
        } else {
            iterator.1 += 1;
            true
        }
    }

    fn current(iterator: &Self::IteratorWrapper) -> core::option::Option<Self::Item> {
        let total = iterator.0;
        let current = iterator.1;
        if total > current {
            Some(current)
        } else {
            None
        }
    }
}

struct RustMiscdev {
    _dev: Pin<Box<miscdev::Registration<Token>>>,
    _debugfs: SeqFileDebugFsDirEntry<Token>,
}

impl KernelModule for RustMiscdev {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust miscellaneous device sample (init)\n");

        let state = SharedState::try_new()?;
        let debugfs = seq_file::debugfs_create_file(fmt!("{name}"), state.clone())?;

        Ok(RustMiscdev {
            _dev: miscdev::Registration::new_pinned(fmt!("{name}"), state)?,
            _debugfs: debugfs,
        })
    }
}

impl Drop for RustMiscdev {
    fn drop(&mut self) {
        pr_info!("Rust seq file device sample (exit)\n");
    }
}
