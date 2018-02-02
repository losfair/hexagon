use std::any::Any;
use std::cmp::Ordering;
use object::Object;
use value::ValueContext;

impl Object for String {
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
        self.as_str()
    }

    fn to_bool(&self) -> bool {
        *self == ""
    }

    fn test_eq(&self, other: &ValueContext) -> bool {
        if let Some(other) = other.as_object_direct().as_any().downcast_ref::<Self>() {
            *other == *self
        } else {
            false
        }
    }

    fn compare(&self, other: &ValueContext) -> Option<Ordering> {
        if let Some(other) = other.as_object_direct().as_any().downcast_ref::<Self>() {
            self.partial_cmp(&other)
        } else {
            None
        }
    }
}
