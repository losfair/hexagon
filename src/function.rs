use std::any::Any;
use object::Object;
use opcode::OpCode;
use basic_block::BasicBlock;

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
}

impl Function {
    pub fn from_basic_blocks(blocks: Vec<BasicBlock>) -> Function {
        Function {
            basic_blocks: blocks
        }
    }
}
