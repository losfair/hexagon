use executor::Executor;
use opcode::OpCode;

#[derive(Clone)]
pub struct Program {
    pub(crate) opcodes: Vec<OpCode>
}

impl Program {
    pub fn from_opcodes(opcodes: Vec<OpCode>) -> Program {
        Program {
            opcodes: opcodes
        }
    }
}
