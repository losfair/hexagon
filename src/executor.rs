use std::cell::{Ref, RefMut, RefCell};
use std::collections::HashMap;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::cmp::Ordering;
use object::Object;
use call_stack::{CallStack, Frame};
use opcode::OpCode;
use errors;
use primitive;
use basic_block::BasicBlock;
use object_info::{ObjectHandle};
use object_pool::ObjectPool;
use smallvec::SmallVec;

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

    object_pool: ObjectPool
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
            object_pool: ObjectPool::new()
        }
    }

    pub fn get_current_frame<'a>(&self) -> &Frame {
        self.stack.top()
    }

    pub fn get_object_pool(&self) -> &ObjectPool {
        &self.object_pool
    }

    pub fn get_object_pool_mut(&mut self) -> &mut ObjectPool {
        &mut self.object_pool
    }

    fn invoke(&mut self, callable_obj_id: usize, this: usize, args: &[usize]) {
        let frame = Frame::with_arguments(this, args);

        // Push the callable object onto the execution stack
        // to prevent it from begin GC-ed.
        //
        // No extra care needs to be taken for arguments
        // bacause they are already on the new frame.

        self.get_current_frame().push_exec(callable_obj_id);

        let callable_obj = self.object_pool.get(callable_obj_id);

        self.stack.push(frame);
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

        self.object_pool.get_static_root().append_child(obj_id);
        self.static_objects.insert(key, obj_id);
    }

    pub fn create_static_object<K: ToString>(&mut self, key: K, obj: Box<Object>) {
        let obj_id = self.object_pool.allocate(obj);
        self.set_static_object(key, obj_id);
    }

    pub fn get_static_object<K: AsRef<str>>(&self, key: K) -> Option<usize> {
        let key = key.as_ref();
        self.static_objects.get(key).map(|v| *v)
    }

    pub fn get_static_object_ref<'a, K: AsRef<str>>(&self, key: K) -> Option<ObjectHandle<'a>> {
        self.get_static_object(key).map(|id| self.object_pool.get(id))
    }

    fn eval_basic_blocks_impl(&mut self, basic_blocks: &[BasicBlock], basic_block_id: usize) -> EvalControlMessage {
        if basic_block_id >= basic_blocks.len() {
            panic!(errors::VMError::from(errors::RuntimeError::new("Basic block id out of bound")));
        }

        for op in &basic_blocks[basic_block_id].opcodes {
            match *op {
                OpCode::LoadNull => {
                    let obj = self.object_pool.allocate(Box::new(primitive::Null::new()));
                    self.get_current_frame().push_exec(obj);
                },
                OpCode::LoadInt(value) => {
                    let obj = self.object_pool.allocate(Box::new(primitive::Int::new(value)));
                    self.get_current_frame().push_exec(obj);
                },
                OpCode::LoadFloat(value) => {
                    let obj = self.object_pool.allocate(Box::new(primitive::Float::new(value)));
                    self.get_current_frame().push_exec(obj);
                },
                OpCode::LoadBool(value) => {
                    let obj = self.object_pool.allocate(Box::new(primitive::Bool::new(value)));
                    self.get_current_frame().push_exec(obj);
                },
                OpCode::LoadString(ref value) => {
                    let obj = self.object_pool.allocate(Box::new(primitive::StringObject::new(value)));
                    self.get_current_frame().push_exec(obj);
                },
                OpCode::LoadThis => {
                    let frame = self.get_current_frame();
                    frame.push_exec(frame.get_this());
                },
                OpCode::Call => {
                    let (target, this, args) = {
                        let frame = self.get_current_frame();

                        let target = frame.pop_exec();
                        let this = frame.pop_exec();
                        let n_args_obj = frame.pop_exec();

                        let n_args = self.object_pool.get(n_args_obj).to_i64();
                        if n_args < 0 {
                            panic!(errors::VMError::from(errors::RuntimeError::new("Invalid number of arguments")));
                        }

                        let args: SmallVec<[usize; 4]> = (0..(n_args as usize))
                            .map(|_| frame.pop_exec())
                            .collect();

                        (target, this, args)
                    };
                    self.invoke(target, this, args.as_slice());
                },
                OpCode::Pop => {
                    self.get_current_frame().pop_exec();
                },
                OpCode::InitLocal => {
                    let frame = self.get_current_frame();
                    let n_slots_obj = frame.pop_exec();
                    let n_slots = self.object_pool.get(n_slots_obj).to_i64();
                    if n_slots < 0 {
                        panic!(errors::VMError::from(errors::RuntimeError::new("Invalid number of slots")));
                    }

                    frame.reset_locals(n_slots as usize);
                },
                OpCode::GetLocal => {
                    let frame = self.get_current_frame();
                    let ind_obj = frame.pop_exec();
                    let ind = self.object_pool.get(ind_obj).to_i64();

                    if ind < 0 {
                        panic!(errors::VMError::from(errors::RuntimeError::new("Invalid index")));
                    }

                    let ret = frame.get_local(ind as usize);
                    frame.push_exec(ret);
                },
                OpCode::SetLocal => {
                    let frame = self.get_current_frame();
                    let ind_obj = frame.pop_exec();
                    let ind = self.object_pool.get(ind_obj).to_i64();

                    let obj_id = frame.pop_exec();

                    if ind < 0 {
                        panic!(errors::VMError::from(errors::RuntimeError::new("Invalid index")));
                    }
                    frame.set_local(ind as usize, obj_id);
                },
                OpCode::GetStatic => {
                    let key_obj_id = self.get_current_frame().pop_exec();
                    let key = self.object_pool.get(key_obj_id).to_str();

                    let maybe_target_obj = self.static_objects.get(key).map(|v| *v);
                    if let Some(target_obj) = maybe_target_obj {
                        self.get_current_frame().push_exec(target_obj);
                    } else {
                        let null_obj_id = self.object_pool.allocate(Box::new(primitive::Null::new()));
                        self.get_current_frame().push_exec(null_obj_id);
                    }
                },
                OpCode::SetStatic => {
                    let key_obj_id = self.get_current_frame().pop_exec();
                    let key = self.object_pool.get(key_obj_id).to_string();

                    let obj_id = self.get_current_frame().pop_exec();

                    self.set_static_object(key, obj_id);
                },
                OpCode::GetField => {
                    // TODO: Implement prototype-based subtyping

                    let target_obj_id = self.get_current_frame().pop_exec();
                    let key_obj_id = self.get_current_frame().pop_exec();

                    let target_obj = self.object_pool.get(target_obj_id);
                    let key = self.object_pool.get(key_obj_id).to_str();

                    if let Some(v) = target_obj.get_field(key) {
                        self.get_current_frame().push_exec(v);
                    } else {
                        let null_obj = self.object_pool.allocate(Box::new(primitive::Null::new()));
                        self.get_current_frame().push_exec(null_obj);
                    }
                },
                OpCode::SetField => {
                    let (target_obj_id, key_obj_id, value_obj_id) = {
                        let frame = self.get_current_frame();
                        (
                            frame.pop_exec(),
                            frame.pop_exec(),
                            frame.pop_exec()
                        )
                    };

                    let target_obj = self.object_pool.get(target_obj_id);
                    let key = self.object_pool.get(key_obj_id).to_str();

                    target_obj.set_field(key, value_obj_id);
                },
                OpCode::Branch => {
                    let target_id_obj_id = self.get_current_frame().pop_exec();
                    let target_id = self.object_pool.get(target_id_obj_id).to_i64();

                    if target_id < 0 {
                        panic!(errors::VMError::from(errors::RuntimeError::new("Invalid target id")));
                    }
                    return EvalControlMessage::Redirect(target_id as usize);
                },
                OpCode::ConditionalBranch => {
                    let (should_branch, target_id) = {
                        let frame = self.get_current_frame();
                        let condition_obj_id = frame.pop_exec();
                        let condition_obj = self.object_pool.get(condition_obj_id);
                        let target_id_obj_id = frame.pop_exec();
                        let target_id = self.object_pool.get(target_id_obj_id).to_i64();

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
                },
                OpCode::IntAdd => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            self.object_pool.must_get_typed::<primitive::Int>(left).value,
                            self.object_pool.must_get_typed::<primitive::Int>(right).value
                        )
                    };

                    let result = self.object_pool.allocate(Box::new(primitive::Int::new(left + right)));
                    self.get_current_frame().push_exec(result);
                },
                OpCode::IntSub => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            self.object_pool.must_get_typed::<primitive::Int>(left).value,
                            self.object_pool.must_get_typed::<primitive::Int>(right).value
                        )
                    };

                    let result = self.object_pool.allocate(Box::new(primitive::Int::new(left - right)));
                    self.get_current_frame().push_exec(result);
                },
                OpCode::IntMul => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            self.object_pool.must_get_typed::<primitive::Int>(left).value,
                            self.object_pool.must_get_typed::<primitive::Int>(right).value
                        )
                    };

                    let result = self.object_pool.allocate(Box::new(primitive::Int::new(left * right)));
                    self.get_current_frame().push_exec(result);
                },
                OpCode::IntDiv => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            self.object_pool.must_get_typed::<primitive::Int>(left).value,
                            self.object_pool.must_get_typed::<primitive::Int>(right).value
                        )
                    };

                    let result = self.object_pool.allocate(Box::new(primitive::Int::new(left / right)));
                    self.get_current_frame().push_exec(result);
                },
                OpCode::IntMod => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            self.object_pool.must_get_typed::<primitive::Int>(left).value,
                            self.object_pool.must_get_typed::<primitive::Int>(right).value
                        )
                    };

                    let result = self.object_pool.allocate(Box::new(primitive::Int::new(left % right)));
                    self.get_current_frame().push_exec(result);
                },
                OpCode::IntPow => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            self.object_pool.must_get_typed::<primitive::Int>(left).value,
                            self.object_pool.must_get_typed::<primitive::Int>(right).value
                        )
                    };

                    let result = self.object_pool.allocate(Box::new(primitive::Int::new(left.pow(right as u32))));
                    self.get_current_frame().push_exec(result);
                },
                OpCode::FloatAdd => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            self.object_pool.must_get_typed::<primitive::Float>(left).value,
                            self.object_pool.must_get_typed::<primitive::Float>(right).value
                        )
                    };

                    let result = self.object_pool.allocate(Box::new(primitive::Float::new(left + right)));
                    self.get_current_frame().push_exec(result);
                },
                OpCode::FloatSub => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            self.object_pool.must_get_typed::<primitive::Float>(left).value,
                            self.object_pool.must_get_typed::<primitive::Float>(right).value
                        )
                    };

                    let result = self.object_pool.allocate(Box::new(primitive::Float::new(left - right)));
                    self.get_current_frame().push_exec(result);
                },
                OpCode::FloatMul => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            self.object_pool.must_get_typed::<primitive::Float>(left).value,
                            self.object_pool.must_get_typed::<primitive::Float>(right).value
                        )
                    };

                    let result = self.object_pool.allocate(Box::new(primitive::Float::new(left * right)));
                    self.get_current_frame().push_exec(result);
                },
                OpCode::FloatDiv => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            self.object_pool.must_get_typed::<primitive::Float>(left).value,
                            self.object_pool.must_get_typed::<primitive::Float>(right).value
                        )
                    };

                    let result = self.object_pool.allocate(Box::new(primitive::Float::new(left / right)));
                    self.get_current_frame().push_exec(result);
                },
                OpCode::FloatPowi => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            self.object_pool.must_get_typed::<primitive::Float>(left).value,
                            self.object_pool.must_get_typed::<primitive::Int>(right).value
                        )
                    };

                    let result = self.object_pool.allocate(Box::new(primitive::Float::new(left.powi(right as i32))));
                    self.get_current_frame().push_exec(result);
                },
                OpCode::FloatPowf => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            self.object_pool.must_get_typed::<primitive::Float>(left).value,
                            self.object_pool.must_get_typed::<primitive::Float>(right).value
                        )
                    };

                    let result = self.object_pool.allocate(Box::new(primitive::Float::new(left.powf(right))));
                    self.get_current_frame().push_exec(result);
                },
                OpCode::CastToFloat => {
                    let value = self.get_current_frame().pop_exec();
                    let value = self.object_pool.get(value).to_f64();

                    let result = self.object_pool.allocate(Box::new(primitive::Float::new(value)));
                    self.get_current_frame().push_exec(result);
                },
                OpCode::CastToInt => {
                    let value = self.get_current_frame().pop_exec();
                    let value = self.object_pool.get(value).to_i64();

                    let result = self.object_pool.allocate(Box::new(primitive::Int::new(value)));
                    self.get_current_frame().push_exec(result);
                },
                OpCode::CastToBool => {
                    let value = self.get_current_frame().pop_exec();
                    let value = self.object_pool.get(value).to_bool();

                    let result = self.object_pool.allocate(Box::new(primitive::Bool::new(value)));
                    self.get_current_frame().push_exec(result);
                },
                OpCode::Not => {
                    let value = self.get_current_frame().pop_exec();
                    let value = self.object_pool.get(value).to_bool();

                    let result = self.object_pool.allocate(Box::new(primitive::Bool::new(!value)));
                    self.get_current_frame().push_exec(result);
                },
                OpCode::TestLt => {
                    let ord = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        self.object_pool.get(left).compare(*self.object_pool.get(right))
                    };

                    let result = self.object_pool.allocate(Box::new(primitive::Bool::new(
                        ord == Some(Ordering::Less)
                    )));
                    self.get_current_frame().push_exec(result);
                },
                OpCode::TestEq => {
                    let ok = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        self.object_pool.get(left).test_eq(*self.object_pool.get(right))
                    };

                    let result = self.object_pool.allocate(Box::new(primitive::Bool::new(
                        ok
                    )));
                    self.get_current_frame().push_exec(result);
                },
                OpCode::TestGt => {
                    let ord = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        self.object_pool.get(left).compare(*self.object_pool.get(right))
                    };

                    let result = self.object_pool.allocate(Box::new(primitive::Bool::new(
                        ord == Some(Ordering::Greater)
                    )));
                    self.get_current_frame().push_exec(result);
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

    pub fn gc(&mut self) {
        self.object_pool.collect(&self.stack);
    }

    pub fn run_callable<K: AsRef<str>>(&mut self, key: K) -> Result<(), errors::VMError> {
        let callable_obj_id = self.get_static_object(key).unwrap_or_else(|| {
            panic!(errors::VMError::from(errors::RuntimeError::new("Static object not found")));
        });

        let new_this = self.object_pool.allocate(Box::new(primitive::Null::new()));

        let frame = Frame::with_arguments(
            new_this,
            &[]
        );

        self.stack.push(frame);
        let ret = catch_unwind(AssertUnwindSafe(|| self.invoke(callable_obj_id, new_this, &[])));
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
