use std::any::Any;
use errors;
use executor::{ExecutorImpl};

pub trait Object {
    fn finalize(&self) {}
    fn is_callable(&self) -> bool {
        false
    }
    fn call(&self, executor: &mut ExecutorImpl) -> usize {
        panic!(errors::RuntimeError::new("Not callable"));
    }
    fn get_field(&self, name: &str) -> Option<usize> {
        None
    }
    fn must_get_field(&self, name: &str) -> usize {
        match self.get_field(name) {
            Some(v) => v,
            None => panic!(errors::FieldNotFoundError::from_field_name(name))
        }
    }
    fn typename(&self) -> &str {
        "object"
    }
    fn to_i64(&self) -> i64 {
        panic!(errors::RuntimeError::new("Cannot cast to i64"));
    }
    fn to_f64(&self) -> f64 {
        panic!(errors::RuntimeError::new("Cannot cast to f64"));
    }
    fn get_children(&self) -> Vec<usize>;
    fn as_any(&self) -> &Any;
    fn as_any_mut(&mut self) -> &mut Any;
}
