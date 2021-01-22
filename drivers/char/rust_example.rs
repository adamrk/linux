// SPDX-License-Identifier: GPL-2.0

#![no_std]
#![feature(allocator_api, global_asm)]

use alloc::boxed::Box;
use core::pin::Pin;
use kernel::prelude::*;
use kernel::{cstr, file_operations::FileOperations, miscdev};

// static mut __MOD: Option<RustExample> = None;
// global_asm!( r#".section ".initcall6.init", "a"
//                 __rust_example_initcall:
//                     .long   __rust_example_init - .
//                     .previous"#);
                
//     #[cfg(not(MODULE))]
//     #[no_mangle]
//     pub extern "C" fn __rust_example_init() -> kernel::c_types::c_int {
//         __init()
//     }
//     #[cfg(not(MODULE))]
//     #[no_mangle]
//     pub extern "C" fn __rust_example_exit() { __exit() }
//     fn __init() -> kernel::c_types::c_int {
//         match <RustExample as KernelModule>::init() {
//             Ok(m) => { unsafe { __MOD = Some(m); } return 0; }
//             Err(e) => { return e.to_kernel_errno(); }
//         }
//     }
//     fn __exit() { unsafe { __MOD = None; } }
//     #[cfg(not(MODULE))]
//     #[link_section = ".modinfo"]
//     #[used]
//     pub static __rust_example_author: [u8; 48] =
//         *b"rust_example.author=Rust for Linux Contributors\0";
//     #[cfg(not(MODULE))]
//     #[link_section = ".modinfo"]
//     #[used]
//     pub static __rust_example_description: [u8; 66] =
//         *b"rust_example.description=An example kernel module written in Rust\0";
//     #[cfg(not(MODULE))]
//     #[link_section = ".modinfo"]
//     #[used]
//     pub static __rust_example_license: [u8; 28] =
//         *b"rust_example.license=GPL v2\0";
//     #[cfg(not(MODULE))]
//     #[link_section = ".modinfo"]
//     #[used]
//     pub static __rust_example_file: [u8; 44] =
//         *b"rust_example.file=drivers/char/rust_example\0";
//     #[cfg(not(MODULE))]
//     #[link_section = ".modinfo"]
//     #[used]
//     pub static __rust_example_parmtype_my_bool: [u8; 35] =
//         *b"rust_example.parmtype=my_bool:bool\0";
//     #[cfg(not(MODULE))]
//     #[link_section = ".modinfo"]
//     #[used]
//     pub static __rust_example_parm_my_bool: [u8; 42] =
//         *b"rust_example.parm=my_bool:Example of bool\0";
//     static mut __rust_example_my_bool_value: bool = true;
//     struct __rust_example_my_bool;
//     impl __rust_example_my_bool {
//         fn read(&self) -> bool { unsafe { __rust_example_my_bool_value } }
//     }
//     const my_bool: __rust_example_my_bool = __rust_example_my_bool;
//     #[repr(transparent)]
//     struct __rust_example_my_bool_RacyKernelParam(kernel::bindings::kernel_param);
//     unsafe impl Sync for __rust_example_my_bool_RacyKernelParam { }
//     #[cfg(not(MODULE))]
//     const __rust_example_my_bool_name: *const kernel::c_types::c_char =
//         b"rust_example.my_bool\0" as *const _ as
//             *const kernel::c_types::c_char;
//     #[link_section = "__param"]
//     #[used]
//     static __rust_example_my_bool_struct:
//      __rust_example_my_bool_RacyKernelParam =
//         __rust_example_my_bool_RacyKernelParam(kernel::bindings::kernel_param{name:
//                                                                                   __rust_example_my_bool_name,
//                                                                               mod_:
//                                                                                   core::ptr::null_mut(),
//                                                                               ops:
//                                                                                   unsafe
//                                                                                   {
//                                                                                       &kernel::bindings::param_ops_bool
//                                                                                   }
//                                                                                       as
//                                                                                       *const kernel::bindings::kernel_param_ops,
//                                                                               perm:
//                                                                                   0,
//                                                                               level:
//                                                                                   -1,
//                                                                               flags:
//                                                                                   0,
//                                                                               __bindgen_anon_1:
//                                                                                   kernel::bindings::kernel_param__bindgen_ty_1{arg:
//                                                                                                                                    unsafe
//                                                                                                                                    {
//                                                                                                                                        &__rust_example_my_bool_value
//                                                                                                                                    }
//                                                                                                                                        as
//                                                                                                                                        *const _
//                                                                                                                                        as
//                                                                                                                                        *mut kernel::c_types::c_void,},});
//     #[cfg(not(MODULE))]
//     #[link_section = ".modinfo"]
//     #[used]
//     pub static __rust_example_parmtype_my_i32: [u8; 33] =
//         *b"rust_example.parmtype=my_i32:int\0";
//     #[cfg(not(MODULE))]
//     #[link_section = ".modinfo"]
//     #[used]
//     pub static __rust_example_parm_my_i32: [u8; 40] =
//         *b"rust_example.parm=my_i32:Example of i32\0";
//     static mut __rust_example_my_i32_value: i32 = 42;
//     struct __rust_example_my_i32;
//     impl __rust_example_my_i32 {
//         fn read(&self) -> i32 { unsafe { __rust_example_my_i32_value } }
//     }
//     const my_i32: __rust_example_my_i32 = __rust_example_my_i32;
//     #[repr(transparent)]
//     struct __rust_example_my_i32_RacyKernelParam(kernel::bindings::kernel_param);
//     unsafe impl Sync for __rust_example_my_i32_RacyKernelParam { }
//     #[cfg(not(MODULE))]
//     const __rust_example_my_i32_name: *const kernel::c_types::c_char =
//         b"rust_example.my_i32\0" as *const _ as
//             *const kernel::c_types::c_char;
//     #[link_section = "__param"]
//     #[used]
//     static __rust_example_my_i32_struct: __rust_example_my_i32_RacyKernelParam
//      =
//         __rust_example_my_i32_RacyKernelParam(kernel::bindings::kernel_param{name:
//                                                                                  __rust_example_my_i32_name,
//                                                                              mod_:
//                                                                                  core::ptr::null_mut(),
//                                                                              ops:
//                                                                                  unsafe
//                                                                                  {
//                                                                                      &kernel::bindings::param_ops_int
//                                                                                  }
//                                                                                      as
//                                                                                      *const kernel::bindings::kernel_param_ops,
//                                                                              perm:
//                                                                                  0o644,
//                                                                              level:
//                                                                                  -1,
//                                                                              flags:
//                                                                                  0,
//                                                                              __bindgen_anon_1:
//                                                                                  kernel::bindings::kernel_param__bindgen_ty_1{arg:
//                                                                                                                                   unsafe
//                                                                                                                                   {
//                                                                                                                                       &__rust_example_my_i32_value
//                                                                                                                                   }
//                                                                                                                                       as
//                                                                                                                                       *const _
//                                                                                                                                       as
//                                                                                                                                       *mut kernel::c_types::c_void,},});
//     #[cfg(not(MODULE))]
//     #[link_section = ".modinfo"]
//     #[used]
//     pub static __rust_example_parmtype_my_str: [u8; 36] =
//         *b"rust_example.parmtype=my_str:string\0";
//     #[cfg(not(MODULE))]
//     #[link_section = ".modinfo"]
//     #[used]
//     pub static __rust_example_parm_my_str: [u8; 51] =
//         *b"rust_example.parm=my_str:Example of a string param\0";
//     static mut __rust_example_my_str_value: [u8; 16] = *b"default str val ";
//     struct __rust_example_my_str;
//     impl __rust_example_my_str {
//         fn read(&self) -> Result<&str, core::str::Utf8Error> {
//             unsafe {
//                 let nul =
//                     __rust_example_my_str_value.iter().position(|&b|
//                                                                     b ==
//                                                                         b' ').unwrap();
//                 core::str::from_utf8(&__rust_example_my_str_value[0..nul])
//             }
//         }
//     }
//     const my_str: __rust_example_my_str = __rust_example_my_str;
//     #[repr(transparent)]
//     struct __rust_example_my_str_RacyKernelParam(kernel::bindings::kernel_param);
//     unsafe impl Sync for __rust_example_my_str_RacyKernelParam { }
//     #[cfg(not(MODULE))]
//     const __rust_example_my_str_name: *const kernel::c_types::c_char =
//         b"rust_example.my_str\0" as *const _ as
//             *const kernel::c_types::c_char;
//     #[link_section = "__param"]
//     #[used]
//     static __rust_example_my_str_struct: __rust_example_my_str_RacyKernelParam
//      =
//         __rust_example_my_str_RacyKernelParam(kernel::bindings::kernel_param{name:
//                                                                                  __rust_example_my_str_name,
//                                                                              mod_:
//                                                                                  core::ptr::null_mut(),
//                                                                              ops:
//                                                                                  unsafe
//                                                                                  {
//                                                                                      &kernel::bindings::param_ops_string
//                                                                                  }
//                                                                                      as
//                                                                                      *const kernel::bindings::kernel_param_ops,
//                                                                              perm:
//                                                                                  0o644,
//                                                                              level:
//                                                                                  -1,
//                                                                              flags:
//                                                                                  0,
//                                                                              __bindgen_anon_1:
//                                                                                  kernel::bindings::kernel_param__bindgen_ty_1{str_:
//                                                                                                                                   &kernel::bindings::kparam_string{maxlen:
//                                                                                                                                                                        unsafe
//                                                                                                                                                                        {
//                                                                                                                                                                            __rust_example_my_str_value.len()
//                                                                                                                                                                        }
//                                                                                                                                                                            as
//                                                                                                                                                                            u32
//                                                                                                                                                                            +
//                                                                                                                                                                            1,
//                                                                                                                                                                    string:
//                                                                                                                                                                        unsafe
//                                                                                                                                                                        {
//                                                                                                                                                                            (&__rust_example_my_str_value).as_ptr()
//                                                                                                                                                                        }
//                                                                                                                                                                            as
//                                                                                                                                                                            *mut kernel::c_types::c_char,}
//                                                                                                                                       as
//                                                                                                                                       *const _,},});
//     #[cfg(not(MODULE))]
//     #[link_section = ".modinfo"]
//     #[used]
//     pub static __rust_example_parmtype_my_charp: [u8; 37] =
//         *b"rust_example.parmtype=my_charp:charp\0";
//     #[cfg(not(MODULE))]
//     #[link_section = ".modinfo"]
//     #[used]
//     pub static __rust_example_parm_my_charp: [u8; 53] =
//         *b"rust_example.parm=my_charp:Example of a string param\0";
//     static mut __rust_example_my_charp_value: *mut u8 =
//         unsafe { b"default charp val " as *const u8 as *mut u8 };
//     struct __rust_example_my_charp;
//     impl __rust_example_my_charp {
//         fn read(&self) -> Result<&str, core::str::Utf8Error> {
//             unsafe {
//                 let mut count = 0;
//                 while *__rust_example_my_charp_value.add(count) != b'\0' {
//                     count += 1;
//                 }
//                 core::str::from_utf8(&core::slice::from_raw_parts(__rust_example_my_charp_value, count))
//             }
//         }
//     }
//     const my_charp: __rust_example_my_charp = __rust_example_my_charp;
//     #[repr(transparent)]
//     struct __rust_example_my_charp_RacyKernelParam(kernel::bindings::kernel_param);
//     unsafe impl Sync for __rust_example_my_charp_RacyKernelParam { }
//     #[cfg(not(MODULE))]
//     const __rust_example_my_charp_name: *const kernel::c_types::c_char =
//         b"rust_example.my_charp\0" as *const _ as
//             *const kernel::c_types::c_char;
//     #[link_section = "__param"]
//     #[used]
//     static __rust_example_my_charp_struct:
//      __rust_example_my_charp_RacyKernelParam =
//         __rust_example_my_charp_RacyKernelParam(kernel::bindings::kernel_param{name:
//                                                                                    __rust_example_my_charp_name,
//                                                                                mod_:
//                                                                                    core::ptr::null_mut(),
//                                                                                ops:
//                                                                                    unsafe
//                                                                                    {
//                                                                                        &kernel::bindings::param_ops_charp
//                                                                                    }
//                                                                                        as
//                                                                                        *const kernel::bindings::kernel_param_ops,
//                                                                                perm:
//                                                                                    0o644,
//                                                                                level:
//                                                                                    -1,
//                                                                                flags:
//                                                                                    0,
//                                                                                __bindgen_anon_1:
//                                                                                    kernel::bindings::kernel_param__bindgen_ty_1{arg:
//                                                                                                                                     unsafe
//                                                                                                                                     {
//                                                                                                                                         &__rust_example_my_charp_value
//                                                                                                                                     }
//                                                                                                                                         as
//                                                                                                                                         *const _
//                                                                                                                                         as
//                                                                                                                                         *mut kernel::c_types::c_void,},});

