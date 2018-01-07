use std::any::Any;
use object::Object;
use basic_block::BasicBlock;
use executor::ExecutorImpl;

pub enum Function {
    Virtual(VirtualFunction),
    Native(NativeFunction)
}

pub struct VirtualFunction {
    basic_blocks: Vec<BasicBlock>
}

pub type NativeFunction = Box<Fn(&mut ExecutorImpl) -> usize>;

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
        match *self {
            Function::Virtual(ref vf) => {
                executor.eval_basic_blocks(vf.basic_blocks.as_slice(), 0)
            },
            Function::Native(ref nf) => {
                nf(executor)
            }
        }
    }
}

impl Function {
    pub fn from_basic_blocks(blocks: Vec<BasicBlock>) -> Function {
        Function::Virtual(VirtualFunction {
            basic_blocks: blocks
        })
    }

    pub fn from_native(nf: NativeFunction) -> Function {
        Function::Native(nf)
    }
}
