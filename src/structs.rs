use crate::enums::{
    AccessFlag, Attribute, ElementValue, StackMapFrameType, TargetInfo, VerificationType,
};

#[derive(Debug, Clone)]
pub struct MemberData {
    pub access_flags: Vec<AccessFlag>,
    pub name: u16,
    pub descriptor: u16,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub struct Field(pub MemberData);

#[derive(Debug, Clone)]
pub struct Method(pub MemberData);

#[derive(Debug, Clone)]
pub struct LineNumber {
    pub start_pc: u16,
    pub line_number: u16,
}

#[derive(Debug, Clone)]
pub struct Annotation {
    pub type_index: u16,
    pub element_value_pairs: Vec<ElementValuePair>,
}

#[derive(Debug, Clone)]
pub struct ElementValuePair {
    pub element_name_index: u16,
    pub value: ElementValue,
}

#[derive(Debug, Clone)]
pub struct LookupSwitchPair {
    pub value: u32,
    pub target: u32,
}

#[derive(Debug, Clone)]
pub struct BootstrapMethod {
    pub bootstrap_method_ref: u16,
    pub bootstrap_arguments: Vec<u16>,
}

#[derive(Debug, Clone)]
pub struct InnerClass {
    pub inner_class_info_index: u16,
    pub outer_class_info_index: u16,
    pub inner_name_index: u16,
    pub inner_class_access_flags: Vec<AccessFlag>,
}

#[derive(Debug, Clone)]
pub struct StackMapFrame {
    pub frame_type: StackMapFrameType,
    pub offset_delta: u16,
    pub locals: Vec<VerificationType>,
    pub stack: Vec<VerificationType>,
}

#[derive(Debug, Clone)]
pub struct LocalVariable {
    pub start_pc: u16,
    pub length: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub index: u16,
}

#[derive(Debug, Clone)]
pub struct LocalVariableType {
    pub start_pc: u16,
    pub length: u16,
    pub name_index: u16,
    pub signature_index: u16,
    pub index: u16,
}

#[derive(Debug, Clone)]
pub struct MethodParameter {
    pub name_index: u16,
    pub access_flags: Vec<AccessFlag>,
}

#[derive(Debug, Clone)]
pub struct ModuleRequires {
    pub requires_index: u16,
    pub requires_flags: Vec<AccessFlag>,
    pub requires_version_index: u16,
}

#[derive(Debug, Clone)]
pub struct ModuleExports {
    pub exports_index: u16,
    pub exports_flags: Vec<AccessFlag>,
    pub exports_to_index: Vec<u16>,
}

#[derive(Debug, Clone)]
pub struct ModuleOpens {
    pub opens_index: u16,
    pub opens_flags: Vec<AccessFlag>,
    pub opens_to_index: Vec<u16>,
}

#[derive(Debug, Clone)]
pub struct ModuleProvides {
    pub provides_index: u16,
    pub provides_with_index: Vec<u16>,
}

#[derive(Debug, Clone)]
pub struct RecordComponent {
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub struct LocalVar {
    pub start_pc: u16,
    pub length: u16,
    pub index: u16,
}

#[derive(Debug, Clone)]
pub struct TypePath {
    pub type_path_kind: u8,
    pub type_argument_index: u8,
}

#[derive(Debug, Clone)]
pub struct TypeAnnotation {
    pub target_info: TargetInfo,
    pub target_path: Vec<TypePath>,
    pub annotation: Annotation,
}
