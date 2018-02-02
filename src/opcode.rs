/// Hexagon VM opcodes.
///
/// Note that the `Rt` variant is only meant to be used internally
/// by the optimizer and will not pass code validation at function
/// creation.
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
    GetLocalIndirect,
    SetLocalIndirect,
    GetArgument(usize),
    GetNArguments,
    GetStatic,
    SetStatic,
    GetField,
    SetField,
    Call(usize),
    CallField(usize),
    Branch(usize),
    ConditionalBranch(usize, usize),
    Return,
    Add,
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
    And,
    Or,
    Not,
    TestLt,
    TestLe,
    TestEq,
    TestNe,
    TestGe,
    TestGt,
    Rotate2,
    Rotate3,

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
            GetLocalIndirect => (1, 1), // pops index, pushes object
            SetLocalIndirect => (2, 0), // pops index & object
            GetArgument(_) => (0, 1), // pushes the argument
            GetNArguments => (0, 1), // pushes n_arguments
            GetStatic => (1, 1), // pops name, pushes object
            SetStatic => (2, 0), // pops name & object
            GetField => (2, 1), // pops target object & key, pushes object
            SetField => (3, 0), // pops target object & key & value
            Branch(_) => (0, 0),
            ConditionalBranch(_, _) => (1, 0), // pops condition
            Return => (1, 0), // pops retval,
            Add
                | IntAdd | IntSub | IntMul | IntDiv | IntMod | IntPow
                | FloatAdd | FloatSub | FloatMul | FloatDiv
                | FloatPowi | FloatPowf
                | StringAdd => (2, 1), // pops the two operands, pushes the result
            CastToFloat | CastToInt | CastToBool | CastToString => (1, 1),
            Not => (1, 1),
            And | Or => (2, 1), // pops the two operands, pushes the result
            TestLt | TestLe | TestEq | TestNe | TestGe | TestGt => (2, 1), // pops the two operands, pushes the result
            Call(n_args) => (n_args + 2, 1), // pops target & this & arguments, pushes the result
            CallField(n_args) => (n_args + 3, 1), // pops target & this & field_name & arguments, pushes the result
            Rotate2 => (2, 2),
            Rotate3 => (3, 3),
            Rt(ref op) => match *op {
                RtOpCode::LoadObject(_) => (0, 1) // pushes the object at id
            }
        }
    }
}
