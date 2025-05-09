#![no_std]

pub extern crate ffi;

pub mod prelude {
    pub use super::ffi;
    pub use super::{aadd, aload, astore, asub, cas, exit, try_box_slice};
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

#[macro_export]
macro_rules! writef {
    ($f:expr, $fmt:expr) => {{
        writef!($f, "{}", $fmt)
    }};
    ($f:expr, $fmt:expr, $($t:expr),*) => {{
        use core::str::from_utf8_unchecked;
        let mut err = Error::new(Unknown.code(), || { "Unknown" }, Backtrace::init());
        let fmt_str = $fmt;
        let fmt_bytes = fmt_str.as_bytes();
        let mut cur = 0;
        $(
            unsafe { ffi::write(2, "z\n".as_ptr(), 2); }
            match fmt_str.findn("{}", cur) {
                Some(index) => {
                    unsafe { ffi::write(2, "a\n".as_ptr(), 2); }
                    if index > cur {
                        let bytes = &fmt_bytes[cur..(index+cur)];
                        #[allow(unused_unsafe)]
                        let s = unsafe { from_utf8_unchecked(bytes) };
                        match $f.append(s) {
                            Ok(_) => {},
                            Err(e) => err = e,
                        }
                    }

                    unsafe { ffi::write(2, "x\n".as_ptr(), 2); }
                    cur = index + 2;
                    match $t.format($f) {
                        Ok(_) => {},
                        Err(e) => err = e,
                    }
                    unsafe { ffi::write(2, "y\n".as_ptr(), 2); }
                }
                None => {unsafe { ffi::write(2, "b\n".as_ptr(), 2); }},
            }
        )*
        if cur < fmt_str.len() {
            unsafe { ffi::write(2, "c\n".as_ptr(), 2);}
            let bytes = &fmt_bytes[cur..fmt_str.len()];
            #[allow(unused_unsafe)]
            let s = unsafe { from_utf8_unchecked(bytes) };
            match $f.append(s) {
                Ok(_) => {},
                Err(e) => err = e,
            }
        }
        if err == Unknown {
            Ok(())
        } else {
            Err(err)
        }
    }};
}

/*
#[macro_export]
macro_rules! format {
        ($fmt:expr) => {{
                format!("{}", $fmt)
        }};
        ($fmt:expr, $($t:expr),*) => {{
                let mut formatter = Formatter::new();
                match writef!(&mut formatter, $fmt, $($t),*) {
                    Ok(_) => Ok(formatter.to_str()),
                    Err(e) => Err(e)
                }
        }};
}
*/

#[macro_export]
macro_rules! println {
    ($fmt:expr) => {{
        println!("{}", $fmt)
    }};
    ($fmt:expr, $($t:expr),*) => {{
        let mut formatter = Formatter::new();

        match writef!(&mut formatter, $fmt, $($t),*) {
            Ok(_) => {
                let s = formatter.to_str();
                #[allow(unused_unsafe)]
                unsafe {
                        ffi::write(2, s.as_ptr(), s.len());
                        ffi::write(2, "\n".as_ptr(), 1);
                }
                Ok(())
            },
            Err(e) => Err(e),
        }
    }};
}

#[macro_export]
macro_rules! exit {
    ($fmt:expr) => {{
        exit!("{}", $fmt);
    }};
    ($fmt:expr,  $($t:expr),*) => {{
        /*
            print!("Panic[@{}:{}]: ", file!(), line!());
            println!($fmt, $($t),*);
            let bt = Backtrace::new();
            println!("{}", bt);
        */

        #[allow(unused_unsafe)]
        unsafe {
            use ffi::exit;
            exit(-1);
        }
        loop {}
    }};
}
