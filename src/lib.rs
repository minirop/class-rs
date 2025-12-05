//! JVM class file reader
//!
//! Reads a .class file into an almost 1-to-1 matching struct or generates a .class file from said structure.

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::error::Error;
use std::io::{Read, Seek, Write};

mod enums;
pub use enums::{
    AccessFlag, Attribute, Constant, ElementValue, Instruction, StackMapFrameType, TargetInfo,
    VerificationType,
};

mod structs;
pub use structs::{
    Annotation, BootstrapMethod, ElementValuePair, ExceptionTableEntry, Field, InnerClass,
    LineNumber, LocalVar, LocalVariable, LocalVariableType, LookupSwitchPair, MemberData, Method,
    MethodParameter, ModuleExports, ModuleOpens, ModuleProvides, ModuleRequires, StackMapFrame,
    TypeAnnotation, TypePath,
};

mod reader;
use crate::reader::{
    extract_class_flags, read_attributes, read_constant_pool, read_fields, read_interfaces,
    read_methods,
};

mod writer;
use crate::writer::{
    compact_class_flags, write_attributes, write_constant_pool, write_fields, write_interfaces,
    write_methods,
};

mod errors;
pub use errors::JavaError;

mod mapping;

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

    pub fn load<R: Read>(&mut self, r: &mut R) -> Result<(), Box<dyn Error>> {
        let magic = r.read_u32::<BigEndian>()?;
        assert_eq!(magic, 0xCAFEBABE);

        self.minor = r.read_u16::<BigEndian>()?;
        self.major = r.read_u16::<BigEndian>()?;

        self.constants = read_constant_pool(r)?;

        let access_flags = r.read_u16::<BigEndian>()?;
        self.access_flags = extract_class_flags(access_flags);

        self.this_class = r.read_u16::<BigEndian>()?;
        self.super_class = r.read_u16::<BigEndian>()?;

        self.interfaces = read_interfaces(r)?;
        self.fields = read_fields(&self, r)?;
        self.methods = read_methods(&self, r)?;
        self.attributes = read_attributes(&self, r)?;

        Ok(())
    }

    pub fn store<W: Write + Seek>(&self, w: &mut W) -> Result<(), Box<dyn Error>> {
        w.write_u32::<BigEndian>(0xCAFEBABE)?;

        w.write_u16::<BigEndian>(self.minor)?;
        w.write_u16::<BigEndian>(self.major)?;

        write_constant_pool(w, &self.constants)?;

        let access_flags = compact_class_flags(&self.access_flags);
        w.write_u16::<BigEndian>(access_flags)?;

        w.write_u16::<BigEndian>(self.this_class)?;
        w.write_u16::<BigEndian>(self.super_class)?;

        write_interfaces(w, &self.interfaces)?;
        write_fields(w, &self.fields, self)?;
        write_methods(w, &self.methods, self)?;
        write_attributes(w, &self.attributes, self)?;

        Ok(())
    }

    pub fn get_string(&self, id: u16) -> Result<&str, JavaError> {
        let id = id as usize;

        if let Some(constant) = self.constants.get(id) {
            match constant {
                Constant::Class { name_index } => {
                    let cname = self.get_string(*name_index)?;

                    if cname.starts_with("java/lang/") {
                        Ok(&cname[10..])
                    } else {
                        Ok(cname)
                    }
                }
                Constant::Utf8(string) => Ok(string),
                Constant::String { string_index } => self.get_string(*string_index),
                _ => Err(JavaError::ConstantTypeError(format!(
                    "#{id} is not a string, but a {constant}"
                ))),
            }
        } else {
            Err(JavaError::InvalidConstantId(id as u16))
        }
    }

    pub fn get_string_index(&self, string: &str) -> Result<u16, JavaError> {
        for (index, constant) in self.constants.iter().enumerate() {
            if let Constant::Utf8(s) = constant {
                if s == string {
                    return Ok(index as u16);
                }
            }
        }

        Err(JavaError::StringNotFound)
    }
}
