#[derive(Copy, Clone, Debug)]
pub enum OpCode {
    Pop,
    InitLocal,
    GetLocal,
    SetLocal,
    GetStatic,
    Call,
    Branch,
    ConditionalBranch,
    Return
}
