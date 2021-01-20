// SPDX-License-Identifier: GPL-2.0
//
// A hack to export Rust symbols for loadable modules without having to redo
// the entire `include/linux/export.h` logic in Rust.
//
// Note that this requires `-Z symbol-mangling-version=v0` because Rust's
// default ("legacy") mangling scheme 1) uses a hash suffix which cannot
// be predicted across compiler versions and 2) uses invalid characters
// for C identifiers (thus we cannot use the `EXPORT_SYMBOL_*` macros).

#include <linux/module.h>

#define EXPORT_SYMBOL_RUST(sym)     extern int sym; EXPORT_SYMBOL(sym);
#define EXPORT_SYMBOL_RUST_GPL(sym) extern int sym; EXPORT_SYMBOL_GPL(sym);

#include "exports_core_generated.h"
#include "exports_alloc_generated.h"
#include "exports_kernel_generated.h"
#include "exports_compiler_builtins_generated.h"
