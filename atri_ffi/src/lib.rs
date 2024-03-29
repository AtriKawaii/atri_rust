use std::mem::{ManuallyDrop, MaybeUninit};
use std::ptr::null_mut;
use std::{mem, slice};

pub mod closure;
pub mod contact;
pub mod error;
pub mod ffi;
pub mod future;
mod managed;
pub mod message;
pub mod plugin;
pub use managed::*;

pub type Handle = *const ();
pub type PHandle = *const Handle;

#[repr(C)]
pub struct RustString {
    pub ptr: *mut u8,
    pub len: usize,
    pub capacity: usize,
}

impl RustString {
    pub fn null() -> Self {
        Self {
            ptr: null_mut(),
            len: 0,
            capacity: 0,
        }
    }

    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }
}

impl From<String> for RustString {
    fn from(s: String) -> Self {
        let mut ma = ManuallyDrop::new(s);
        let ptr = ma.as_mut_ptr();
        let len = ma.len();
        let cap = ma.capacity();

        Self {
            ptr,
            len,
            capacity: cap,
        }
    }
}

impl From<RustString> for String {
    fn from(s: RustString) -> Self {
        let str = unsafe { String::from_raw_parts(s.ptr, s.len, s.capacity) };
        str
    }
}

impl AsRef<str> for RustString {
    fn as_ref(&self) -> &str {
        unsafe {
            let slice = slice::from_raw_parts(self.ptr, self.len);
            std::str::from_utf8_unchecked(slice)
        }
    }
}

impl ToString for RustString {
    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }
}

#[repr(C)]
pub struct RustStr {
    pub slice: *const u8,
    pub len: usize,
}

impl RustStr {
    pub fn as_str<'a>(&self) -> &'a str {
        unsafe {
            let slice = slice::from_raw_parts(self.slice, self.len);
            std::str::from_utf8_unchecked(slice)
        }
    }
}

impl From<&str> for RustStr {
    fn from(s: &str) -> Self {
        Self {
            slice: s.as_ptr(),
            len: s.len(),
        }
    }
}

impl AsRef<str> for RustStr {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl ToString for RustStr {
    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }
}

#[repr(C)]
pub struct RustVec<T: ?Sized> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
}

unsafe impl<T: Send> Send for RustVec<T> {}
unsafe impl<T: Sync> Sync for RustVec<T> {}

impl<T> RustVec<T> {
    pub fn into_vec(self) -> Vec<T> {
        unsafe { Vec::from_raw_parts(self.ptr, self.len, self.capacity) }
    }
}

impl<T> From<Vec<T>> for RustVec<T> {
    fn from(mut v: Vec<T>) -> Self {
        let (ptr, len, cap) = (v.as_mut_ptr(), v.len(), v.capacity());
        mem::forget(v);
        Self {
            ptr,
            len,
            capacity: cap,
        }
    }
}

#[repr(C)]
pub struct RustSlice<T> {
    ptr: *const T,
    len: usize,
}

impl<T> RustSlice<T> {
    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl<T> From<&[T]> for RustSlice<T> {
    fn from(slice: &[T]) -> Self {
        Self {
            ptr: slice.as_ptr(),
            len: slice.len(),
        }
    }
}

unsafe impl<T: Send> Send for RustSlice<T> {}
unsafe impl<T: Sync> Sync for RustSlice<T> {}

#[repr(C)]
pub struct FFIOption<T> {
    is_some: bool,
    value: MaybeUninit<T>,
}

unsafe impl<T: Send> Send for FFIOption<T> {}
unsafe impl<T: Sync> Sync for FFIOption<T> {}

impl<T> From<Option<T>> for FFIOption<T> {
    fn from(val: Option<T>) -> Self {
        match val {
            Some(t) => Self {
                is_some: true,
                value: MaybeUninit::new(t),
            },
            None => Self {
                is_some: false,
                value: MaybeUninit::uninit(),
            },
        }
    }
}

impl<T> From<FFIOption<T>> for Option<T> {
    fn from(ffi: FFIOption<T>) -> Self {
        if ffi.is_some {
            unsafe { Some(ffi.value.assume_init()) }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Managed, RustStr, RustString, RustVec};

    #[test]
    fn vec() {
        let v = vec![1, 1, 4, 5, 1, 4];
        let raw = RustVec::from(v);
        let v = raw.into_vec();

        assert_eq!(v, [1, 1, 4, 5, 1, 4]);
    }

    #[test]
    fn string() {
        let s = String::from("114514");
        let raw = RustString::from(s);
        let s = String::from(raw);

        assert_eq!(s, "114514");

        let slice = &s[1..];
        let raw = RustStr::from(slice);
        let slice = raw.as_ref();

        assert_eq!(slice, "14514");
    }

    #[test]
    fn managed_value() {
        #[derive(Debug, Clone)]
        struct Test {
            a: i32,
            b: usize,
            c: Option<Box<(Test, Test)>>,
        }

        impl PartialEq for Test {
            fn eq(&self, other: &Self) -> bool {
                if self.a != other.a {
                    return false;
                }
                if self.b != other.b {
                    return false;
                }

                self.c == other.c
            }
        }

        let test = Test {
            a: 233,
            b: 114514,
            c: Some(Box::new((
                Test {
                    a: 23114,
                    b: 114514,
                    c: None,
                },
                Test {
                    a: 114514,
                    b: 2333,
                    c: None,
                },
            ))),
        };
        let test0 = test.clone();
        let managed = Managed::from_value(test);
        let test: Test = unsafe { managed.into_value() };

        assert_eq!(test, test0);
    }
}
