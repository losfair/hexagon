#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum OpCode {
    LoadNull,
    LoadInt(i64),
    LoadFloat(f64),
    LoadString(String),
    LoadBool(bool),
    LoadThis,
    Pop,
    Dup,
    InitLocal(usize),
    GetLocal(usize),
    SetLocal(usize),
    GetStatic,
    SetStatic,
    GetField,
    SetField,
    Call(usize),
    Branch(usize),
    ConditionalBranch(usize, usize),
    Return,
    IntAdd,
    IntSub,
    IntMul,
    IntDiv,
    IntMod,
    IntPow,
    FloatAdd,
    FloatSub,
    FloatMul,
    FloatDiv,
    FloatPowi,
    FloatPowf,
    StringAdd,
    CastToFloat,
    CastToInt,
    CastToBool,
    CastToString,
    Not,
    TestLt,
    TestEq,
    TestGt,

    #[serde(skip_serializing, skip_deserializing)]
    Rt(RtOpCode)
}

#[derive(Clone, Debug, PartialEq)]
pub enum RtOpCode {
    LoadObject(usize)
}

impl OpCode {
    pub fn get_stack_depth_change(&self) -> (usize, usize) {
        use self::OpCode::*;

        // (pop, push)
        match *self {
            LoadNull | LoadInt(_) | LoadFloat(_) | LoadString(_) | LoadBool(_) | LoadThis => (0, 1), // pushes the value
            Pop => (1, 0), // pops the object on the top
            Dup => (0, 1), // duplicates the object on the top
            InitLocal(_) => (0, 0),
            GetLocal(_) => (0, 1), // pushes object
            SetLocal(_) => (1, 0), // pops object
            GetStatic => (1, 1), // pops name, pushes object
            SetStatic => (2, 0), // pops name & object
            GetField => (2, 1), // pops target object & key, pushes object
            SetField => (3, 0), // pops target object & key & value
            Branch(_) => (0, 0),
            ConditionalBranch(_, _) => (1, 0), // pops condition
            Return => (1, 0), // pops retval,
            IntAdd | IntSub | IntMul | IntDiv | IntMod | IntPow
                | FloatAdd | FloatSub | FloatMul | FloatDiv
                | FloatPowi | FloatPowf
                | StringAdd => (2, 1), // pops the two operands, pushes the result
            CastToFloat | CastToInt | CastToBool | CastToString => (1, 1),
            Not => (1, 1),
            TestLt | TestEq | TestGt => (2, 1), // pops the two operands, pushes the result
            Call(n_args) => (n_args + 2, 1), // pops target & this & arguments, pushes the result
            Rt(_) => panic!("Unexpected runtime opcode")
        }
    }
}
