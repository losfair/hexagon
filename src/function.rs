use std::any::Any;
use object::Object;
use opcode::OpCode;
use basic_block::BasicBlock;
use executor::ExecutorImpl;

pub struct Function {
    basic_blocks: Vec<BasicBlock>
}

impl Object for Function {
    fn get_children(&self) -> Vec<usize> {
        Vec::new()
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self as &mut Any
    }

    fn call(&self, executor: &mut ExecutorImpl) -> usize {
        executor.eval_basic_blocks(self.basic_blocks.as_slice(), 0)
    }
}

impl Function {
    pub fn from_basic_blocks(blocks: Vec<BasicBlock>) -> Function {
        Function {
            basic_blocks: blocks
        }
    }
}
