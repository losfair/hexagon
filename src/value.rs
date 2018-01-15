use executor::ExecutorImpl;

pub enum Value {
    Object(usize),
    Null,
    Bool(bool),
    Int(i64),
    Float(f64)
}

pub struct ValueContext<'a> {
    value: &'a Value,
    executor: &'a ExecutorImpl
}

impl<'a> ValueContext<'a> {
    pub fn new(v: &'a Value, executor: &'a ExecutorImpl) -> ValueContext<'a> {
        ValueContext {
            value: v,
            executor: executor
        }
    }

    pub fn to_i64(&self) -> i64 {
        match *self.value {
            Value::Object(id) => self.executor.get_object_pool().get_direct(id).to_i64(),
            Value::Null => 0,
            Value::Bool(v) => if v {
                1
            } else {
                0
            },
            Value::Int(v) => v,
            Value::Float(v) => v as i64
        }
    }

    pub fn to_f64(&self) -> f64 {
        match *self.value {
            Value::Object(id) => self.executor.get_object_pool().get_direct(id).to_f64(),
            Value::Null => 0.0,
            Value::Bool(v) => if v {
                1.0
            } else {
                0.0
            },
            Value::Int(v) => v as f64,
            Value::Float(v) => v
        }
    }

    pub fn to_bool(&self) -> bool {
        match *self.value {
            Value::Object(id) => self.executor.get_object_pool().get_direct(id).to_bool(),
            Value::Null => false,
            Value::Bool(v) => v,
            Value::Int(v) => if v != 0 {
                true
            } else {
                false
            },
            Value::Float(v) => if v != 0.0 {
                true
            } else {
                false
            }
        }
    }
}
