use crate::structs::LookupSwitchPair;

#[derive(Debug)]
pub enum Instruction {
    AALoad,
    AAStore,
    ALoad(u8),
    ALoadW(u16),
    ANewArray(u16),
    ANull,
    AReturn,
    ArrayLength,
    AStore(u8),
    AStoreW(u16),
    AThrow,
    BALoad,
    BAStore,
    Bipush(u8),
    CALoad,
    CAStore,
    CheckCast(u16),
    D2F,
    D2I,
    D2L,
    DAdd,
    DALoad,
    DAStore,
    DCmpg,
    DCmpl,
    DConst(f64),
    DDiv,
    DLoad(u8),
    DLoadW(u16),
    DMul,
    DNeg,
    DRem,
    DReturn,
    DStore(u8),
    DStoreW(u16),
    DSub,
    Dup,
    Dup2,
    Dup2X1,
    Dup2X2,
    DupX1,
    DupX2,
    F2D,
    F2I,
    F2L,
    FAdd,
    FALoad,
    FAStore,
    FCmpg,
    FCmpl,
    FConst(f32),
    FDiv,
    FLoad(u8),
    FLoadW(u16),
    FMul,
    FNeg,
    FRem,
    FReturn,
    FStore(u8),
    FStoreW(u16),
    FSub,
    GetField(u16),
    GetStatic(u16),
    Goto(i16),
    GotoW(u32),
    I2B,
    I2C,
    I2D,
    I2F,
    I2L,
    I2S,
    IAdd,
    IALoad,
    IAnd,
    IAStore,
    IConst(i32),
    IDiv,
    IfAcmpeq(i16),
    IfAcmpne(i16),
    Ifeq(i16),
    Ifge(i16),
    Ifgt(i16),
    IfIcmpeq(i16),
    IfIcmpge(i16),
    IfIcmpgt(i16),
    IfIcmple(i16),
    IfIcmplt(i16),
    IfIcmpne(i16),
    Ifle(i16),
    Iflt(i16),
    Ifne(i16),
    IfNonNull(u16),
    IfNull(u16),
    IInc(u8, i8),
    IIncW(u16, u16),
    ILoad(u8),
    ILoadW(u16),
    IMul,
    INeg,
    InstanceOf(u16),
    InvokeDynamic(u16),
    InvokeInterface {
        index: u16,
        count: u8,
    },
    InvokeSpecial(u16),
    InvokeStatic(u16),
    InvokeVirtual(u16),
    IOr,
    IRem,
    IReturn,
    IShl,
    IShr,
    IStore(u8),
    IStoreW(u16),
    ISub,
    IUShr,
    IXor,
    Jsr(i16),
    JsrW(u32),
    L2D,
    L2F,
    L2I,
    LAdd,
    LALoad,
    LAnd,
    LAStore,
    LCmp,
    LConst(i64),
    Ldc(u8),
    Ldc2W(u16),
    LdcW(u16),
    LDiv,
    LLoad(u8),
    LLoadW(u16),
    LMul,
    LNeg,
    LookupSwitch {
        padding: u32,
        default: u32,
        pairs: Vec<LookupSwitchPair>,
    },
    LOr,
    LRem,
    LReturn,
    LShl,
    LShr,
    LStore(u8),
    LStoreW(u16),
    LSub,
    LUShr,
    LXor,
    MonitorEnter,
    MonitorExit,
    MultiANewArray(u16, u8),
    New(u16),
    NewArray(u8),
    Nop,
    Pop,
    Pop2,
    PutField(u16),
    PutStatic(u16),
    Ret(u8),
    RetW(u16),
    Return,
    SALoad,
    SAStore,
    Sipush(i16),
    Swap,
    TableSwitch {
        padding: u32,
        minimum: u32,
        maximum: u32,
        jump_targets: Vec<u32>,
        default: u32,
    },
}

