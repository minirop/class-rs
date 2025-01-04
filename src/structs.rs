use crate::enums::{AccessFlag, Attribute, ElementValue, StackMapFrameType, VerificationType};

#[derive(Debug)]
pub struct MemberData {
    pub access_flags: Vec<AccessFlag>,
    pub name: u16,
    pub descriptor: u16,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug)]
pub struct Field(pub MemberData);

#[derive(Debug)]
pub struct Method(pub MemberData);

#[derive(Debug)]
pub struct LineNumber {
    pub start_pc: u16,
    pub line_number: u16,
}

#[derive(Debug)]
pub struct Annotation {
    pub type_index: u16,
    pub element_value_pairs: Vec<ElementValuePair>,
}

#[derive(Debug)]
pub struct ElementValuePair {
    pub element_name_index: u16,
    pub value: ElementValue,
}

#[derive(Debug)]
pub struct LookupSwitchPair {
    pub value: u32,
    pub target: u32,
}

#[derive(Debug)]
pub struct BootstrapMethod {
    pub bootstrap_method_ref: u16,
    pub bootstrap_arguments: Vec<u16>,
}

#[derive(Debug)]
pub struct InnerClass {
    pub inner_class_info_index: u16,
    pub outer_class_info_index: u16,
    pub inner_name_index: u16,
    pub inner_class_access_flags: Vec<AccessFlag>,
}

#[derive(Debug)]
pub struct StackMapFrame {
    pub frame_type: StackMapFrameType,
    pub offset_delta: u16,
    pub locals: Vec<VerificationType>,
    pub stack: Vec<VerificationType>,
}

#[derive(Debug)]
pub struct LocalVariable {
    pub start_pc: u16,
    pub length: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub index: u16,
}

#[derive(Debug)]
pub struct LocalVariableType {
    pub start_pc: u16,
    pub length: u16,
    pub name_index: u16,
    pub signature_index: u16,
    pub index: u16,
}
