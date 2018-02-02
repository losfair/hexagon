pub mod array;
pub mod dynamic_object;

use std::any::Any;
use object::Object;
use value::Value;
use executor::ExecutorImpl;
use errors::{VMError, FieldNotFoundError};

pub struct BuiltinObject {

}

impl BuiltinObject {
    pub fn new() -> BuiltinObject {
        BuiltinObject {}
    }
}

impl Object for BuiltinObject {
    fn get_children(&self) -> Vec<usize> {
        Vec::new()
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self as &mut Any
    }

    fn call_field(&self, name: &str, executor: &mut ExecutorImpl) -> Value {
        match name {
            "new_array" => {
                let array_obj: Box<Object> = Box::new(array::Array::new());
                Value::Object(
                    executor.get_object_pool_mut().allocate(array_obj)
                )
            },
            "new_dynamic" => {
                let prototype = match executor.get_current_frame().must_get_argument(0) {
                    Value::Object(id) => Some(id),
                    Value::Null => None,
                    _ => panic!(VMError::from("Invalid prototype object"))
                };
                Value::Object(executor.get_object_pool_mut().allocate(
                    Box::new(dynamic_object::DynamicObject::new(prototype))
                ))
            },
            _ => panic!(VMError::from(FieldNotFoundError::from_field_name(name)))
        }
    }
}
