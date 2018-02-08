use std::cell::Cell;
use std::cell::UnsafeCell;
use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use smallvec::SmallVec;
use errors;
use value::Value;
use opcode::{StackMapPattern, ValueLocation};
use object_pool::ObjectPool;

thread_local! {
    static FRAME_POOL: RefCell<FramePool> = RefCell::new(FramePool::new(128));
}

struct FramePool {
    frames: Vec<Option<Box<Frame>>>,
    available: Vec<usize>,
    in_use: Vec<usize>
}

impl FramePool {
    fn new(n: usize) -> FramePool {
        FramePool {
            frames: (0..n).map(|_| Some(Box::new(Frame::new()))).collect(),
            available: (0..n).collect(),
            in_use: Vec::new()
        }
    }

    fn add(&mut self, mut frame: Box<Frame>) {
        frame.reset();

        if self.in_use.is_empty() {
            self.frames.push(Some(frame));
            self.available.push(self.frames.len() - 1);
        } else {
            let id = self.in_use.pop().unwrap();
            self.frames[id] = Some(frame);
            self.available.push(id);
        }
    }

    fn get(&mut self) -> Box<Frame> {
        if self.available.is_empty() {
            Box::new(Frame::new())
        } else {
            let id = self.available.pop().unwrap();
            let frame = ::std::mem::replace(&mut self.frames[id], None).unwrap();
            self.in_use.push(id);
            frame
        }
    }
}

pub struct CallStack {
    frames: Vec<FrameHandle>,
    limit: Option<usize>
}

pub struct FrameHandle {
    frame: Option<Box<Frame>>
}

impl Deref for FrameHandle {
    type Target = Frame;
    fn deref(&self) -> &Frame {
        self.frame.as_ref().unwrap()
    }
}

impl DerefMut for FrameHandle {
    fn deref_mut(&mut self) -> &mut Frame {
        self.frame.as_mut().unwrap()
    }
}

impl Drop for FrameHandle {
    fn drop(&mut self) {
        let frame = ::std::mem::replace(&mut self.frame, None).unwrap();
        FRAME_POOL.with(|pool| pool.borrow_mut().add(frame));
    }
}

impl FrameHandle {
    pub fn new() -> FrameHandle {
        FrameHandle {
            frame: Some(FRAME_POOL.with(|pool| pool.borrow_mut().get()))
        }
    }
}


// [unsafe]
// These fields are guaranteed to be accessed properly (as an implementation detail).
pub struct Frame {
    this: Cell<Value>,
    arguments: UnsafeCell<SmallVec<[Value; 4]>>,
    locals: UnsafeCell<SmallVec<[Value; 8]>>,
    pub(crate) exec_stack: UnsafeCell<SmallVec<[Value; 8]>>
}

impl CallStack {
    pub fn new() -> CallStack {
        CallStack {
            frames: vec! [ FrameHandle::new() ],
            limit: None
        }
    }

    pub fn set_limit(&mut self, limit: usize) {
        self.limit = Some(limit);
    }

    pub fn push(&mut self, frame: FrameHandle) {
        if let Some(limit) = self.limit {
            if self.frames.len() >= limit {
                panic!(errors::VMError::from(errors::RuntimeError::new("Virtual stack overflow")));
            }
        }
        self.frames.push(frame);
    }

    pub fn pop(&mut self) -> FrameHandle {
        self.frames.pop().unwrap()
    }

    pub fn top(&self) -> &FrameHandle {
        &self.frames[self.frames.len() - 1]
    }

    pub fn collect_objects(&self) -> Vec<usize> {
        let mut objs = HashSet::new();
        for frame in &self.frames {
            if let Value::Object(id) = frame.this.get() {
                objs.insert(id);
            }
            for v in unsafe { &*frame.arguments.get() }.iter() {
                if let Value::Object(id) = *v {
                    objs.insert(id);
                }
            }
            for v in unsafe { &*frame.locals.get() }.iter() {
                if let Value::Object(id) = *v {
                    objs.insert(id);
                }
            }
            for v in unsafe { &*frame.exec_stack.get() }.iter() {
                if let Value::Object(id) = *v {
                    objs.insert(id);
                }
            }
        }
        objs.into_iter().collect()
    }
}

