#[macro_export]
macro_rules! fixed_array {
    ($name:ident, $len:expr) => {

pub struct $name<T: Copy> {
    data: [Cell<T>; $len],
    data_len: Cell<usize>
}

impl<T: Copy> $name<T> {
    pub fn new(default_value: T) -> $name<T> {
        let mut arr: [Cell<T>; $len] = unsafe { ::std::mem::uninitialized() };
        for i in 0..$len {
            unsafe { ::std::ptr::write(&mut arr[i], Cell::new(default_value)); }
        }

        $name {
            data: arr,
            data_len: Cell::new(0)
        }
    }

    pub fn push(&self, v: T) {
        let len = self.len();
        if len >= self.data.len() {
            panic!(errors::VMError::from("FixedArray overflow"));
        }
        self.data[len].set(v);
        self.data_len.set(len + 1);
    }

    pub fn pop(&self) -> T {
        let len = self.len();
        if len <= 0 {
            panic!(errors::VMError::from("FixedArray underflow"));
        }
        let v = self.data[len - 1].get();
        self.data_len.set(len - 1);
        v
    }

    pub fn top(&self) -> T {
        let len = self.len();
        if len <= 0 {
            panic!(errors::VMError::from("FixedArray underflow"));
        }
        self.data[len - 1].get()
    }

    pub fn len(&self) -> usize {
        self.data_len.get()
    }

    pub fn get(&self, id: usize) -> Option<T> {
        if id < self.len() {
            Some(self.data[id].get())
        } else {
            None
        }
    }

    pub fn set(&self, id: usize, v: T) {
        if id < self.len() {
            self.data[id].set(v);
        } else {
            panic!("Index out of bound");
        }
    }

    pub fn clear(&self) {
        self.data_len.set(0);
    }
}

    }
}
