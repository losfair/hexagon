use super::opcode::OpCode;

#[derive(Serialize, Deserialize, Clone, Debug)]
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
