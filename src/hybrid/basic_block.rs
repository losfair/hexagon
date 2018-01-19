use super::opcode::OpCode;

#[derive(Clone, Debug)]
pub struct BasicBlock {
    pub(super) opcodes: Vec<OpCode>
}

impl BasicBlock {
    pub fn from_opcodes(opcodes: Vec<OpCode>) -> BasicBlock {
        BasicBlock {
            opcodes: opcodes
        }
    }
}
