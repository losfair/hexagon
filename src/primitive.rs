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

    fn to_bool(&self) -> bool {
        false
    }
}

impl Null {
    pub fn new() -> Null {
        Null {}
    }
}

pub struct Bool {
    value: bool
}

impl Object for Bool {
    fn get_children(&self) -> Vec<usize> {
        Vec::new()
    }

    fn typename(&self) -> &str {
        "bool"
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self as &mut Any
    }

    fn to_i64(&self) -> i64 {
        if self.value {
            1
        } else {
            0
        }
    }

    fn to_f64(&self) -> f64 {
        if self.value {
            1.0
        } else {
            0.0
        }
    }

    fn to_bool(&self) -> bool {
        self.value
    }
}

impl Bool {
    pub fn new(value: bool) -> Bool {
        Bool {
            value: value
        }
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

    fn to_bool(&self) -> bool {
        self.value == 0
    }
}

impl Int {
    pub fn new(value: i64) -> Int {
        Int {
            value: value
        }
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

    fn to_bool(&self) -> bool {
        self.value == 0.0
    }
}

impl Float {
    pub fn new(value: f64) -> Float {
        Float {
            value: value
        }
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

    fn to_bool(&self) -> bool {
        self.value == ""
    }
}

impl StringObject {
    pub fn new<T: ToString>(value: T) -> StringObject {
        StringObject {
            value: value.to_string()
        }
    }
}