impl Frame {
    fn new() -> Frame {
        Frame {
            this: Cell::new(Value::Null),
            arguments: UnsafeCell::new(SmallVec::new()),
            locals: UnsafeCell::new(SmallVec::new()),
            exec_stack: UnsafeCell::new(SmallVec::new())
        }
    }
    fn reset(&mut self) {
        self.this.set(Value::Null);
        unsafe {
            (&mut *self.arguments.get()).clear();
            (&mut *self.locals.get()).clear();
            (&mut *self.exec_stack.get()).clear();
        }
    }

    pub fn init_with_arguments(&mut self, this: Value, args: &[Value]) {
        self.this.set(this);
        let arguments = unsafe { &mut *self.arguments.get() };
        for arg in args {
            arguments.push(*arg);
        }
    }

    #[inline]
    pub fn push_exec(&self, obj: Value) {
        unsafe { &mut *self.exec_stack.get() }.push(obj);
    }

    #[inline]
    pub fn pop_exec(&self) -> Value {
        match unsafe { &mut *self.exec_stack.get() }.pop() {
            Some(v) => v,
            None => panic!(errors::VMError::from(errors::RuntimeError::new("Execution stack corrupted")))
        }
    }

    #[inline]
    pub fn dup_exec(&self) {
        let stack = unsafe { &mut *self.exec_stack.get() };
        if stack.is_empty() {
            panic!(errors::VMError::from(errors::RuntimeError::new("Execution stack corrupted")));
        }

        let last = stack[stack.len() - 1].clone();
        stack.push(last);
    }

    pub fn map_exec(&self, p: &StackMapPattern, pool: &mut ObjectPool) {
        let stack = unsafe { &mut *self.exec_stack.get() };

        let mut new_values: SmallVec<[Value; 4]> = SmallVec::with_capacity(p.map.len());
        for loc in &p.map {
            new_values.push(loc.extract(self, pool));
        }

        if p.end_state < 0 {
            for _ in 0..(-p.end_state) {
                stack.pop().unwrap();
            }
        } else {
            for _ in 0..p.end_state {
                stack.push(Value::Null);
            }
        }

        for i in 0..new_values.len() {
            stack[stack.len() - 1 - i] = new_values[new_values.len() - 1 - i];
        }
    }

    pub fn bulk_load(&self, values: &[Value]) {
        let stack = unsafe { &mut *self.exec_stack.get() };
        stack.reserve(values.len());
        for v in values {
            stack.push(*v);
        }
    }

    pub fn reset_locals(&self, n_slots: usize) {
        let locals = unsafe { &mut *self.locals.get() };
        locals.clear();
        for _ in 0..n_slots {
            locals.push(Value::Null);
        }
    }

    #[inline]
    pub fn get_local(&self, ind: usize) -> Value {
        let locals = unsafe { &*self.locals.get() };
        if ind >= locals.len() {
            panic!(errors::VMError::from(errors::RuntimeError::new("Index out of bound")));
        }

        (*locals)[ind]
    }

    #[inline]
    pub fn set_local(&self, ind: usize, obj: Value) {
        let locals = unsafe { &mut *self.locals.get() };
        if ind >= locals.len() {
            panic!(errors::VMError::from(errors::RuntimeError::new("Index out of bound")));
        }

        (*locals)[ind] = obj;
    }

    #[inline]
    pub fn get_argument(&self, id: usize) -> Option<Value> {
        let args = unsafe { &*self.arguments.get() };
        if id < args.len() {
            Some(args[id])
        } else {
            None
        }
    }

    #[inline]
    pub fn must_get_argument(&self, id: usize) -> Value {
        self.get_argument(id).unwrap_or_else(|| {
            panic!(errors::VMError::from(errors::RuntimeError::new("Argument index out of bound")))
        })
    }

    #[inline]
    pub fn get_n_arguments(&self) -> usize {
        unsafe { &*self.arguments.get() }.len()
    }

    #[inline]
    pub fn get_this(&self) -> Value {
        self.this.get()
    }

    #[inline]
    pub fn set_this(&self, this: Value) {
        self.this.set(this);
    }
}
