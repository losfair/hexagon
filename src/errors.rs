use std::any::Any;
use object::Object;

pub struct VMError {
    inner: Box<Object>
}

unsafe impl Send for VMError {}

impl<T> From<T> for VMError where T: Object + 'static {
    fn from(other: T) -> VMError {
        VMError {
            inner: Box::new(other)
        }
    }
}

impl VMError {
    pub fn unwrap(self) -> Box<Object> {
        self.inner
    }
}

pub struct RuntimeError {
    description: String
}

impl Object for RuntimeError {
    fn get_children(&self) -> Vec<usize> {
        Vec::new()
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self as &mut Any
    }

    fn to_string(&self) -> String {
        self.description.clone()
    }
}

impl RuntimeError {
    pub fn new<T: ToString>(desc: T) -> RuntimeError {
        RuntimeError {
            description: desc.to_string()
        }
    }
}

pub struct FieldNotFoundError {
    field_name: String
}

impl Object for FieldNotFoundError {
    fn get_children(&self) -> Vec<usize> {
        Vec::new()
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self as &mut Any
    }

    fn to_string(&self) -> String {
        format!("Field not found: {}", self.field_name)
    }
}

impl FieldNotFoundError {
    pub fn from_field_name<T: ToString>(name: T) -> FieldNotFoundError {
        FieldNotFoundError {
            field_name: name.to_string()
        }
    }
}
