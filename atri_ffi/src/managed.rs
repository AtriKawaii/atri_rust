use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ptr::null_mut;

#[repr(C)]
pub struct Managed {
    pub pointer: *mut (),
    pub drop: extern "C" fn(*mut ()),
}

unsafe impl Send for Managed {}
unsafe impl Sync for Managed {}

impl Managed {
    pub fn from_value<T>(value: T) -> Self {
        Self::from_box(Box::new(value))
    }

    pub fn from_box<T>(b: Box<T>) -> Self {
        extern "C" fn _drop<B>(pointer: *mut ()) {
            drop(unsafe { Box::from_raw(pointer.cast::<B>()) });
        }

        let ptr = Box::into_raw(b);

        Self {
            pointer: ptr.cast(),
            drop: _drop::<T>,
        }
    }

    pub fn from_static<T>(static_ref: &'static T) -> Self {
        extern "C" fn _drop(_: *mut ()) {
            // nothing to do
        }

        Self {
            pointer: (static_ref as *const T).cast_mut().cast(),
            drop: _drop,
        }
    }

    pub fn as_mut_ptr(&self) -> *mut () {
        self.pointer
    }

    pub fn as_ptr(&self) -> *const () {
        self.pointer
    }

    /// Consume this managed value, turning it into the type
    ///
    /// Safety: This is unsafe because we don't know the type
    /// behind the raw pointer
    pub unsafe fn into_value<T>(self) -> T {
        let ma = ManuallyDrop::new(self);
        *Box::from_raw(ma.pointer as *mut T)
    }

    /// Construct a managed null value
    ///
    /// Safety: This is unsafe because caller's behavior is unknown
    pub unsafe fn null() -> Self {
        extern "C" fn _drop_null(_: *mut ()) {}

        Self {
            pointer: null_mut(),
            drop: _drop_null,
        }
    }
}

impl Drop for Managed {
    fn drop(&mut self) {
        (self.drop)(self.pointer);
    }
}

#[repr(C)]
pub struct ManagedCloneable {
    pub value: Managed,
    clone: extern "C" fn(this: *const ()) -> ManagedCloneable,
}

impl From<ManagedCloneable> for Managed {
    fn from(ma: ManagedCloneable) -> Self {
        ma.value
    }
}

impl ManagedCloneable {
    pub fn from_value<T: Clone>(value: T) -> Self {
        extern "C" fn _clone<T: Clone>(this: *const ()) -> ManagedCloneable {
            let this = unsafe { &*(this as *const T) };
            ManagedCloneable::from_value(this.clone())
        }

        let value = Managed::from_value(value);
        Self {
            value,
            clone: _clone::<T>,
        }
    }

    /// Consume this managed value, turning it into the type
    ///
    /// Safety: This is unsafe because we don't know the type
    /// behind the raw pointer
    pub unsafe fn into_value<T>(self) -> T {
        self.value.into_value()
    }

    /// Safety: use this as option
    pub unsafe fn null() -> Self {
        extern "C" fn _clone_null(_: *const ()) -> ManagedCloneable {
            unsafe { ManagedCloneable::null() }
        }

        Self {
            value: Managed::null(),
            clone: _clone_null,
        }
    }
}

impl Clone for ManagedCloneable {
    fn clone(&self) -> Self {
        (self.clone)(self.value.pointer)
    }
}

impl Deref for ManagedCloneable {
    type Target = Managed;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