module! {
    type: RustExample,
    name: b"rust_example",
    author: b"Rust for Linux Contributors",
    description: b"An example kernel module written in Rust",
    license: b"GPL v2",
    params: {
        my_bool: bool {
            default: true,
            permissions: 0,
            description: b"Example of bool",
        },
        my_i32: i32 {
            default: 42,
            permissions: 0o644,
            description: b"Example of i32",
        },
        my_str: str {
            default: "default str val",
            permissions: 0o644,
            description: b"Example of a string param",
        },
    },
}

struct RustFile;

impl FileOperations for RustFile {
    type Wrapper = Box<Self>;

    fn open() -> KernelResult<Self::Wrapper> {
        println!("rust file was opened!");
        Ok(Box::try_new(Self)?)
    }
}

struct RustExample {
    message: String,
    _dev: Pin<Box<miscdev::Registration>>,
}

impl KernelModule for RustExample {
    fn init() -> KernelResult<Self> {
        println!("Rust Example (init)");
        println!("Am I built-in? {}", !cfg!(MODULE));
        println!("Parameters:");
        println!("  my_bool:  {}", my_bool.read());
        println!("  my_i32:   {}", my_i32.read());
        println!(
            "  my_str:   {}",
            my_str.read().expect("Expected valid UTF8 parameter")
        );

        Ok(RustExample {
            message: "on the heap!".to_owned(),
            _dev: miscdev::Registration::new_pinned::<RustFile>(cstr!("rust_miscdev"), None)?,
        })
    }
}

impl Drop for RustExample {
    fn drop(&mut self) {
        println!("My message is {}", self.message);
        println!("Rust Example (exit)");
    }
}
