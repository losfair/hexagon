use std::any::Any;
use object::Object;
use opcode::OpCode;

pub struct Function {
    opcodes: Vec<OpCode>
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
    pub fn from_opcodes(opcodes: Vec<OpCode>) -> Function {
        Function {
            opcodes: opcodes
        }
    }
}
