use std::collections::VecDeque;
use opcode::{OpCode, RtOpCode, StackMapPattern};
use errors;
use value::Value;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BasicBlock {
    pub(crate) opcodes: Vec<OpCode>
}

impl BasicBlock {
    pub fn from_opcodes(opcodes: Vec<OpCode>) -> BasicBlock {
        BasicBlock {
            opcodes: opcodes
        }
    }

    pub fn branch_targets(&self) -> (Option<usize>, Option<usize>) {
        let last_opcode = &self.opcodes[self.opcodes.len() - 1];
        match *last_opcode {
            OpCode::ConditionalBranch(if_true, if_false) => (Some(if_true), Some(if_false)),
            OpCode::Branch(t) => (Some(t), None),
            OpCode::Return => (None, None),
            _ => panic!("Terminator not found")
        }
    }

    pub fn validate(&self, allow_runtime_opcodes: bool) -> Result<(), errors::ValidateError> {
        let mut itr = self.opcodes.iter();
        let mut terminator_found: bool = false;
        let mut stack_depth: isize = 0;

        for op in &mut itr {
            if !allow_runtime_opcodes {
                if let OpCode::Rt(_) = *op {
                    return Err(errors::ValidateError::new("Runtime opcodes are not allowed"));
                }
            }

            if let OpCode::RotateReverse(n) = *op {
                if n <= 0 {
                    return Err(errors::ValidateError::new("RotateReverse only accepts an operand greater than zero"));
                }
            }

            let (n_pops, n_pushes) = op.get_stack_depth_change();

            stack_depth -= n_pops as isize;

            if stack_depth < 0 {
                return Err(errors::ValidateError::new("Invalid use of stack"));
            }

            stack_depth += n_pushes as isize;

            let terminated = match *op {
                OpCode::ConditionalBranch(_, _)
                    | OpCode::Branch(_)
                    | OpCode::Return => true,
                _ => false
            };
            if terminated {
                terminator_found = true;
                break;
            }
        }

        if stack_depth != 0 {
            return Err(errors::ValidateError::new(format!("Stack not empty at the end of basic block (Depth: {})", stack_depth)));
        }

        if itr.next().is_some() {
            return Err(errors::ValidateError::new("Invalid terminator found in basic block"));
        }

        if !terminator_found {
            return Err(errors::ValidateError::new("Terminator not found"));
        }

        Ok(())
    }

    pub fn rebuild_stack_patterns(&mut self) {
        fn pack_deferred_ops(ops: Vec<BasicStackOp>) -> Option<OpCode> {
            if ops.len() == 0 {
                return None;
            }
            if ops.len() == 1 {
                return Some(match ops[0] {
                    BasicStackOp::Dup => OpCode::Dup,
                    BasicStackOp::Pop => OpCode::Pop,
                    BasicStackOp::Rotate2 => OpCode::Rotate2,
                    BasicStackOp::Rotate3 => OpCode::Rotate3,
                    BasicStackOp::RotateReverse(n) => OpCode::RotateReverse(n)
                })
            }

            let mut lower_bound: isize = 0;
            let mut upper_bound: isize = 0;
            let mut current: isize = 0;

            for op in &ops {
                match *op {
                    BasicStackOp::Dup => {
                        current += 1;
                    },
                    BasicStackOp::Pop => {
                        current -= 1;
                    },
                    BasicStackOp::Rotate2 => {
                        if current - 1 < lower_bound {
                            lower_bound = current - 1;
                        }
                    },
                    BasicStackOp::Rotate3 => {
                        if current - 2 < lower_bound {
                            lower_bound = current - 2;
                        }
                    },
                    BasicStackOp::RotateReverse(n) => {
                        let end_id = current - (n as isize - 1);
                        if end_id < lower_bound {
                            lower_bound = end_id;
                        }
                    }
                }
                if current > upper_bound {
                    upper_bound = current;
                }
                if current < lower_bound {
                    lower_bound = current;
                }
            }
            let end_state = current;

            let mut stack_map: Vec<isize> = Vec::new();
            for i in lower_bound..upper_bound + 1 {
                stack_map.push(i);
            }

            let mut current: usize = (0 - lower_bound) as usize;
            assert!(stack_map[current] == 0);
            let center = current;

            for op in &ops {
                match *op {
                    BasicStackOp::Dup => {
                        current += 1;
                        stack_map[current] = stack_map[current - 1];
                    },
                    BasicStackOp::Pop => {
                        current -= 1;
                    },
                    BasicStackOp::Rotate2 => {
                        stack_map.swap(current - 1, current);
                    },
                    BasicStackOp::Rotate3 => {
                        let a = stack_map[current];
                        let b = stack_map[current - 1];
                        let c = stack_map[current - 2];

                        stack_map[current - 2] = b;
                        stack_map[current - 1] = a;
                        stack_map[current] = c;
                    },
                    BasicStackOp::RotateReverse(n) => {
                        let mut seq: Vec<isize> = (0..n).map(|i| stack_map[current - i]).collect();
                        for i in 0..n {
                            stack_map[current - i] = seq.pop().unwrap();
                        }
                    }
                }
            }

            Some(OpCode::Rt(RtOpCode::StackMap(StackMapPattern {
                map: (0..(end_state - lower_bound + 1) as usize).map(|i| stack_map[i]).collect(),
                end_state: end_state
            })))
        }

        let mut new_ops: Vec<OpCode> = Vec::new();
        let mut deferred_stack_ops: Vec<BasicStackOp> = Vec::new();

        for op in &self.opcodes {
            let (n_pops, n_pushes) = op.get_stack_depth_change();

            match *op {
                OpCode::Dup => {
                    deferred_stack_ops.push(BasicStackOp::Dup);
                },
                OpCode::Pop => {
                    deferred_stack_ops.push(BasicStackOp::Pop);
                },
                OpCode::Rotate2 => {
                    deferred_stack_ops.push(BasicStackOp::Rotate2);
                },
                OpCode::Rotate3 => {
                    deferred_stack_ops.push(BasicStackOp::Rotate3);
                },
                OpCode::RotateReverse(n) => {
                    deferred_stack_ops.push(BasicStackOp::RotateReverse(n));
                },
                _ => {
                    let packed = pack_deferred_ops(::std::mem::replace(&mut deferred_stack_ops, Vec::new()));
                    if let Some(v) = packed {
                        new_ops.push(v);
                    }
                    new_ops.push(op.clone());
                }
            }
        }

        let packed = pack_deferred_ops(::std::mem::replace(&mut deferred_stack_ops, Vec::new()));
        if let Some(v) = packed {
            new_ops.push(v);
        }

        self.opcodes = new_ops;
    }
}

struct ValueInfo {
    kind: ValueType,
    const_val: Option<Value>
}

enum ValueType {
    Any,
    Int,
    Float,
    Bool,
    Null
}

impl Default for ValueInfo {
    fn default() -> ValueInfo {
        ValueInfo {
            kind: ValueType::Any,
            const_val: None
        }
    }
}

#[derive(Copy, Clone)]
enum BasicStackOp {
    Dup,
    Pop,
    Rotate2,
    Rotate3,
    RotateReverse(usize)
}
