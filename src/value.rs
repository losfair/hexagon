use std::cmp::Ordering;
use errors;
use object::Object;
use object_pool::ObjectPool;
use object_info::ObjectHandle;

#[derive(Copy, Clone, Debug)]
pub enum Value {
    Object(usize),
    Null,
    Bool(bool),
    Int(i64),
    Float(f64)
}

pub struct ValueContext<'a, 'b> {
    pub value: &'a Value,
    pub pool: &'b ObjectPool
}

impl<'a, 'b> ValueContext<'a, 'b> {
    pub fn new(v: &'a Value, pool: &'b ObjectPool) -> ValueContext<'a, 'b> {
        ValueContext {
            value: v,
            pool: pool
        }
    }

    pub fn as_object_id(&self) -> usize {
        if let Value::Object(obj) = *self.value {
            obj
        } else {
            panic!(errors::VMError::from(errors::RuntimeError::new(
                format!("Not an object: {:?}", self.value)
            )));
        }
    }

    pub fn as_object(&self) -> ObjectHandle<'b> {
        self.pool.get(self.as_object_id())
    }

    pub fn as_object_direct(&self) -> &'b Object {
        self.pool.get_direct(self.as_object_id())
    }

    pub fn to_i64(&self) -> i64 {
        match *self.value {
            Value::Object(id) => self.pool.get_direct(id).to_i64(),
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
            Value::Object(id) => self.pool.get_direct(id).to_f64(),
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
            Value::Object(id) => self.pool.get_direct(id).to_bool(),
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

    pub fn compare(&self, other: &ValueContext) -> Option<Ordering> {
        if let Value::Object(_) = *self.value {
            return self.as_object_direct().compare(other);
        }
        if let Value::Object(_) = *other.value {
            return other.as_object_direct().compare(self);
        }

        match (*self.value, *other.value) {
            (Value::Null, Value::Null) => Some(Ordering::Equal),
            (Value::Bool(a), Value::Bool(b)) => a.partial_cmp(&b),
            (Value::Int(a), Value::Int(b)) => a.partial_cmp(&b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(&b),
            _ => None
        }
    }
}
