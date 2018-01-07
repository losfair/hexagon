#[derive(Clone, Debug)]
pub enum OpCode {
    LoadNull,
    LoadInt(i64),
    LoadFloat(f64),
    LoadString(String),
    LoadBool(bool),
    Pop,
    InitLocal,
    GetLocal,
    SetLocal,
    GetStatic,
    SetStatic,
    Call,
    Branch,
    ConditionalBranch,
    Return
}
