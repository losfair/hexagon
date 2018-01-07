use std::any::Any;
use std::cell::{Ref, RefMut, RefCell};
use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use object::Object;
use call_stack::{CallStack, Frame};
use static_root::StaticRoot;
use program::Program;
use opcode::OpCode;
use errors;

pub struct Executor {
    inner: RefCell<ExecutorImpl>
}

impl Executor {
    pub fn handle<'a>(&'a self) -> Ref<'a, ExecutorImpl> {
        self.inner.borrow()
    }

    pub fn handle_mut<'a>(&'a mut self) -> RefMut<'a, ExecutorImpl> {
        self.inner.borrow_mut()
    }
}

pub struct ExecutorImpl {
    stack: CallStack,
    static_objects: HashMap<String, usize>,

    // TODO: Split these out into a new struct
    objects: Vec<Option<Box<Object>>>,
    object_idx_pool: Vec<usize>
}

impl ExecutorImpl {
    pub fn new() -> ExecutorImpl {
        ExecutorImpl {
            stack: CallStack::new(),
            static_objects: HashMap::new(),
            objects: vec![
                Some(Box::new(StaticRoot::new()))
            ],
            object_idx_pool: vec![]
        }
    }

    fn allocate_object(&mut self, inner: Box<Object>) -> usize {
        let pool = &mut self.object_idx_pool;
        let id = if let Some(id) = pool.pop() {
            id
        } else {
            let objects = &mut self.objects;
            objects.push(None);
            objects.len() - 1
        };
        self.objects[id] = Some(inner);
        id
    }

    unsafe fn deallocate_object(&mut self, id: usize) {
        let mut objects = &mut self.objects;
        let mut pool = &mut self.object_idx_pool;

        objects[id] = None;
        pool.push(id);
    }

    fn get_object(&self, id: usize) -> &Object {
        &**self.objects[id].as_ref().unwrap()
    }

    fn get_typed_object<T: 'static>(&self, id: usize) -> Option<&T> {
        self.objects[id].as_ref().unwrap().as_any().downcast_ref::<T>()
    }

    fn get_static_root(&self) -> &StaticRoot {
        self.get_typed_object(0).unwrap()
    }

    fn get_current_frame(&self) -> &Frame {
        self.get_typed_object::<Frame>(self.stack.top()).unwrap()
    }

    fn pop_object_from_exec_stack(&self) -> &Object {
        self.get_object(self.get_current_frame().pop_exec())
    }

    fn invoke(&mut self, callable_obj_id: usize, args: Vec<usize>) -> usize {
        let frame = Frame::with_arguments(args);
        let frame_obj = self.allocate_object(Box::new(frame));

        // Push the callable object onto the execution stack
        // to prevent it from begin GC-ed.
        //
        // No extra care needs to be taken for arguments
        // bacause they are already on the new frame.

        self.get_current_frame().push_exec(callable_obj_id);

        // [TEST REQUIRED]
        // the object stays alive as long as it is reachable.
        // so this should be safe.
        let callable_obj = unsafe {
            ::std::mem::transmute::<&Object, &'static Object>(
                self.get_object(callable_obj_id)
            )
        };

        self.stack.push(frame_obj);
        let ret = catch_unwind(AssertUnwindSafe(|| callable_obj.call(self)));
        self.stack.pop();

        self.get_current_frame().pop_exec();

        match ret {
            Ok(v) => v,
            Err(e) => panic!(e)
        }
    }

    pub fn create_static_object<K: ToString>(&mut self, key: K, obj: Box<Object>) {
        let obj_id = self.allocate_object(obj);
        self.get_static_root().append_child(obj_id);
    }

    pub fn eval(&mut self, program: Program) {
        for op in program.opcodes {
            match op {
                OpCode::Call => {
                    let (target, args) = {
                        let frame = self.get_current_frame();

                        let target = frame.pop_exec();
                        let n_args_obj = frame.pop_exec();

                        let n_args = self.get_object(n_args_obj).to_i64();
                        if n_args < 0 {
                            panic!(errors::RuntimeError::new("Invalid number of arguments"));
                        }

                        let args: Vec<usize> = (0..(n_args as usize))
                            .map(|_| frame.pop_exec())
                            .collect();

                        (target, args)
                    };
                    let ret = self.invoke(target, args);
                    self.get_current_frame().push_exec(ret);
                },
                _ => panic!(errors::RuntimeError::new("Not implemented"))
            }
        }
    }
}
