pub mod array;
pub mod dynamic_object;

use std::any::Any;
use object::Object;
use function::Function;
use value::{Value, ValueContext};
use executor::ExecutorImpl;
use errors::{VMError, FieldNotFoundError};
use generic_arithmetic;

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
            "freeze_dynamic" => {
                let target_id = match executor.get_current_frame().must_get_argument(0) {
                    Value::Object(id) => id,
                    _ => panic!(VMError::from("Invalid target object"))
                };
                let target: &dynamic_object::DynamicObject = executor.get_object_pool().must_get_direct_typed(target_id);
                target.freeze();
                Value::Null
            },
            "optimize" => {
                let target_id = match executor.get_current_frame().must_get_argument(0) {
                    Value::Object(id) => id,
                    _ => panic!(VMError::from("Invalid target object"))
                };
                let target = executor.get_object_pool().must_get_typed::<Function>(target_id);
                target.dynamic_optimize(executor.get_object_pool_mut());
                Value::Null
            },
            "add" => {
                generic_arithmetic::exec_add(executor, executor.get_current_frame().must_get_argument(0), executor.get_current_frame().must_get_argument(1))
            },
            "sub" => {
                generic_arithmetic::exec_sub(executor, executor.get_current_frame().must_get_argument(0), executor.get_current_frame().must_get_argument(1))
            },
            "mul" => {
                generic_arithmetic::exec_mul(executor, executor.get_current_frame().must_get_argument(0), executor.get_current_frame().must_get_argument(1))
            },
            "div" => {
                generic_arithmetic::exec_div(executor, executor.get_current_frame().must_get_argument(0), executor.get_current_frame().must_get_argument(1))
            },
            "mod" => {
                generic_arithmetic::exec_mod(executor, executor.get_current_frame().must_get_argument(0), executor.get_current_frame().must_get_argument(1))
            },
            "pow" => {
                generic_arithmetic::exec_pow(executor, executor.get_current_frame().must_get_argument(0), executor.get_current_frame().must_get_argument(1))
            },
            _ => panic!(VMError::from(FieldNotFoundError::from_field_name(name)))
        }
    }
}
