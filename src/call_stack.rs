use std::any::Any;
use std::cell::RefCell;
use object::Object;
use errors;

pub struct CallStack {
    frames: Vec<usize>
}

pub struct Frame {
    pub(crate) arguments: RefCell<Vec<usize>>,
    pub(crate) locals: RefCell<Vec<usize>>,
    pub(crate) exec_stack: RefCell<Vec<usize>>
}

impl Object for Frame {
    fn get_children(&self) -> Vec<usize> {
        let mut children = self.arguments.borrow().clone();
        children.extend(self.locals.borrow().clone());
        children.extend(self.exec_stack.borrow().clone());
        children
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self as &mut Any
    }
}

impl CallStack {
    pub fn new() -> CallStack {
        CallStack {
            frames: Vec::new()
        }
    }

    pub fn push(&mut self, frame: usize) {
        self.frames.push(frame);
    }

    pub fn pop(&mut self) -> usize {
        self.frames.pop().unwrap()
    }

    pub fn top(&self) -> usize {
        self.frames[self.frames.len() - 1]
    }
}

impl Frame {
    pub fn with_arguments(args: Vec<usize>) -> Frame {
        Frame {
            arguments: RefCell::new(args),
            locals: RefCell::new(Vec::new()),
            exec_stack: RefCell::new(Vec::new())
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

    pub fn reset_locals(&self, n_slots: usize) {
        *self.locals.borrow_mut() = vec![0; n_slots];
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
}
