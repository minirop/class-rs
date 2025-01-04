//! JVM class file reader
//!
//! Reads a .class file into an almost 1-to-1 matching struct.

use byteorder::{BigEndian, ReadBytesExt};
use std::error::Error;
use std::io::Read;

mod enums;
pub use enums::{
    AccessFlag, Attribute, Constant, ElementValue, Instruction, StackMapFrameType, TargetInfo,
    VerificationType,
};

mod structs;
pub use structs::{
    Annotation, BootstrapMethod, ElementValuePair, Field, InnerClass, LineNumber, LocalVar,
    LocalVariable, LocalVariableType, LookupSwitchPair, MemberData, Method, MethodParameter,
    ModuleExports, ModuleOpens, ModuleProvides, ModuleRequires, StackMapFrame, TypeAnnotation,
    TypePath,
};

mod parser;
use crate::parser::{
    extract_class_flags, read_attributes, read_constant_pool, read_fields, read_interfaces,
    read_methods,
};

mod errors;
pub use errors::JavaError;

#[derive(Debug)]
pub struct JVMClass {
    pub major: u16,
    pub minor: u16,
    pub access_flags: Vec<AccessFlag>,
    pub this_class: u16,
    pub super_class: u16,
    pub constants: Vec<Constant>,
    pub interfaces: Vec<u16>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub attributes: Vec<Attribute>,
}

impl JVMClass {
    pub fn new() -> Self {
        Self {
            major: 0,
            minor: 0,
            access_flags: vec![],
            this_class: 0,
            super_class: 0,
            constants: vec![],
            interfaces: vec![],
            fields: vec![],
            methods: vec![],
            attributes: vec![],
        }
    }

    pub fn load<R: Read>(&mut self, mut r: &mut R) -> Result<(), Box<dyn Error>> {
        //let mut f = File::open(filename)?;
        let magic = r.read_u32::<BigEndian>()?;
        assert_eq!(magic, 0xCAFEBABE);

        self.minor = r.read_u16::<BigEndian>()?;
        self.major = r.read_u16::<BigEndian>()?;

        self.constants = read_constant_pool(&mut r)?;

        let access_flags = r.read_u16::<BigEndian>()?;
        self.access_flags = extract_class_flags(access_flags);

        self.this_class = r.read_u16::<BigEndian>()?;
        self.super_class = r.read_u16::<BigEndian>()?;

        self.interfaces = read_interfaces(&mut r)?;
        self.fields = read_fields(&self, &mut r)?;
        self.methods = read_methods(&self, &mut r)?;
        self.attributes = read_attributes(&self, &mut r)?;

        Ok(())
    }

    pub fn get_string(&self, id: u16) -> Result<&str, JavaError> {
        let id = id as usize;

        if let Some(constant) = self.constants.get(id) {
            match constant {
                Constant::Class { name_index } => self.get_string(*name_index),
                Constant::Utf8(string) => Ok(string),
                Constant::String { string_index } => self.get_string(*string_index),
                _ => Err(JavaError::ConstantTypeError(format!(
                    "#{id} is not a string, but a {constant}"
                ))),
            }
        } else {
            Err(JavaError::InvalidConstantId)
        }
    }
}
