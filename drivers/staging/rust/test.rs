#![no_std]
#![feature(global_asm)]
#![feature(alloc_error_handler)]

use core::panic::PanicInfo;

extern crate alloc;
use core::alloc::{GlobalAlloc, Layout};
//use alloc::alloc::{GlobalAlloc, Layout};
//use std::alloc::{GlobalAlloc, Layout};
//use alloc::borrow::ToOwned;
//use alloc::string::String;
use alloc::boxed::Box;

#[panic_handler]
fn my_panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

extern "C" {
    fn printk(format: *const u8, ...) -> i32;
    fn panic(format: *const u8);
    // TODO: check param types
    fn __kmalloc(size: usize, flags: u32) -> *mut u8;
    fn kfree(ptr: *const u8);
}

pub struct KMallocator;

unsafe impl GlobalAlloc for KMallocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let gfp_io: u32 = 0x40;
        let gfp_fs: u32 = 0x80;
        let gfp_direct_reclaim: u32 = 0x400;
        let gfp_kswapd_reclaim: u32 = 0x800;
        let gfp_reclaim: u32 = gfp_direct_reclaim | gfp_kswapd_reclaim;
        let gfp_kernel: u32 = gfp_reclaim | gfp_io | gfp_fs;
        printk("\x014XXX: custom alloc impl\n\0".as_ptr());
        // void* __kmalloc(size_t, gfp_t);
        __kmalloc(layout.size(), gfp_kernel) as *mut u8
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        kfree(ptr as *const u8);
    }
}

#[global_allocator]
static KMALLOCATOR: KMallocator = KMallocator;

#[alloc_error_handler]
fn oom(_layout: Layout) -> ! {
    loop {}
}

#[no_mangle]
#[link_section = ".init.text"]
pub extern "C" fn my_init() -> i32 {
    unsafe {
        printk("\x014XXX: hello world rust\n\0".as_ptr());
        //panic("rust panic\n\0".as_ptr());
        //let foo: String = "\x014XXX: heap allocated str\n\0".to_owned();
        //printk(foo.as_ptr());

        // TODO: linkage failure
        //let x = Box::new(42);
        //printk("\x014XXX: dynamically allocated %s\n\0".as_ptr(), x);
    }
    0
}

// needed for static compilation
global_asm!(r#"
.section .initcall6.init, "a"
__initcall_my_init6:
.long my_init - .
.previous
"#);


// No symbol aliasing, should this alias my_init?
// needed for .init member of test.mod.c
#[no_mangle]
pub extern "C" fn init_module() -> i32 {
    my_init()
}

#[no_mangle]
pub extern "C" fn __inittest() -> *const u8 {
    my_init as *const u8
}

#[no_mangle]
#[link_section = ".exit.text"]
pub extern "C" fn my_exit() {
    unsafe {
        printk("\x014XXX: goodbye world rust\n\0".as_ptr());
    }
}

// needed for .exit member of test.mod.c
#[no_mangle]
pub extern "C" fn cleanup_module() {
    my_exit();
}


// TODO: sort this out so we don't taint the kernel.
#[link_section = ".modinfo"]
#[export_name = "__UNIQUE_ID_file"]
static FILE: &'static [u8] = "test.file=drivers/staging/rust/test\0".as_bytes();
#[link_section = ".modinfo"]
#[export_name = "__UNIQUE_ID_author"]
static AUTHOR: &'static [u8] = "test.author=Nick Desaulniers\0".as_bytes();
#[link_section = ".modinfo"]
#[export_name = "__UNIQUE_ID_description"]
static DESC: &'static [u8] = "test.description=Rust hello world\0".as_bytes();
#[link_section = ".modinfo"]
#[export_name = "__UNIQUE_ID_license"]
static license: &'static [u8] = "test.license=GPL\0".as_bytes();
