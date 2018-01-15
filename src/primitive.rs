use std::any::Any;
use std::cmp::Ordering;
use object::Object;
use value::ValueContext;

pub struct StringObject {
    value: String
}

impl Object for StringObject {
    fn get_children(&self) -> Vec<usize> {
        Vec::new()
    }

    fn typename(&self) -> &str {
        "string"
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self as &mut Any
    }

    fn to_str(&self) -> &str {
        self.value.as_str()
    }

    fn to_bool(&self) -> bool {
        self.value == ""
    }

    fn test_eq(&self, other: &ValueContext) -> bool {
        if let Some(other) = other.as_object_direct().as_any().downcast_ref::<Self>() {
            other.value == self.value
        } else {
            false
        }
    }

    fn compare(&self, other: &ValueContext) -> Option<Ordering> {
        if let Some(other) = other.as_object_direct().as_any().downcast_ref::<Self>() {
            self.value.partial_cmp(&other.value)
        } else {
            None
        }
    }
}

impl StringObject {
    pub fn new<T: ToString>(value: T) -> StringObject {
        StringObject {
            value: value.to_string()
        }
    }
}
