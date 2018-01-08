use std::cell::RefCell;
use std::collections::HashSet;
use smallvec::SmallVec;
use errors;

pub struct CallStack {
    frames: Vec<Frame>
}

pub struct Frame {
    pub(crate) this: usize,
    pub(crate) arguments: RefCell<SmallVec<[usize; 4]>>,
    pub(crate) locals: RefCell<SmallVec<[usize; 16]>>,
    pub(crate) exec_stack: RefCell<SmallVec<[usize; 16]>>
}

impl CallStack {
    pub fn new() -> CallStack {
        CallStack {
            frames: Vec::new()
        }
    }

    pub fn push(&mut self, frame: Frame) {
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
            for v in frame.arguments.borrow().iter() {
                objs.insert(*v);
            }
            for v in frame.locals.borrow().iter() {
                objs.insert(*v);
            }
            for v in frame.exec_stack.borrow().iter() {
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
            arguments: RefCell::new(args.into()),
            locals: RefCell::new(SmallVec::new()),
            exec_stack: RefCell::new(SmallVec::new())
        }
    }

    pub fn push_exec(&self, id: usize) {
        self.exec_stack.borrow_mut().push(id);
    }

    pub fn pop_exec(&self) -> usize {
        match self.exec_stack.borrow_mut().pop() {
            Some(v) => v,
            None => panic!(errors::VMError::from(errors::RuntimeError::new("Execution stack corrupted")))
        }
    }

    pub fn dup_exec(&self) {
        let mut stack = self.exec_stack.borrow_mut();
        if stack.is_empty() {
            panic!(errors::VMError::from(errors::RuntimeError::new("Execution stack corrupted")));
        }

        let last = stack[stack.len() - 1];
        stack.push(last);
    }

    pub fn reset_locals(&self, n_slots: usize) {
        let mut locals = self.locals.borrow_mut();
        locals.clear();
        for _ in 0..n_slots {
            locals.push(0);
        }
    }

    pub fn get_local(&self, ind: usize) -> usize {
        let locals = self.locals.borrow();
        if ind >= locals.len() {
            panic!(errors::VMError::from(errors::RuntimeError::new("Index out of bound")));
        }

        (*locals)[ind]
    }

    pub fn set_local(&self, ind: usize, obj_id: usize) {
        let mut locals = self.locals.borrow_mut();
        if ind >= locals.len() {
            panic!(errors::VMError::from(errors::RuntimeError::new("Index out of bound")));
        }

        (*locals)[ind] = obj_id;
    }

    pub fn get_argument(&self, id: usize) -> Option<usize> {
        let args = self.arguments.borrow();
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
        self.arguments.borrow().len()
    }

    pub fn get_this(&self) -> usize {
        self.this
    }
}