impl Instruction {
    pub fn size(&self) -> u32 {
        match self {
            Instruction::AALoad
            | Instruction::AAStore
            | Instruction::ANull
            | Instruction::AReturn
            | Instruction::ArrayLength
            | Instruction::AThrow
            | Instruction::BALoad
            | Instruction::BAStore
            | Instruction::CALoad
            | Instruction::CAStore
            | Instruction::D2F
            | Instruction::D2I
            | Instruction::D2L
            | Instruction::DAdd
            | Instruction::DALoad
            | Instruction::DAStore
            | Instruction::DCmpg
            | Instruction::DCmpl
            | Instruction::DConst(..)
            | Instruction::DDiv
            | Instruction::DMul
            | Instruction::DNeg
            | Instruction::DRem
            | Instruction::DReturn
            | Instruction::DSub
            | Instruction::Dup
            | Instruction::Dup2
            | Instruction::Dup2X1
            | Instruction::Dup2X2
            | Instruction::DupX1
            | Instruction::DupX2
            | Instruction::F2D
            | Instruction::F2I
            | Instruction::F2L
            | Instruction::FAdd
            | Instruction::FALoad
            | Instruction::FAStore
            | Instruction::FCmpg
            | Instruction::FCmpl
            | Instruction::FConst(..)
            | Instruction::FDiv
            | Instruction::FMul
            | Instruction::FNeg
            | Instruction::FRem
            | Instruction::FReturn
            | Instruction::FSub
            | Instruction::I2B
            | Instruction::I2C
            | Instruction::I2D
            | Instruction::I2F
            | Instruction::I2L
            | Instruction::I2S
            | Instruction::IAdd
            | Instruction::IALoad
            | Instruction::IAnd
            | Instruction::IAStore
            | Instruction::IConst(..)
            | Instruction::IDiv
            | Instruction::IMul
            | Instruction::INeg
            | Instruction::IOr
            | Instruction::IRem
            | Instruction::IReturn
            | Instruction::IShl
            | Instruction::IShr
            | Instruction::ISub
            | Instruction::IUShr
            | Instruction::IXor
            | Instruction::L2D
            | Instruction::L2F
            | Instruction::L2I
            | Instruction::LAdd
            | Instruction::LALoad
            | Instruction::LAnd
            | Instruction::LAStore
            | Instruction::LCmp
            | Instruction::LConst(..)
            | Instruction::LDiv
            | Instruction::LMul
            | Instruction::LNeg
            | Instruction::LOr
            | Instruction::LRem
            | Instruction::LReturn
            | Instruction::LShl
            | Instruction::LShr
            | Instruction::LSub
            | Instruction::LUShr
            | Instruction::LXor
            | Instruction::MonitorEnter
            | Instruction::MonitorExit
            | Instruction::Nop
            | Instruction::Pop
            | Instruction::Pop2
            | Instruction::Return
            | Instruction::SALoad
            | Instruction::SAStore
            | Instruction::Swap => 1,
            Instruction::ALoad(index)
            | Instruction::DLoad(index)
            | Instruction::FLoad(index)
            | Instruction::ILoad(index)
            | Instruction::LLoad(index) => {
                if *index < 4 {
                    1
                } else {
                    2
                }
            }
            Instruction::AStore(index)
            | Instruction::DStore(index)
            | Instruction::FStore(index)
            | Instruction::IStore(index)
            | Instruction::LStore(index) => {
                if *index < 4 {
                    1
                } else {
                    2
                }
            }
            Instruction::Bipush(..)
            | Instruction::Ldc(..)
            | Instruction::NewArray(..)
            | Instruction::Ret(..) => 2,
            Instruction::ANewArray(..)
            | Instruction::CheckCast(..)
            | Instruction::GetField(..)
            | Instruction::GetStatic(..)
            | Instruction::Goto(..)
            | Instruction::IfAcmpeq(..)
            | Instruction::IfAcmpne(..)
            | Instruction::Ifeq(..)
            | Instruction::Ifge(..)
            | Instruction::Ifgt(..)
            | Instruction::IfIcmpeq(..)
            | Instruction::IfIcmpge(..)
            | Instruction::IfIcmpgt(..)
            | Instruction::IfIcmple(..)
            | Instruction::IfIcmplt(..)
            | Instruction::IfIcmpne(..)
            | Instruction::Ifle(..)
            | Instruction::Iflt(..)
            | Instruction::Ifne(..)
            | Instruction::IfNonNull(..)
            | Instruction::IfNull(..)
            | Instruction::IInc(..)
            | Instruction::InstanceOf(..)
            | Instruction::InvokeSpecial(..)
            | Instruction::InvokeStatic(..)
            | Instruction::InvokeVirtual(..)
            | Instruction::Jsr(..)
            | Instruction::Ldc2W(..)
            | Instruction::LdcW(..)
            | Instruction::New(..)
            | Instruction::PutField(..)
            | Instruction::PutStatic(..)
            | Instruction::Sipush(..) => 3,
            Instruction::ALoadW(..)
            | Instruction::AStoreW(..)
            | Instruction::DLoadW(..)
            | Instruction::DStoreW(..)
            | Instruction::FLoadW(..)
            | Instruction::FStoreW(..)
            | Instruction::ILoadW(..)
            | Instruction::InvokeInterface { .. }
            | Instruction::IStoreW(..)
            | Instruction::LLoadW(..)
            | Instruction::LStoreW(..)
            | Instruction::MultiANewArray(..)
            | Instruction::RetW(..) => 4,
            Instruction::GotoW(..) | Instruction::InvokeDynamic(..) | Instruction::JsrW(..) => 5,
            Instruction::IIncW(..) => 6,
            Instruction::LookupSwitch {
                padding,
                default: _,
                pairs,
            } => 1 + padding + 8 + pairs.len() as u32 * 8,
            Instruction::TableSwitch {
                padding,
                minimum: _,
                maximum: _,
                jump_targets,
                default: _,
            } => 1 + padding + 12 + jump_targets.len() as u32,
        }
    }
}
