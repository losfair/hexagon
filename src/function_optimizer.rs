use basic_block::BasicBlock;
use opcode::{OpCode, RtOpCode};
use object::Object;
use object_pool::ObjectPool;
use value::Value;

pub struct FunctionOptimizer<'a> {
    basic_blocks: &'a mut Vec<BasicBlock>,
    rt_handles: &'a mut Vec<usize>,
    pool: &'a mut ObjectPool
}

impl<'a> FunctionOptimizer<'a> {
    pub fn new(basic_blocks: &'a mut Vec<BasicBlock>, rt_handles: &'a mut Vec<usize>, pool: &'a mut ObjectPool) -> FunctionOptimizer<'a> {
        FunctionOptimizer {
            basic_blocks: basic_blocks,
            rt_handles: rt_handles,
            pool: pool
        }
    }

    pub fn optimize(&mut self) {
        self.optimize_const_static_load();
        for bb in self.basic_blocks.iter_mut() {
            bb.rebuild_stack_patterns();
        }
    }

    pub fn optimize_const_static_load(&mut self) {
        for bb in self.basic_blocks.iter_mut() {
            let mut new_opcodes: Vec<OpCode> = Vec::new();
            let mut patterns: Vec<Option<String>> = vec![None; bb.opcodes.len()];

            for i in 0..bb.opcodes.len() - 1 {
                match (&bb.opcodes[i], &bb.opcodes[i + 1]) {
                    (&OpCode::LoadString(ref s), &OpCode::GetStatic) => {
                        patterns[i] = Some(s.clone());
                    },
                    _ => {}
                }
            }

            let mut remove_count: usize = 0;

            for i in 0..bb.opcodes.len() {
                if let Some(ref name) = patterns[i] {
                    if let Some(val) = self.pool.get_static_object(name) {
                        let op = match *val {
                            Value::Bool(v) => OpCode::LoadBool(v),
                            Value::Float(v) => OpCode::LoadFloat(v),
                            Value::Int(v) => OpCode::LoadInt(v),
                            Value::Null => OpCode::LoadNull,
                            Value::Object(id) => {
                                self.rt_handles.push(id);
                                OpCode::Rt(RtOpCode::LoadObject(id))
                            }
                        };
                        new_opcodes.push(op);
                        // the next opcode will be the original `GetStatic`, which is not needed any more.
                        remove_count = 1;
                        continue;
                    }
                }

                if remove_count > 0 {
                    remove_count -= 1;
                } else {
                    new_opcodes.push(bb.opcodes[i].clone());
                }
            }

            bb.opcodes = new_opcodes;
            match bb.validate(true) {
                Ok(_) => {},
                Err(e) => panic!(e.to_str().to_string())
            }
        }
    }
}
