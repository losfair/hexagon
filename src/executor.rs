use std::any::Any;
use std::cell::{Ref, RefMut, RefCell};
use std::collections::HashMap;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use object::Object;
use call_stack::{CallStack, Frame};
use static_root::StaticRoot;
use opcode::OpCode;
use errors;
use primitive;
use basic_block::BasicBlock;

pub struct Executor {
    inner: RefCell<ExecutorImpl>
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            inner: RefCell::new(ExecutorImpl::new())
        }
    }

    pub fn handle<'a>(&'a self) -> Ref<'a, ExecutorImpl> {
        self.inner.borrow()
    }

    pub fn handle_mut<'a>(&'a self) -> RefMut<'a, ExecutorImpl> {
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

enum EvalControlMessage {
    Return(usize),
    Redirect(usize)
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

    fn invoke(&mut self, callable_obj_id: usize, args: Vec<usize>) {
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
            Ok(v) => {
                self.get_current_frame().push_exec(v);
            },
            Err(e) => resume_unwind(e)
        }
    }

    fn set_static_object<K: ToString>(&mut self, key: K, obj_id: usize) {
        let key = key.to_string();

        // Replacing static objects is denied to ensure
        // `get_static_object_ref` is safe.
        if self.static_objects.get(key.as_str()).is_some() {
            panic!(errors::VMError::from(errors::RuntimeError::new("A static object with the same key already exists")));
        }

        self.get_static_root().append_child(obj_id);
        self.static_objects.insert(key, obj_id);
    }

    pub fn create_static_object<K: ToString>(&mut self, key: K, obj: Box<Object>) {
        let obj_id = self.allocate_object(obj);
        self.set_static_object(key, obj_id);
    }

    pub fn get_static_object<K: AsRef<str>>(&self, key: K) -> Option<usize> {
        let key = key.as_ref();
        self.static_objects.get(key).map(|v| *v)
    }

    pub fn get_static_object_ref<K: AsRef<str>>(&self, key: K) -> Option<&Object> {
        self.get_static_object(key).map(|id| self.get_object(id))
    }

    fn eval_basic_blocks_impl(&mut self, basic_blocks: &[BasicBlock], basic_block_id: usize) -> EvalControlMessage {
        if basic_block_id >= basic_blocks.len() {
            panic!(errors::VMError::from(errors::RuntimeError::new("Basic block id out of bound")));
        }

        for op in &basic_blocks[basic_block_id].opcodes {
            match *op {
                OpCode::LoadNull => {
                    let obj = self.allocate_object(Box::new(primitive::Null::new()));
                    self.get_current_frame().push_exec(obj);
                },
                OpCode::LoadInt(value) => {
                    let obj = self.allocate_object(Box::new(primitive::Int::new(value)));
                    self.get_current_frame().push_exec(obj);
                },
                OpCode::LoadFloat(value) => {
                    let obj = self.allocate_object(Box::new(primitive::Float::new(value)));
                    self.get_current_frame().push_exec(obj);
                },
                OpCode::LoadBool(value) => {
                    let obj = self.allocate_object(Box::new(primitive::Bool::new(value)));
                    self.get_current_frame().push_exec(obj);
                },
                OpCode::LoadString(ref value) => {
                    let obj = self.allocate_object(Box::new(primitive::StringObject::new(value)));
                    self.get_current_frame().push_exec(obj);
                },
                OpCode::Call => {
                    let (target, args) = {
                        let frame = self.get_current_frame();

                        let target = frame.pop_exec();
                        let n_args_obj = frame.pop_exec();

                        let n_args = self.get_object(n_args_obj).to_i64();
                        if n_args < 0 {
                            panic!(errors::VMError::from(errors::RuntimeError::new("Invalid number of arguments")));
                        }

                        let args: Vec<usize> = (0..(n_args as usize))
                            .map(|_| frame.pop_exec())
                            .collect();

                        (target, args)
                    };
                    self.invoke(target, args);
                },
                OpCode::Pop => {
                    self.get_current_frame().pop_exec();
                },
                OpCode::InitLocal => {
                    let frame = self.get_current_frame();
                    let n_slots_obj = frame.pop_exec();
                    let n_slots = self.get_object(n_slots_obj).to_i64();
                    if n_slots < 0 {
                        panic!(errors::VMError::from(errors::RuntimeError::new("Invalid number of slots")));
                    }

                    frame.reset_locals(n_slots as usize);
                },
                OpCode::GetLocal => {
                    let frame = self.get_current_frame();
                    let ind_obj = frame.pop_exec();
                    let ind = self.get_object(ind_obj).to_i64();

                    if ind < 0 {
                        panic!(errors::VMError::from(errors::RuntimeError::new("Invalid index")));
                    }

                    let ret = frame.get_local(ind as usize);
                    frame.push_exec(ret);
                },
                OpCode::SetLocal => {
                    let frame = self.get_current_frame();
                    let ind_obj = frame.pop_exec();
                    let ind = self.get_object(ind_obj).to_i64();

                    let obj_id = frame.pop_exec();

                    if ind < 0 {
                        panic!(errors::VMError::from(errors::RuntimeError::new("Invalid index")));
                    }
                    frame.set_local(ind as usize, obj_id);
                },
                OpCode::GetStatic => {
                    let key_obj_id = self.get_current_frame().pop_exec();
                    let key = self.get_object(key_obj_id).to_string();

                    let maybe_target_obj = self.static_objects.get(key.as_str()).map(|v| *v);
                    if let Some(target_obj) = maybe_target_obj {
                        self.get_current_frame().push_exec(target_obj);
                    } else {
                        let null_obj_id = self.allocate_object(Box::new(primitive::Null::new()));
                        self.get_current_frame().push_exec(null_obj_id);
                    }
                },
                OpCode::SetStatic => {
                    let key_obj_id = self.get_current_frame().pop_exec();
                    let key = self.get_object(key_obj_id).to_string();

                    let obj_id = self.get_current_frame().pop_exec();

                    self.set_static_object(key, obj_id);
                },
                OpCode::Branch => {
                    let target_id_obj_id = self.get_current_frame().pop_exec();
                    let target_id = self.get_object(target_id_obj_id).to_i64();

                    if target_id < 0 {
                        panic!(errors::VMError::from(errors::RuntimeError::new("Invalid target id")));
                    }
                    return EvalControlMessage::Redirect(target_id as usize);
                },
                OpCode::ConditionalBranch => {
                    let (should_branch, target_id) = {
                        let frame = self.get_current_frame();
                        let condition_obj_id = frame.pop_exec();
                        let condition_obj = self.get_object(condition_obj_id);
                        let target_id_obj_id = frame.pop_exec();
                        let target_id = self.get_object(target_id_obj_id).to_i64();

                        if target_id < 0 {
                            panic!(errors::VMError::from(errors::RuntimeError::new("Invalid target id")));
                        }

                        (condition_obj.to_bool(), target_id)
                    };

                    if should_branch {
                        return EvalControlMessage::Redirect(target_id as usize);
                    }
                },
                OpCode::Return => {
                    let ret_val = self.get_current_frame().pop_exec();
                    return EvalControlMessage::Return(ret_val);
                }
            }
        }

        panic!(errors::VMError::from(errors::RuntimeError::new("Leaving a basic block without terminator")));
    }

    pub(crate) fn eval_basic_blocks(&mut self, basic_blocks: &[BasicBlock], basic_block_id: usize) -> usize {
        let mut current_id = basic_block_id;

        loop {
            let msg = self.eval_basic_blocks_impl(basic_blocks, current_id);
            match msg {
                EvalControlMessage::Redirect(target) => {
                    current_id = target;
                },
                EvalControlMessage::Return(value) => {
                    return value;
                }
            }
        }
    }

    pub fn run_callable<K: AsRef<str>>(&mut self, key: K) -> Result<(), errors::VMError> {
        let callable_obj_id = self.get_static_object(key).unwrap_or_else(|| {
            panic!(errors::VMError::from(errors::RuntimeError::new("Static object not found")));
        });

        let frame = Frame::with_arguments(vec! []);
        let frame_obj = self.allocate_object(Box::new(frame));

        self.stack.push(frame_obj);
        let ret = catch_unwind(AssertUnwindSafe(|| self.invoke(callable_obj_id, vec! [])));
        self.stack.pop();

        match ret {
            Ok(_) => Ok(()),
            Err(e) => {
                if let Ok(e) = e.downcast::<errors::VMError>() {
                    Err(*e)
                } else {
                    panic!("Unknown error from VM");
                }
            }
        }
    }
}
