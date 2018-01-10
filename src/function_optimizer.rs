use basic_block::BasicBlock;
use object_pool::ObjectPool;
use opcode::{OpCode, RtOpCode};
use primitive;

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
        let mut basic_blocks = self.basic_blocks.clone();
        for bb in &mut basic_blocks {
            bb.optimize_const_loads(self);
        }
        *self.basic_blocks = basic_blocks;
    }
}

impl BasicBlock {
    fn optimize_const_loads<'a>(&mut self, optimizer: &'a mut FunctionOptimizer) {
        let mut new_opcodes: Vec<OpCode> = Vec::new();

        for op in &self.opcodes {
            let const_res = match *op {
                OpCode::LoadNull => {
                    Some(optimizer.pool.allocate(Box::new(primitive::Null::new())))
                },
                OpCode::LoadInt(value) => {
                    Some(optimizer.pool.allocate(Box::new(primitive::Int::new(value))))
                },
                OpCode::LoadFloat(value) => {
                    Some(optimizer.pool.allocate(Box::new(primitive::Float::new(value))))
                },
                OpCode::LoadBool(value) => {
                    Some(optimizer.pool.allocate(Box::new(primitive::Bool::new(value))))
                },
                _ => None
            };
            if let Some(res) = const_res {
                optimizer.rt_handles.push(res);
                new_opcodes.push(OpCode::Rt(RtOpCode::LoadObject(res)));
            } else {
                new_opcodes.push(op.clone());
            }
        }

        self.opcodes = new_opcodes;
    }
}
