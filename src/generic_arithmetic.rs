use executor::ExecutorImpl;
use value::{Value, ValueContext};
use errors::VMError;

pub fn exec_add(executor: &mut ExecutorImpl, left: Value, right: Value) -> Value {
    match left {
        Value::Object(_) => {
            executor.invoke(left, Value::Null, Some("__add__"), &[right]);
            executor.get_current_frame().pop_exec()
        },
        Value::Int(v) => {
            Value::Int(
                v + ValueContext::new(&right, executor.get_object_pool()).to_i64()
            )
        },
        Value::Float(v) => {
            Value::Float(
                v + ValueContext::new(&right, executor.get_object_pool()).to_f64()
            )
        },
        _ => panic!(VMError::from("Invalid operation"))
    }
}

pub fn exec_sub(executor: &mut ExecutorImpl, left: Value, right: Value) -> Value {
    match left {
        Value::Object(_) => {
            executor.invoke(left, Value::Null, Some("__sub__"), &[right]);
            executor.get_current_frame().pop_exec()
        },
        Value::Int(v) => {
            Value::Int(
                v - ValueContext::new(&right, executor.get_object_pool()).to_i64()
            )
        },
        Value::Float(v) => {
            Value::Float(
                v - ValueContext::new(&right, executor.get_object_pool()).to_f64()
            )
        },
        _ => panic!(VMError::from("Invalid operation"))
    }
}

pub fn exec_mul(executor: &mut ExecutorImpl, left: Value, right: Value) -> Value {
    match left {
        Value::Object(_) => {
            executor.invoke(left, Value::Null, Some("__mul__"), &[right]);
            executor.get_current_frame().pop_exec()
        },
        Value::Int(v) => {
            Value::Int(
                v * ValueContext::new(&right, executor.get_object_pool()).to_i64()
            )
        },
        Value::Float(v) => {
            Value::Float(
                v * ValueContext::new(&right, executor.get_object_pool()).to_f64()
            )
        },
        _ => panic!(VMError::from("Invalid operation"))
    }
}

pub fn exec_div(executor: &mut ExecutorImpl, left: Value, right: Value) -> Value {
    match left {
        Value::Object(_) => {
            executor.invoke(left, Value::Null, Some("__div__"), &[right]);
            executor.get_current_frame().pop_exec()
        },
        Value::Int(v) => {
            Value::Int(
                v / ValueContext::new(&right, executor.get_object_pool()).to_i64()
            )
        },
        Value::Float(v) => {
            Value::Float(
                v / ValueContext::new(&right, executor.get_object_pool()).to_f64()
            )
        },
        _ => panic!(VMError::from("Invalid operation"))
    }
}

pub fn exec_mod(executor: &mut ExecutorImpl, left: Value, right: Value) -> Value {
    match left {
        Value::Object(_) => {
            executor.invoke(left, Value::Null, Some("__mod__"), &[right]);
            executor.get_current_frame().pop_exec()
        },
        Value::Int(v) => {
            Value::Int(
                v % ValueContext::new(&right, executor.get_object_pool()).to_i64()
            )
        },
        Value::Float(v) => {
            Value::Float(
                v % ValueContext::new(&right, executor.get_object_pool()).to_f64()
            )
        },
        _ => panic!(VMError::from("Invalid operation"))
    }
}
