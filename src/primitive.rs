use std::any::Any;
use object::Object;

pub struct Null {
}

impl Object for Null {
    fn get_children(&self) -> Vec<usize> {
        Vec::new()
    }

    fn typename(&self) -> &str {
        "(null)"
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self as &mut Any
    }
}

impl Null {
    pub fn new() -> Null {
        Null {}
    }
}

pub struct Int {
    value: i64
}

impl Object for Int {
    fn get_children(&self) -> Vec<usize> {
        Vec::new()
    }

    fn typename(&self) -> &str {
        "int"
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self as &mut Any
    }

    fn to_i64(&self) -> i64 {
        self.value
    }

    fn to_f64(&self) -> f64 {
        self.value as f64
    }
}

pub struct Float {
    value: f64
}

impl Object for Float {
    fn get_children(&self) -> Vec<usize> {
        Vec::new()
    }

    fn typename(&self) -> &str {
        "float"
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self as &mut Any
    }

    fn to_f64(&self) -> f64 {
        self.value
    }

    fn to_i64(&self) -> i64 {
        self.value as i64
    }
}

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

    fn to_string(&self) -> String {
        self.value.clone()
    }
}
