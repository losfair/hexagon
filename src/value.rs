use executor::ExecutorImpl;

pub enum Value {
    Object(usize),
    Int64(i64),
    Float64(f64)
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
            Value::Int64(v) => v,
            Value::Float64(v) => v as i64
        }
    }

    pub fn to_f64(&self) -> f64 {
        match *self.value {
            Value::Object(id) => self.executor.get_object_pool().get_direct(id).to_f64(),
            Value::Int64(v) => v as f64,
            Value::Float64(v) => v
        }
    }
}
