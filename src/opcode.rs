use smallvec::SmallVec;
use call_stack::Frame;
use object_pool::ObjectPool;
use value::Value;

/// Hexagon VM opcodes.
///
/// Note that the `Rt` variant is only meant to be used internally
/// by the optimizer and will not pass code validation at function
/// creation.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum OpCode {
    Nop,
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
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
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
    RotateReverse(usize),

    #[serde(skip_serializing, skip_deserializing)]
    Rt(RtOpCode)
}

#[derive(Clone, Debug, PartialEq)]
pub enum RtOpCode {
    LoadObject(usize),
    LoadValue(Value),
    StackMap(StackMapPattern),
    ConstCall(ValueLocation /* target */, ValueLocation /* this */, usize /* n_args */)
}

#[derive(Clone, Debug, PartialEq)]
pub struct StackMapPattern {
    pub(crate) map: SmallVec<[ValueLocation; 4]>,
    pub(crate) end_state: isize
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueLocation {
    Stack(isize),
    Local(usize),
    Argument(usize),
    ConstInt(i64),
    ConstFloat(f64),
    ConstString(String),
    ConstBool(bool),
    ConstNull,
    ConstObject(usize)
}

impl ValueLocation {
    pub fn from_opcode(op: &OpCode) -> Option<ValueLocation> {
        match *op {
            OpCode::GetLocal(id) => Some(ValueLocation::Local(id)),
            OpCode::GetArgument(id) => Some(ValueLocation::Argument(id)),
            OpCode::LoadInt(v) => Some(ValueLocation::ConstInt(v)),
            OpCode::LoadFloat(v) => Some(ValueLocation::ConstFloat(v)),
            OpCode::LoadString(ref v) => Some(ValueLocation::ConstString(v.clone())),
            OpCode::LoadBool(v) => Some(ValueLocation::ConstBool(v)),
            OpCode::LoadNull => Some(ValueLocation::ConstNull),
            OpCode::Rt(RtOpCode::LoadObject(id)) => Some(ValueLocation::ConstObject(id)),
            _ => None
        }
    }

    pub fn extract(&self, frame: &Frame, pool: &mut ObjectPool) -> Value {
        let stack = unsafe { &mut *frame.exec_stack.get() };
        let center = stack.len() - 1;

        match *self {
            ValueLocation::Stack(dt) => stack[(center as isize + dt) as usize],
            ValueLocation::Local(id) => frame.get_local(id),
            ValueLocation::Argument(id) => frame.must_get_argument(id),
            ValueLocation::ConstString(ref s) => {
                Value::Object(pool.allocate(Box::new(s.clone())))
            },
            ValueLocation::ConstNull => Value::Null,
            ValueLocation::ConstInt(v) => Value::Int(v),
            ValueLocation::ConstFloat(v) => Value::Float(v),
            ValueLocation::ConstBool(v) => Value::Bool(v),
            ValueLocation::ConstObject(id) => Value::Object(id)
        }
    }
}

impl OpCode {
    pub fn get_stack_depth_change(&self) -> (usize, usize) {
        use self::OpCode::*;

        // (pop, push)
        match *self {
            Nop => (0, 0),
            LoadNull | LoadInt(_) | LoadFloat(_) | LoadString(_) | LoadBool(_) | LoadThis => (0, 1), // pushes the value
            Pop => (1, 0), // pops the object on the top
            Dup => (0, 1), // duplicates the object on the top
            InitLocal(_) => (0, 0),
            GetLocal(_) => (0, 1), // pushes object
            SetLocal(_) => (1, 0), // pops object
            GetArgument(_) => (0, 1), // pushes the argument
            GetNArguments => (0, 1), // pushes n_arguments
            GetStatic => (1, 1), // pops name, pushes object
            SetStatic => (2, 0), // pops name & object
            GetField => (2, 1), // pops target object & key, pushes object
            SetField => (3, 0), // pops target object & key & value
            Branch(_) => (0, 0),
            ConditionalBranch(_, _) => (1, 0), // pops condition
            Return => (1, 0), // pops retval,
            Add | Sub | Mul | Div | Mod | Pow
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
            RotateReverse(n) => (n, n),
            Rt(ref op) => match *op {
                RtOpCode::LoadObject(_) | RtOpCode::LoadValue(_) => (0, 1), // pushes the object at id
                RtOpCode::StackMap(ref p) => if p.end_state >= 0 {
                    (0, p.end_state as usize)
                } else {
                    ((-p.end_state) as usize, 0)
                },
                RtOpCode::ConstCall(_, _, n_args) => (n_args, 1) // pops arguments, pushes the result
            }
        }
    }
}
