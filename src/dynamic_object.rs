use std::any::Any;
use std::collections::HashMap;
use std::cell::RefCell;
use object::Object;
use object_pool::ObjectPool;
use value::Value;

pub struct DynamicObject {
    prototype: Option<usize>,
    fields: RefCell<HashMap<String, Value>>
}

impl Object for DynamicObject {
    fn get_children(&self) -> Vec<usize> {
        let mut children: Vec<usize> = self.fields.borrow().iter().map(|(_, v)| {
            if let Value::Object(id) = *v {
                Some(id)
            } else {
                None
            }
        }).filter(|v| if v.is_some() { true } else { false }).map(|v| v.unwrap()).collect();
        if let Some(prototype) = self.prototype {
            children.push(prototype);
        }
        children
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self as &mut Any
    }

    fn get_field(&self, pool: &ObjectPool, name: &str) -> Option<Value> {
        if let Some(v) = self.fields.borrow().get(name) {
            Some(*v)
        } else {
            if let Some(prototype) = self.prototype {
                let pt_object = pool.get_direct(prototype);
                pt_object.get_field(pool, name)
            } else {
                None
            }
        }
    }

    fn set_field(&self, name: &str, value: Value) {
        self.fields.borrow_mut().insert(name.to_string(), value);
    }
}

impl DynamicObject {
    pub fn new(prototype: Option<usize>) -> DynamicObject {
        DynamicObject {
            prototype: prototype,
            fields: RefCell::new(HashMap::new())
        }
    }
}
