#![no_std]

pub extern crate ffi;

pub mod prelude {
    pub use super::ffi;
    pub use super::{aadd, aload, astore, asub, cas, try_box_slice};
}

#[macro_export]
macro_rules! aadd {
    ($a:expr, $v:expr) => {{
        use ffi::atomic_fetch_add_u64;
        #[allow(unused_unsafe)]
        unsafe {
            atomic_fetch_add_u64($a, $v)
        }
    }};
}

#[macro_export]
macro_rules! asub {
    ($a:expr, $v:expr) => {{
        use ffi::atomic_fetch_sub_u64;
        #[allow(unused_unsafe)]
        unsafe {
            atomic_fetch_sub_u64($a, $v)
        }
    }};
}

#[macro_export]
macro_rules! aload {
    ($a:expr) => {{
        use ffi::atomic_load_u64;
        #[allow(unused_unsafe)]
        unsafe {
            atomic_load_u64($a)
        }
    }};
}

#[macro_export]
macro_rules! astore {
    ($a:expr, $v:expr) => {{
        use ffi::atomic_store_u64;
        #[allow(unused_unsafe)]
        unsafe {
            atomic_store_u64($a, $v)
        }
    }};
}

#[macro_export]
macro_rules! cas {
    ($v:expr, $expect:expr, $desired:expr) => {{
        use ffi::cas_release;
        #[allow(unused_unsafe)]
        unsafe {
            cas_release($v, $expect, $desired)
        }
    }};
}

#[macro_export]
macro_rules! try_box_slice {
    ($value:expr, $len:expr) => {{
        use core::mem::size_of_val;
        use core::ptr::write;
        use core::slice::from_raw_parts_mut;
        use ffi::alloc;
        let count = $len;
        let elem_size = size_of_val(&$value);
        let total_size = elem_size * count;
        if total_size == 0 {
            err!(IllegalState)
        } else {
            #[allow(unused_unsafe)]
            unsafe {
                let rptr = alloc(total_size) as *mut u8;
                if rptr.is_null() {
                    err!(Alloc)
                } else {
                    let mut write_ptr = rptr;
                    for _ in 0..count {
                        write(write_ptr as *mut _, $value);
                        write_ptr = write_ptr.add(elem_size);
                    }
                    Ok(Box::from_raw(Ptr::new(from_raw_parts_mut(
                        rptr as *mut _,
                        count,
                    ))))
                }
            }
        }
    }};
}
