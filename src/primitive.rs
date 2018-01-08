use std::any::Any;
use std::cmp::Ordering;
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

    fn test_eq(&self, other: &Object) -> bool {
        other.as_any().is::<Self>()
    }

    fn compare(&self, other: &Object) -> Option<Ordering> {
        if self.test_eq(other) {
            Some(Ordering::Equal)
        } else {
            None
        }
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

    fn test_eq(&self, other: &Object) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            other.value == self.value
        } else {
            false
        }
    }

    fn compare(&self, other: &Object) -> Option<Ordering> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.value.partial_cmp(&other.value)
        } else {
            None
        }
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
    pub(crate) value: i64
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

    fn test_eq(&self, other: &Object) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            other.value == self.value
        } else {
            false
        }
    }

    fn compare(&self, other: &Object) -> Option<Ordering> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.value.partial_cmp(&other.value)
        } else {
            None
        }
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
    pub(crate) value: f64
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

    fn test_eq(&self, other: &Object) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            other.value == self.value
        } else {
            false
        }
    }

    fn compare(&self, other: &Object) -> Option<Ordering> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.value.partial_cmp(&other.value)
        } else {
            None
        }
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

    fn to_str(&self) -> &str {
        self.value.as_str()
    }

    fn to_bool(&self) -> bool {
        self.value == ""
    }

    fn test_eq(&self, other: &Object) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            other.value == self.value
        } else {
            false
        }
    }

    fn compare(&self, other: &Object) -> Option<Ordering> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.value.partial_cmp(&other.value)
        } else {
            None
        }
    }
}

impl StringObject {
    pub fn new<T: ToString>(value: T) -> StringObject {
        StringObject {
            value: value.to_string()
        }
    }
}
