// SPDX-License-Identifier: GPL-2.0

//! The `kernel` prelude

/// ICE if we use `--extern module`
extern crate module;

pub use alloc::{borrow::ToOwned, string::String};

pub use module::module;

pub use super::{println, KernelModule, KernelResult};
