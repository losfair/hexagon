use std::cell::{Ref, RefMut, RefCell};
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::cmp::Ordering;
use object::Object;
use call_stack::{CallStack, Frame};
use opcode::{OpCode, RtOpCode};
use errors;
use basic_block::BasicBlock;
use object_pool::ObjectPool;
use smallvec::SmallVec;
use hybrid::executor::Executor as HybridExecutor;
use value::{Value, ValueContext};
use builtin::BuiltinObject;

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
    hybrid_executor: HybridExecutor,

    object_pool: ObjectPool
}

enum EvalControlMessage {
    Return(Value),
    Redirect(usize)
}

impl ExecutorImpl {
    pub fn new() -> ExecutorImpl {
        let mut ret = ExecutorImpl {
            stack: CallStack::new(),
            hybrid_executor: HybridExecutor::new(),
            object_pool: ObjectPool::new()
        };
        ret.create_static_object("__builtin", Box::new(BuiltinObject::new()));
        ret
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

    pub fn set_stack_limit(&mut self, limit: usize) {
        self.stack.set_limit(limit);
    }

    pub fn get_hybrid_executor(&self) -> &HybridExecutor {
        &self.hybrid_executor
    }

    pub fn get_hybrid_executor_mut(&mut self) -> &mut HybridExecutor {
        &mut self.hybrid_executor
    }

    pub fn invoke(&mut self, callable_val: Value, this: Value, field_name: Option<&str>, args: &[Value]) {
        let frame = Frame::with_arguments(this, args);

        // Push the callable object onto the execution stack
        // to prevent it from begin GC-ed.
        //
        // No extra care needs to be taken for arguments
        // bacause they are already on the new frame.

        let callable_obj_id = match callable_val {
            Value::Object(id) => id,
            _ => panic!(errors::VMError::from(
                format!("Not callable. Got: {:?}", callable_val)
            ))
        };

        self.get_current_frame().push_exec(callable_val);

        let callable_obj = self.object_pool.get(callable_obj_id);

        self.stack.push(frame);
        let ret = catch_unwind(AssertUnwindSafe(|| match field_name {
            Some(v) => callable_obj.call_field(v, self),
            None => callable_obj.call(self)
        }));
        self.stack.pop();

        self.get_current_frame().pop_exec();

        match ret {
            Ok(v) => {
                self.get_current_frame().push_exec(v);
            },
            Err(e) => resume_unwind(e)
        }
    }

    fn set_static_object<K: ToString>(&mut self, key: K, obj: Value) {
        self.get_object_pool_mut().set_static_object(key, obj);
    }

    pub fn create_static_object<K: ToString>(&mut self, key: K, obj: Box<Object>) {
        let obj_id = self.object_pool.allocate(obj);
        self.set_static_object(key, Value::Object(obj_id));
    }

    pub fn get_static_object<K: AsRef<str>>(&self, key: K) -> Option<&Value> {
        self.get_object_pool().get_static_object(key)
    }

    fn eval_basic_blocks_impl(&mut self, basic_blocks: &[BasicBlock], basic_block_id: usize) -> EvalControlMessage {
        if basic_block_id >= basic_blocks.len() {
            panic!(errors::VMError::from(errors::RuntimeError::new("Basic block id out of bound")));
        }

        if self.object_pool.get_alloc_count() >= 1000 {
            self.object_pool.reset_alloc_count();
            self.object_pool.collect(&self.stack);
        }

        for op in &basic_blocks[basic_block_id].opcodes {
            match *op {
                OpCode::LoadNull => {
                    self.get_current_frame().push_exec(Value::Null);
                },
                OpCode::LoadInt(value) => {
                    self.get_current_frame().push_exec(Value::Int(value));
                },
                OpCode::LoadFloat(value) => {
                    self.get_current_frame().push_exec(Value::Float(value));
                },
                OpCode::LoadBool(value) => {
                    self.get_current_frame().push_exec(Value::Bool(value));
                },
                OpCode::LoadString(ref value) => {
                    let obj = self.object_pool.allocate(Box::new(value.clone()));
                    self.get_current_frame().push_exec(Value::Object(obj));
                },
                OpCode::LoadThis => {
                    let frame = self.get_current_frame();
                    frame.push_exec(frame.get_this());
                },
                OpCode::Call(n_args) => {
                    let (target, this, args) = {
                        let frame = self.get_current_frame();

                        let target = frame.pop_exec();
                        let this = frame.pop_exec();

                        let args: SmallVec<[Value; 4]> = (0..n_args)
                            .map(|_| frame.pop_exec())
                            .collect();

                        (target, this, args)
                    };
                    self.invoke(target, this, None, args.as_slice());
                },
                OpCode::CallField(n_args) => {
                    let (target, this, field_name, args) = {
                        let frame = self.get_current_frame();

                        let target = frame.pop_exec();
                        let this = frame.pop_exec();
                        let field_name = frame.pop_exec();

                        let args: SmallVec<[Value; 4]> = (0..n_args)
                            .map(|_| frame.pop_exec())
                            .collect();

                        (target, this, field_name, args)
                    };
                    let field_name = ValueContext::new(&field_name, self.get_object_pool()).to_str().to_string();
                    self.invoke(target, this, Some(field_name.as_str()), args.as_slice());
                },
                OpCode::Pop => {
                    self.get_current_frame().pop_exec();
                },
                OpCode::Dup => {
                    self.get_current_frame().dup_exec();
                },
                OpCode::InitLocal(n_slots) => {
                    let frame = self.get_current_frame();
                    frame.reset_locals(n_slots);
                },
                OpCode::GetLocal(ind) => {
                    let frame = self.get_current_frame();
                    let ret = frame.get_local(ind);
                    frame.push_exec(ret);
                },
                OpCode::SetLocal(ind) => {
                    let frame = self.get_current_frame();
                    let value = frame.pop_exec();
                    frame.set_local(ind, value);
                },
                OpCode::GetLocalIndirect => {
                    let frame = self.get_current_frame();
                    let ind = ValueContext::new(
                        &frame.pop_exec(),
                        self.get_object_pool()
                    ).to_i64() as usize;
                    let ret = frame.get_local(ind);
                    frame.push_exec(ret);
                },
                OpCode::SetLocalIndirect => {
                    let frame = self.get_current_frame();
                    let ind = ValueContext::new(
                        &frame.pop_exec(),
                        self.get_object_pool()
                    ).to_i64() as usize;
                    let value = frame.pop_exec();
                    frame.set_local(ind, value);
                },
                OpCode::GetArgument(ind) => {
                    let frame = self.get_current_frame();
                    frame.push_exec(frame.must_get_argument(ind));
                },
                OpCode::GetNArguments => {
                    let frame = self.get_current_frame();
                    frame.push_exec(Value::Int(frame.get_n_arguments() as i64));
                },
                OpCode::GetStatic => {
                    let key_val = self.get_current_frame().pop_exec();
                    let key = ValueContext::new(
                        &key_val,
                        self.get_object_pool()
                    ).as_object_direct().to_str();
                    let maybe_target_obj = self.get_static_object(key).map(|v| *v);

                    if let Some(target_obj) = maybe_target_obj {
                        self.get_current_frame().push_exec(target_obj);
                    } else {
                        self.get_current_frame().push_exec(Value::Null);
                    }
                },
                OpCode::SetStatic => {
                    let key_val = self.get_current_frame().pop_exec();
                    let key = ValueContext::new(
                        &key_val,
                        self.get_object_pool()
                    ).as_object_direct().to_string();

                    let value = self.get_current_frame().pop_exec();

                    self.set_static_object(key, value);
                },
                OpCode::GetField => {
                    let target_obj_val = self.get_current_frame().pop_exec();
                    let target_obj = ValueContext::new(
                        &target_obj_val,
                        self.get_object_pool()
                    ).as_object_direct();

                    let key_val = self.get_current_frame().pop_exec();
                    let key = ValueContext::new(
                        &key_val,
                        self.get_object_pool()
                    ).as_object_direct().to_str();

                    if let Some(v) = target_obj.get_field(self.get_object_pool(), key) {
                        self.get_current_frame().push_exec(v);
                    } else {
                        self.get_current_frame().push_exec(Value::Null);
                    }
                },
                OpCode::SetField => {
                    let (target_obj_val, key_val, value) = {
                        let frame = self.get_current_frame();
                        (
                            frame.pop_exec(),
                            frame.pop_exec(),
                            frame.pop_exec()
                        )
                    };

                    let target_obj = ValueContext::new(
                        &target_obj_val,
                        self.get_object_pool()
                    ).as_object_direct();

                    let key = ValueContext::new(
                        &key_val,
                        self.get_object_pool()
                    ).as_object_direct().to_str();

                    target_obj.set_field(key, value);
                },
                OpCode::Branch(target_id) => {
                    return EvalControlMessage::Redirect(target_id);
                },
                OpCode::ConditionalBranch(if_true, if_false) => {
                    let condition_is_true = {
                        let frame = self.get_current_frame();
                        ValueContext::new(
                            &frame.pop_exec(),
                            self.get_object_pool()
                        ).to_bool()
                    };

                    return EvalControlMessage::Redirect(if condition_is_true {
                        if_true
                    } else {
                        if_false
                    });
                },
                OpCode::Return => {
                    let ret_val = self.get_current_frame().pop_exec();
                    return EvalControlMessage::Return(ret_val);
                },
                OpCode::Add => {
                    let frame = self.get_current_frame();
                    let (left, right) = (frame.pop_exec(), frame.pop_exec());
                    self.get_current_frame().push_exec(
                        ValueContext::new(&left, self.get_object_pool()).add(
                            &ValueContext::new(&right, self.get_object_pool())
                        )
                    );
                },
                OpCode::IntAdd => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            ValueContext::new(&left, self.get_object_pool()).to_i64(),
                            ValueContext::new(&right, self.get_object_pool()).to_i64(),
                        )
                    };

                    self.get_current_frame().push_exec(Value::Int(left + right));
                },
                OpCode::IntSub => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            ValueContext::new(&left, self.get_object_pool()).to_i64(),
                            ValueContext::new(&right, self.get_object_pool()).to_i64(),
                        )
                    };

                    self.get_current_frame().push_exec(Value::Int(left - right));
                },
                OpCode::IntMul => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            ValueContext::new(&left, self.get_object_pool()).to_i64(),
                            ValueContext::new(&right, self.get_object_pool()).to_i64(),
                        )
                    };

                    self.get_current_frame().push_exec(Value::Int(left * right));
                },
                OpCode::IntDiv => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            ValueContext::new(&left, self.get_object_pool()).to_i64(),
                            ValueContext::new(&right, self.get_object_pool()).to_i64(),
                        )
                    };

                    self.get_current_frame().push_exec(Value::Int(left / right));
                },
                OpCode::IntMod => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            ValueContext::new(&left, self.get_object_pool()).to_i64(),
                            ValueContext::new(&right, self.get_object_pool()).to_i64(),
                        )
                    };

                    self.get_current_frame().push_exec(Value::Int(left % right));
                },
                OpCode::IntPow => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            ValueContext::new(&left, self.get_object_pool()).to_i64(),
                            ValueContext::new(&right, self.get_object_pool()).to_i64(),
                        )
                    };

                    self.get_current_frame().push_exec(Value::Int(left.pow(right as u32)));
                },
                OpCode::FloatAdd => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            ValueContext::new(&left, self.get_object_pool()).to_f64(),
                            ValueContext::new(&right, self.get_object_pool()).to_f64(),
                        )
                    };

                    self.get_current_frame().push_exec(Value::Float(left + right));
                },
                OpCode::FloatSub => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            ValueContext::new(&left, self.get_object_pool()).to_f64(),
                            ValueContext::new(&right, self.get_object_pool()).to_f64(),
                        )
                    };

                    self.get_current_frame().push_exec(Value::Float(left - right));
                },
                OpCode::FloatMul => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            ValueContext::new(&left, self.get_object_pool()).to_f64(),
                            ValueContext::new(&right, self.get_object_pool()).to_f64(),
                        )
                    };

                    self.get_current_frame().push_exec(Value::Float(left * right));
                },
                OpCode::FloatDiv => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            ValueContext::new(&left, self.get_object_pool()).to_f64(),
                            ValueContext::new(&right, self.get_object_pool()).to_f64(),
                        )
                    };

                    self.get_current_frame().push_exec(Value::Float(left / right));
                },
                OpCode::FloatPowi => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            ValueContext::new(&left, self.get_object_pool()).to_f64(),
                            ValueContext::new(&right, self.get_object_pool()).to_i64(),
                        )
                    };

                    self.get_current_frame().push_exec(Value::Float(left.powi(right as i32)));
                },
                OpCode::FloatPowf => {
                    let (left, right) = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        (
                            ValueContext::new(&left, self.get_object_pool()).to_f64(),
                            ValueContext::new(&right, self.get_object_pool()).to_f64(),
                        )
                    };

                    self.get_current_frame().push_exec(Value::Float(left.powf(right)));
                },
                OpCode::StringAdd => {
                    let new_value = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());
                        let (left, right) = (
                            ValueContext::new(&left, self.get_object_pool()),
                            ValueContext::new(&right, self.get_object_pool())
                        );
                        format!("{}{}", left.to_str(), right.to_str())
                    };
                    let new_value = self.get_object_pool_mut().allocate(
                        Box::new(new_value)
                    );

                    self.get_current_frame().push_exec(Value::Object(
                        new_value
                    ));
                },
                OpCode::CastToFloat => {
                    let value = ValueContext::new(
                        &self.get_current_frame().pop_exec(),
                        self.get_object_pool()
                    ).to_f64();
                    self.get_current_frame().push_exec(Value::Float(value));
                },
                OpCode::CastToInt => {
                    let value = ValueContext::new(
                        &self.get_current_frame().pop_exec(),
                        self.get_object_pool()
                    ).to_i64();
                    self.get_current_frame().push_exec(Value::Int(value));
                },
                OpCode::CastToBool => {
                    let value = ValueContext::new(
                        &self.get_current_frame().pop_exec(),
                        self.get_object_pool()
                    ).to_bool();
                    self.get_current_frame().push_exec(Value::Bool(value));
                },
                OpCode::CastToString => {
                    let value = ValueContext::new(
                        &self.get_current_frame().pop_exec(),
                        self.get_object_pool()
                    ).to_str().to_string();
                    let value = self.get_object_pool_mut().allocate(
                        Box::new(value)
                    );
                    self.get_current_frame().push_exec(Value::Object(value));
                },
                OpCode::And => {
                    let frame = self.get_current_frame();
                    let (left, right) = (frame.pop_exec(), frame.pop_exec());

                    let left_ctx = ValueContext::new(
                        &left,
                        self.get_object_pool()
                    );
                    let right_ctx = ValueContext::new(
                        &right,
                        self.get_object_pool()
                    );

                    self.get_current_frame().push_exec(Value::Bool(left_ctx.to_bool() && right_ctx.to_bool()));
                },
                OpCode::Or => {
                    let frame = self.get_current_frame();
                    let (left, right) = (frame.pop_exec(), frame.pop_exec());

                    let left_ctx = ValueContext::new(
                        &left,
                        self.get_object_pool()
                    );
                    let right_ctx = ValueContext::new(
                        &right,
                        self.get_object_pool()
                    );

                    self.get_current_frame().push_exec(Value::Bool(left_ctx.to_bool() || right_ctx.to_bool()));
                },
                OpCode::Not => {
                    let value = ValueContext::new(
                        &self.get_current_frame().pop_exec(),
                        self.get_object_pool()
                    ).to_bool();
                    self.get_current_frame().push_exec(Value::Bool(!value));
                },
                OpCode::TestLt => {
                    let ord = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());

                        let left_ctx = ValueContext::new(
                            &left,
                            self.get_object_pool()
                        );
                        let right_ctx = ValueContext::new(
                            &right,
                            self.get_object_pool()
                        );
                        left_ctx.compare(&right_ctx)
                    };

                    self.get_current_frame().push_exec(Value::Bool(ord == Some(Ordering::Less)));
                },
                OpCode::TestLe => {
                    let ord = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());

                        let left_ctx = ValueContext::new(
                            &left,
                            self.get_object_pool()
                        );
                        let right_ctx = ValueContext::new(
                            &right,
                            self.get_object_pool()
                        );
                        left_ctx.compare(&right_ctx)
                    };

                    self.get_current_frame().push_exec(Value::Bool(
                        ord == Some(Ordering::Less) || ord == Some(Ordering::Equal)
                    ));
                },
                OpCode::TestEq => {
                    let ord = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());

                        let left_ctx = ValueContext::new(
                            &left,
                            self.get_object_pool()
                        );
                        let right_ctx = ValueContext::new(
                            &right,
                            self.get_object_pool()
                        );
                        left_ctx.compare(&right_ctx)
                    };

                    self.get_current_frame().push_exec(Value::Bool(ord == Some(Ordering::Equal)));
                },
                OpCode::TestNe => {
                    let ord = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());

                        let left_ctx = ValueContext::new(
                            &left,
                            self.get_object_pool()
                        );
                        let right_ctx = ValueContext::new(
                            &right,
                            self.get_object_pool()
                        );
                        left_ctx.compare(&right_ctx)
                    };

                    self.get_current_frame().push_exec(Value::Bool(ord != Some(Ordering::Equal)));
                },
                OpCode::TestGe => {
                    let ord = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());

                        let left_ctx = ValueContext::new(
                            &left,
                            self.get_object_pool()
                        );
                        let right_ctx = ValueContext::new(
                            &right,
                            self.get_object_pool()
                        );
                        left_ctx.compare(&right_ctx)
                    };

                    self.get_current_frame().push_exec(Value::Bool(
                        ord == Some(Ordering::Greater) || ord == Some(Ordering::Equal)
                    ));
                },
                OpCode::TestGt => {
                    let ord = {
                        let frame = self.get_current_frame();
                        let (left, right) = (frame.pop_exec(), frame.pop_exec());

                        let left_ctx = ValueContext::new(
                            &left,
                            self.get_object_pool()
                        );
                        let right_ctx = ValueContext::new(
                            &right,
                            self.get_object_pool()
                        );
                        left_ctx.compare(&right_ctx)
                    };

                    self.get_current_frame().push_exec(Value::Bool(ord == Some(Ordering::Greater)));
                },
                OpCode::Rotate2 => {
                    let frame = self.get_current_frame();
                    let a = frame.pop_exec();
                    let b = frame.pop_exec();

                    frame.push_exec(a);
                    frame.push_exec(b);
                },
                OpCode::Rotate3 => {
                    let frame = self.get_current_frame();
                    let a = frame.pop_exec();
                    let b = frame.pop_exec();
                    let c = frame.pop_exec();

                    frame.push_exec(b);
                    frame.push_exec(a);
                    frame.push_exec(c);
                },
                OpCode::Rt(ref op) => {
                    match *op {
                        RtOpCode::LoadObject(id) => {
                            self.get_current_frame().push_exec(Value::Object(id));
                        }
                    }
                }
            }
        }

        panic!(errors::VMError::from(errors::RuntimeError::new("Leaving a basic block without terminator")));
    }

    pub(crate) fn eval_basic_blocks(&mut self, basic_blocks: &[BasicBlock], basic_block_id: usize) -> Value {
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
        let callable_obj_id = *self.get_static_object(key).unwrap_or_else(|| {
            panic!(errors::VMError::from(errors::RuntimeError::new("Static object not found")));
        });

        let new_this = Value::Null;

        let frame = Frame::with_arguments(
            new_this,
            &[]
        );

        self.stack.push(frame);
        let ret = catch_unwind(AssertUnwindSafe(|| self.invoke(callable_obj_id, new_this, None, &[])));
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
