use std::cell::UnsafeCell;
use std::collections::HashSet;
use smallvec::SmallVec;
use errors;

pub struct CallStack {
    frames: Vec<Frame>,
    limit: Option<usize>
}


// [unsafe]
// These fields are guaranteed to be accessed properly (as an implementation detail).
pub struct Frame {
    this: usize,
    arguments: UnsafeCell<SmallVec<[usize; 4]>>,
    locals: UnsafeCell<SmallVec<[usize; 16]>>,
    exec_stack: UnsafeCell<SmallVec<[usize; 16]>>
}

impl CallStack {
    pub fn new() -> CallStack {
        CallStack {
            frames: Vec::new(),
            limit: None
        }
    }

    pub fn set_limit(&mut self, limit: usize) {
        self.limit = Some(limit);
    }

    pub fn push(&mut self, frame: Frame) {
        if let Some(limit) = self.limit {
            if self.frames.len() >= limit {
                panic!(errors::VMError::from(errors::RuntimeError::new("Virtual stack overflow")));
            }
        }
        self.frames.push(frame);
    }

    pub fn pop(&mut self) -> Frame {
        self.frames.pop().unwrap()
    }

    pub fn top(&self) -> &Frame {
        &self.frames[self.frames.len() - 1]
    }

    pub fn collect_objects(&self) -> Vec<usize> {
        let mut objs = HashSet::new();
        for frame in &self.frames {
            objs.insert(frame.this);
            for v in unsafe { &*frame.arguments.get() }.iter() {
                objs.insert(*v);
            }
            for v in unsafe { &*frame.locals.get() }.iter() {
                objs.insert(*v);
            }
            for v in unsafe { &*frame.exec_stack.get() }.iter() {
                objs.insert(*v);
            }
        }
        objs.into_iter().collect()
    }
}

impl Frame {
    pub fn with_arguments(this: usize, args: &[usize]) -> Frame {
        Frame {
            this: this,
            arguments: UnsafeCell::new(args.into()),
            locals: UnsafeCell::new(SmallVec::new()),
            exec_stack: UnsafeCell::new(SmallVec::new())
        }
    }

    pub fn push_exec(&self, id: usize) {
        unsafe { &mut *self.exec_stack.get() }.push(id);
    }

    pub fn pop_exec(&self) -> usize {
        match unsafe { &mut *self.exec_stack.get() }.pop() {
            Some(v) => v,
            None => panic!(errors::VMError::from(errors::RuntimeError::new("Execution stack corrupted")))
        }
    }

    pub fn dup_exec(&self) {
        let stack = unsafe { &mut *self.exec_stack.get() };
        if stack.is_empty() {
            panic!(errors::VMError::from(errors::RuntimeError::new("Execution stack corrupted")));
        }

        let last = stack[stack.len() - 1];
        stack.push(last);
    }

    pub fn reset_locals(&self, n_slots: usize) {
        let locals = unsafe { &mut *self.locals.get() };
        locals.clear();
        for _ in 0..n_slots {
            locals.push(0);
        }
    }

    pub fn get_local(&self, ind: usize) -> usize {
        let locals = unsafe { &*self.locals.get() };
        if ind >= locals.len() {
            panic!(errors::VMError::from(errors::RuntimeError::new("Index out of bound")));
        }

        (*locals)[ind]
    }

    pub fn set_local(&self, ind: usize, obj_id: usize) {
        let locals = unsafe { &mut *self.locals.get() };
        if ind >= locals.len() {
            panic!(errors::VMError::from(errors::RuntimeError::new("Index out of bound")));
        }

        (*locals)[ind] = obj_id;
    }

    pub fn get_argument(&self, id: usize) -> Option<usize> {
        let args = unsafe { &*self.arguments.get() };
        if id < args.len() {
            Some(args[id])
        } else {
            None
        }
    }

    pub fn must_get_argument(&self, id: usize) -> usize {
        self.get_argument(id).unwrap_or_else(|| panic!("Argument index out of bound"))
    }

    pub fn get_n_arguments(&self) -> usize {
        unsafe { &*self.arguments.get() }.len()
    }

    pub fn get_this(&self) -> usize {
        self.this
    }
}
