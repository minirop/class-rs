use crate::AccessFlag;

pub const CLASS_FLAGS: [(u16, AccessFlag); 9] = [
    (0x0001, AccessFlag::Public),
    (0x0010, AccessFlag::Final),
    (0x0020, AccessFlag::Super),
    (0x0200, AccessFlag::Interface),
    (0x0400, AccessFlag::Abstract),
    (0x1000, AccessFlag::Synthetic),
    (0x2000, AccessFlag::Annotation),
    (0x4000, AccessFlag::Enum),
    (0x8000, AccessFlag::Module),
];

pub const INNER_CLASS_FLAGS: [(u16, AccessFlag); 10] = [
    (0x0001, AccessFlag::Public),
    (0x0002, AccessFlag::Private),
    (0x0004, AccessFlag::Protected),
    (0x0008, AccessFlag::Static),
    (0x0010, AccessFlag::Final),
    (0x0200, AccessFlag::Interface),
    (0x0400, AccessFlag::Abstract),
    (0x1000, AccessFlag::Synthetic),
    (0x2000, AccessFlag::Annotation),
    (0x4000, AccessFlag::Enum),
];

pub const FIELD_FLAGS: [(u16, AccessFlag); 9] = [
    (0x0001, AccessFlag::Public),
    (0x0002, AccessFlag::Private),
    (0x0004, AccessFlag::Protected),
    (0x0008, AccessFlag::Static),
    (0x0010, AccessFlag::Final),
    (0x0040, AccessFlag::Volatile),
    (0x0080, AccessFlag::Transient),
    (0x1000, AccessFlag::Synthetic),
    (0x4000, AccessFlag::Enum),
];

pub const METHOD_FLAGS: [(u16, AccessFlag); 12] = [
    (0x0001, AccessFlag::Public),
    (0x0002, AccessFlag::Private),
    (0x0004, AccessFlag::Protected),
    (0x0008, AccessFlag::Static),
    (0x0010, AccessFlag::Final),
    (0x0020, AccessFlag::Synchronized),
    (0x0040, AccessFlag::Bridge),
    (0x0080, AccessFlag::VarArgs),
    (0x0100, AccessFlag::Native),
    (0x0400, AccessFlag::Abstract),
    (0x0800, AccessFlag::Strict),
    (0x1000, AccessFlag::Synthetic),
];

pub const METHOD_PARAMETER_FLAGS: [(u16, AccessFlag); 3] = [
    (0x0010, AccessFlag::Final),
    (0x1000, AccessFlag::Synthetic),
    (0x8000, AccessFlag::Mandated),
];

pub const MODULE_FLAGS: [(u16, AccessFlag); 3] = [
    (0x0020, AccessFlag::Open),
    (0x1000, AccessFlag::Synthetic),
    (0x8000, AccessFlag::Mandated),
];

pub const MODULE_REQUIRES_FLAGS: [(u16, AccessFlag); 4] = [
    (0x0020, AccessFlag::Transitive),
    (0x0040, AccessFlag::StaticPhase),
    (0x1000, AccessFlag::Synthetic),
    (0x8000, AccessFlag::Mandated),
];

pub const MODULE_OPENS_FLAGS: [(u16, AccessFlag); 2] = [
    (0x1000, AccessFlag::Synthetic),
    (0x8000, AccessFlag::Mandated),
];

pub const MODULE_EXPORTS_FLAGS: [(u16, AccessFlag); 2] = [
    (0x1000, AccessFlag::Synthetic),
    (0x8000, AccessFlag::Mandated),
];
