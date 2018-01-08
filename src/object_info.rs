use std::ops::Deref;
use std::rc::Rc;
use std::cell::RefCell;
use object::Object;

pub struct ObjectInfo {
    object: Box<Object>,
    native_ref_info: ObjectNativeRefInfo
}

pub struct ObjectHandle<'a> {
    object: &'a Object,
    _native_ref_info: ObjectNativeRefInfo
}

impl<'a> Deref for ObjectHandle<'a> {
    type Target = &'a Object;
    fn deref(&self) -> &&'a Object {
        &self.object
    }
}

pub struct ObjectNativeRefInfo {
    n_refs: Rc<RefCell<usize>>,

    // in case n_refs becomes zero
    gc_notified: bool
}

impl ObjectInfo {
    pub fn new(obj: Box<Object>) -> ObjectInfo {
        ObjectInfo {
            object: obj,
            native_ref_info: ObjectNativeRefInfo {
                n_refs: Rc::new(RefCell::new(0)),
                gc_notified: false
            }
        }
    }

    pub fn gc_notify(&mut self) {
        self.native_ref_info.gc_notified = true;
    }

    pub fn handle<'a>(&self) -> ObjectHandle<'a> {
        ObjectHandle {
            object: unsafe {
                ::std::mem::transmute::<&Object, &'static Object>(&*self.object)
            },
            _native_ref_info: self.native_ref_info.clone()
        }
    }
}

impl Drop for ObjectInfo {
    fn drop(&mut self) {
        if *self.native_ref_info.n_refs.borrow() != 0 {
            eprintln!("Attempting to drop object with alive references");
            ::std::process::abort();
        }
    }
}

impl Clone for ObjectNativeRefInfo {
    fn clone(&self) -> Self {
        (*self.n_refs.borrow_mut()) += 1;
        ObjectNativeRefInfo {
            n_refs: self.n_refs.clone(),
            gc_notified: false
        }
    }
}

impl Drop for ObjectNativeRefInfo {
    fn drop(&mut self) {
        let mut n_refs = self.n_refs.borrow_mut();

        if self.gc_notified {
            assert_eq!(*n_refs, 0);
        } else {
            assert!(*n_refs > 0);
            *n_refs -= 1;
        }
    }
}

pub struct TypedObjectHandle<'a, T: 'a> {
    _handle: ObjectHandle<'a>,
    value: &'a T
}

impl<'a, T> Deref for TypedObjectHandle<'a, T> where T: 'a {
    type Target = &'a T;
    fn deref(&self) -> &&'a T {
        &self.value
    }
}

impl<'a, T> TypedObjectHandle<'a, T> where T: 'static {
    pub fn downcast_from(other: ObjectHandle<'a>) -> Option<TypedObjectHandle<'a, T>> {
        let value = match other.object.as_any().downcast_ref::<T>() {
            Some(v) => v,
            None => return None
        };
        Some(TypedObjectHandle {
            _handle: other,
            value: value
        })
    }
}
