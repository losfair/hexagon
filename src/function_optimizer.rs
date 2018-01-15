use basic_block::BasicBlock;
use object_pool::ObjectPool;

pub struct FunctionOptimizer<'a> {
    _basic_blocks: &'a mut Vec<BasicBlock>,
    _rt_handles: &'a mut Vec<usize>,
    _pool: &'a mut ObjectPool
}

impl<'a> FunctionOptimizer<'a> {
    pub fn new(basic_blocks: &'a mut Vec<BasicBlock>, rt_handles: &'a mut Vec<usize>, pool: &'a mut ObjectPool) -> FunctionOptimizer<'a> {
        FunctionOptimizer {
            _basic_blocks: basic_blocks,
            _rt_handles: rt_handles,
            _pool: pool
        }
    }

    pub fn optimize(&mut self) {
    }
}
