use std::error::Error;
use std::io::{self, Cursor, Read, Seek, SeekFrom};

use byteorder::{BigEndian, ReadBytesExt};

use crate::enums::{
    AccessFlag, Attribute, Constant, ElementValue, Instruction, StackMapFrameType, TargetInfo,
    VerificationType,
};
use crate::structs::{
    Annotation, BootstrapMethod, ElementValuePair, ExceptionTableEntry, Field, InnerClass,
    LineNumber, LocalVar, LocalVariable, LocalVariableType, LookupSwitchPair, MemberData, Method,
    MethodParameter, ModuleExports, ModuleOpens, ModuleProvides, ModuleRequires, RecordComponent,
    StackMapFrame, TypeAnnotation, TypePath,
};
use crate::JVMClass;

use crate::mapping::{
    CLASS_FLAGS, FIELD_FLAGS, INNER_CLASS_FLAGS, METHOD_FLAGS, METHOD_PARAMETER_FLAGS,
    MODULE_EXPORTS_FLAGS, MODULE_FLAGS, MODULE_OPENS_FLAGS, MODULE_REQUIRES_FLAGS,
};

pub fn read_constant_pool<R: Read>(r: &mut R) -> Result<Vec<Constant>, io::Error> {
    let count = r.read_u16::<BigEndian>()?;

    let mut constants = vec![Constant::Invalid];

    let mut double_constants = 0;
    for i in 1..count {
        if i >= count - double_constants {
            break;
        }
        let tag = r.read_u8()?;
        let cnst = match tag {
            1 => {
                let length = r.read_u16::<BigEndian>()? as usize;
                let mut buff = vec![0u8; length as usize];
                r.read(&mut buff).unwrap();

                let string = String::from_utf8(buff).unwrap();

                Constant::Utf8(string)
            }
            3 => {
                let value = r.read_i32::<BigEndian>()?;
                Constant::Integer(value)
            }
            4 => {
                let value = r.read_f32::<BigEndian>()?;
                Constant::Float(value)
            }
            5 => {
                double_constants += 1;
                let value = r.read_i64::<BigEndian>()?;
                Constant::Long(value)
            }
            6 => {
                double_constants += 1;
                let value = r.read_f64::<BigEndian>()?;
                Constant::Double(value)
            }
            7 => {
                let name_index = r.read_u16::<BigEndian>()?;

                Constant::Class { name_index }
            }
            8 => {
                let string_index = r.read_u16::<BigEndian>()?;

                Constant::String { string_index }
            }
            9 => {
                let class_index = r.read_u16::<BigEndian>()?;
                let name_and_type_index = r.read_u16::<BigEndian>()?;

                Constant::Fieldref {
                    class_index,
                    name_and_type_index,
                }
            }
            10 => {
                let class_index = r.read_u16::<BigEndian>()?;
                let name_and_type_index = r.read_u16::<BigEndian>()?;

                Constant::Methodref {
                    class_index,
                    name_and_type_index,
                }
            }
            11 => {
                let class_index = r.read_u16::<BigEndian>()?;
                let name_and_type_index = r.read_u16::<BigEndian>()?;

                Constant::InterfaceMethodref {
                    class_index,
                    name_and_type_index,
                }
            }
            12 => {
                let name_index = r.read_u16::<BigEndian>()?;
                let descriptor_index = r.read_u16::<BigEndian>()?;

                Constant::NameAndType {
                    name_index,
                    descriptor_index,
                }
            }
            15 => {
                let reference_kind = r.read_u8()?;
                let reference_index = r.read_u16::<BigEndian>()?;

                Constant::MethodHandle {
                    reference_kind,
                    reference_index,
                }
            }
            16 => {
                let descriptor_index = r.read_u16::<BigEndian>()?;

                Constant::MethodType { descriptor_index }
            }
            17 => {
                let bootstrap_method_attr_index = r.read_u16::<BigEndian>()?;
                let name_and_type_index = r.read_u16::<BigEndian>()?;

                Constant::Dynamic {
                    bootstrap_method_attr_index,
                    name_and_type_index,
                }
            }
            18 => {
                let bootstrap_method_attr_index = r.read_u16::<BigEndian>()?;
                let name_and_type_index = r.read_u16::<BigEndian>()?;

                Constant::InvokeDynamic {
                    bootstrap_method_attr_index,
                    name_and_type_index,
                }
            }
            19 => {
                let name_index = r.read_u16::<BigEndian>()?;

                Constant::Module { name_index }
            }
            20 => {
                let name_index = r.read_u16::<BigEndian>()?;

                Constant::Package { name_index }
            }
            _ => panic!("Unknown constant type: {tag}"),
        };

        match cnst {
            Constant::Double(..) | Constant::Long(..) => {
                constants.push(cnst);
                constants.push(Constant::Invalid);
            }
            _ => constants.push(cnst),
        }
    }

    Ok(constants)
}

fn extract_flags<T: Copy>(flags: u16, mapping: &[(u16, T)]) -> Vec<T> {
    mapping
        .iter()
        .filter(|(value, _)| (value & flags) != 0)
        .map(|(_, e)| *e)
        .collect::<Vec<_>>()
}

pub fn extract_class_flags(flags: u16) -> Vec<AccessFlag> {
    extract_flags(flags, &CLASS_FLAGS)
}

fn extract_inner_class_flags(flags: u16) -> Vec<AccessFlag> {
    extract_flags(flags, &INNER_CLASS_FLAGS)
}

fn extract_field_flags(flags: u16) -> Vec<AccessFlag> {
    extract_flags(flags, &FIELD_FLAGS)
}

fn extract_method_flags(flags: u16) -> Vec<AccessFlag> {
    extract_flags(flags, &METHOD_FLAGS)
}

fn extract_method_parameter_flags(flags: u16) -> Vec<AccessFlag> {
    extract_flags(flags, &METHOD_PARAMETER_FLAGS)
}

fn extract_module_flags(flags: u16) -> Vec<AccessFlag> {
    extract_flags(flags, &MODULE_FLAGS)
}

fn extract_module_requires_flags(flags: u16) -> Vec<AccessFlag> {
    extract_flags(flags, &MODULE_REQUIRES_FLAGS)
}

fn extract_module_opens_flags(flags: u16) -> Vec<AccessFlag> {
    extract_flags(flags, &MODULE_OPENS_FLAGS)
}

fn extract_module_exports_flags(flags: u16) -> Vec<AccessFlag> {
    extract_flags(flags, &MODULE_EXPORTS_FLAGS)
}

pub fn read_interfaces<R: Read>(r: &mut R) -> Result<Vec<u16>, io::Error> {
    let count = r.read_u16::<BigEndian>()?;

    let mut interfaces = vec![];

    for _ in 0..count {
        let id = r.read_u16::<BigEndian>()?;
        interfaces.push(id);
    }

    Ok(interfaces)
}

pub fn read_fields<R: Read>(jvm: &JVMClass, r: &mut R) -> Result<Vec<Field>, Box<dyn Error>> {
    let count = r.read_u16::<BigEndian>()?;

    let mut fields = vec![];

    for _ in 0..count {
        let access_flags = r.read_u16::<BigEndian>()?;
        let access_flags = extract_field_flags(access_flags);
        let name = r.read_u16::<BigEndian>()?;
        let descriptor = r.read_u16::<BigEndian>()?;
        let attributes = read_attributes(jvm, r)?;

        fields.push(Field(MemberData {
            access_flags,
            name,
            descriptor,
            attributes,
        }));
    }

    Ok(fields)
}

pub fn read_methods<R: Read>(jvm: &JVMClass, r: &mut R) -> Result<Vec<Method>, Box<dyn Error>> {
    let count = r.read_u16::<BigEndian>()?;

    let mut methods = vec![];

    for _ in 0..count {
        let access_flags = r.read_u16::<BigEndian>()?;
        let access_flags = extract_method_flags(access_flags);
        let name = r.read_u16::<BigEndian>()?;
        let descriptor = r.read_u16::<BigEndian>()?;
        let attributes = read_attributes(jvm, r)?;

        methods.push(Method(MemberData {
            access_flags,
            name,
            descriptor,
            attributes,
        }));
    }

    Ok(methods)
}

pub fn read_annotations<R: Read>(r: &mut R) -> Result<Vec<Annotation>, io::Error> {
    let num_annotations = r.read_u16::<BigEndian>()?;

    let mut annotations = vec![];

    for _ in 0..num_annotations {
        let annotation = read_annotation(r)?;
        annotations.push(annotation);
    }

    Ok(annotations)
}

fn read_annotation<R: Read>(r: &mut R) -> Result<Annotation, io::Error> {
    let type_index = r.read_u16::<BigEndian>()?;
    let num_element_value_pairs = r.read_u16::<BigEndian>()?;

    let mut element_value_pairs = vec![];

    for _ in 0..num_element_value_pairs {
        let element_name_index = r.read_u16::<BigEndian>()?;
        let value = read_element_value(r)?;

        element_value_pairs.push(ElementValuePair {
            element_name_index,
            value,
        });
    }

    Ok(Annotation {
        type_index,
        element_value_pairs,
    })
}

fn read_element_value<R: Read>(r: &mut R) -> Result<ElementValue, io::Error> {
    let tag = r.read_u8()?;
    Ok(match tag {
        b'B' | b'C' | b'D' | b'F' | b'I' | b'J' | b'S' | b'Z' | b's' => {
            let const_value_index = r.read_u16::<BigEndian>()?;
            ElementValue::ConstValueIndex {
                tag,
                const_value_index,
            }
        }
        b'c' => {
            let class_info_index = r.read_u16::<BigEndian>()?;
            ElementValue::ClassInfoIndex(class_info_index)
        }
        b'e' => {
            let type_name_index = r.read_u16::<BigEndian>()?;
            let const_name_index = r.read_u16::<BigEndian>()?;
            ElementValue::EnumConstValue {
                type_name_index,
                const_name_index,
            }
        }
        b'@' => {
            let annotation = read_annotation(r)?;
            ElementValue::AnnotationValue(annotation)
        }
        b'[' => {
            let num_values = r.read_u16::<BigEndian>()?;

            let mut values = vec![];
            for _ in 0..num_values {
                let value = read_element_value(r)?;
                values.push(value);
            }

            ElementValue::ArrayValue(values)
        }
        _ => unreachable!(),
    })
}

pub fn read_attributes<R: Read>(
    jvm: &JVMClass,
    r: &mut R,
) -> Result<Vec<Attribute>, Box<dyn Error>> {
    let attributes_count = r.read_u16::<BigEndian>()?;

    let mut attributes = vec![];

    for _ in 0..attributes_count {
        let attribute_name_index = r.read_u16::<BigEndian>()?;
        let attribute_length = r.read_u32::<BigEndian>()?;

        let name = jvm.get_string(attribute_name_index)?;
        let attr = match name {
            "ConstantValue" => {
                let constantvalue_index = r.read_u16::<BigEndian>()?;
                Attribute::ConstantValue {
                    constantvalue_index,
                }
            }
            "Code" => {
                let max_stack = r.read_u16::<BigEndian>()?;
                let max_locals = r.read_u16::<BigEndian>()?;
                let code = decompile(r)?;

                let exception_table_length = r.read_u16::<BigEndian>()?;
                let mut exception_table = vec![];
                for _ in 0..exception_table_length {
                    let start_pc = r.read_u16::<BigEndian>()?;
                    let end_pc = r.read_u16::<BigEndian>()?;
                    let handler_pc = r.read_u16::<BigEndian>()?;
                    let catch_type = r.read_u16::<BigEndian>()?;

                    exception_table.push(ExceptionTableEntry {
                        start_pc,
                        end_pc,
                        handler_pc,
                        catch_type,
                    });
                }
                let attributes = read_attributes(jvm, r)?;

                Attribute::Code {
                    code,
                    max_stack,
                    max_locals,
                    exception_table,
                    attributes,
                }
            }
            "StackMapTable" => {
                let number_of_entries = r.read_u16::<BigEndian>()?;

                let mut frames = vec![];
                for _ in 0..number_of_entries {
                    let mut frame = StackMapFrame {
                        frame_type: StackMapFrameType::SameFrame(0),
                        offset_delta: 0,
                        locals: vec![],
                        stack: vec![],
                    };

                    let frame_type = r.read_u8()?;
                    frame.frame_type = match frame_type {
                        0..=63 => StackMapFrameType::SameFrame(frame_type),
                        64..=127 => {
                            frame.stack.push(read_verification_type(r)?);
                            StackMapFrameType::SameLocals1StackItemFrame(frame_type)
                        }
                        247 => {
                            frame.stack.push(read_verification_type(r)?);
                            StackMapFrameType::SameLocals1StackItemFrameExtended
                        }
                        248..=250 => {
                            frame.offset_delta = r.read_u16::<BigEndian>()?;
                            StackMapFrameType::ChopFrame(frame_type)
                        }
                        251 => {
                            frame.offset_delta = r.read_u16::<BigEndian>()?;
                            StackMapFrameType::SameFrameExtended
                        }
                        252..=254 => {
                            frame.offset_delta = r.read_u16::<BigEndian>()?;

                            for _ in 0..(frame_type - 251) {
                                let verification_type = read_verification_type(r)?;
                                frame.locals.push(verification_type);
                            }

                            StackMapFrameType::AppendFrame(frame_type)
                        }
                        255 => {
                            let number_of_locals = r.read_u16::<BigEndian>()?;
                            for _ in 0..number_of_locals {
                                let verification_type = read_verification_type(r)?;
                                frame.locals.push(verification_type);
                            }

                            let number_of_stack_items = r.read_u16::<BigEndian>()?;
                            for _ in 0..number_of_stack_items {
                                let verification_type = read_verification_type(r)?;
                                frame.stack.push(verification_type);
                            }

                            StackMapFrameType::FullFrame
                        }
                        _ => unreachable!(),
                    };

                    frames.push(frame);
                }

                Attribute::StackMapTable(frames)
            }
            "Exceptions" => {
                let number_of_exceptions = r.read_u16::<BigEndian>()?;

                let mut exceptions = vec![];
                for _ in 0..number_of_exceptions {
                    exceptions.push(r.read_u16::<BigEndian>()?);
                }

                Attribute::Exceptions(exceptions)
            }
            "InnerClasses" => {
                let number_of_classes = r.read_u16::<BigEndian>()?;

                let mut inner_classes = vec![];
                for _ in 0..number_of_classes {
                    let inner_class_info_index = r.read_u16::<BigEndian>()?;
                    let outer_class_info_index = r.read_u16::<BigEndian>()?;
                    let inner_name_index = r.read_u16::<BigEndian>()?;
                    let inner_class_access_flags = r.read_u16::<BigEndian>()?;
                    let inner_class_access_flags =
                        extract_inner_class_flags(inner_class_access_flags);

                    inner_classes.push(InnerClass {
                        inner_class_info_index,
                        outer_class_info_index,
                        inner_name_index,
                        inner_class_access_flags,
                    });
                }

                Attribute::InnerClasses(inner_classes)
            }
            "EnclosingMethod" => {
                assert_eq!(attribute_length, 4);
                let class_index = r.read_u16::<BigEndian>()?;
                let method_index = r.read_u16::<BigEndian>()?;

                Attribute::EnclosingMethod {
                    class_index,
                    method_index,
                }
            }
            "Synthetic" => {
                assert_eq!(attribute_length, 0);
                Attribute::Synthetic
            }
            "Signature" => {
                let signature_index = r.read_u16::<BigEndian>()?;
                Attribute::Signature { signature_index }
            }
            "SourceFile" => {
                assert_eq!(attribute_length, 2);

                let sourcefile_index = r.read_u16::<BigEndian>()?;
                Attribute::SourceFile { sourcefile_index }
            }
            "SourceDebugExtension" => {
                let mut debug_extension = vec![0u8; attribute_length as usize];
                r.read(&mut debug_extension)?;

                Attribute::SourceDebugExtension { debug_extension }
            }
            "LineNumberTable" => {
                let line_number_table_length = r.read_u16::<BigEndian>()?;

                let mut line_number_table = vec![];
                for _ in 0..line_number_table_length {
                    let start_pc = r.read_u16::<BigEndian>()?;
                    let line_number = r.read_u16::<BigEndian>()?;

                    line_number_table.push(LineNumber {
                        start_pc,
                        line_number,
                    });
                }

                Attribute::LineNumberTable(line_number_table)
            }
            "LocalVariableTable" => {
                let local_variable_table_length = r.read_u16::<BigEndian>()?;

                let mut local_variable_table = vec![];
                for _ in 0..local_variable_table_length {
                    let start_pc = r.read_u16::<BigEndian>()?;
                    let length = r.read_u16::<BigEndian>()?;
                    let name_index = r.read_u16::<BigEndian>()?;
                    let descriptor_index = r.read_u16::<BigEndian>()?;
                    let index = r.read_u16::<BigEndian>()?;

                    local_variable_table.push(LocalVariable {
                        start_pc,
                        length,
                        name_index,
                        descriptor_index,
                        index,
                    });
                }

                Attribute::LocalVariableTable(local_variable_table)
            }
            "LocalVariableTypeTable" => {
                let local_variable_type_table_length = r.read_u16::<BigEndian>()?;

                let mut local_variable_type_table = vec![];
                for _ in 0..local_variable_type_table_length {
                    let start_pc = r.read_u16::<BigEndian>()?;
                    let length = r.read_u16::<BigEndian>()?;
                    let name_index = r.read_u16::<BigEndian>()?;
                    let signature_index = r.read_u16::<BigEndian>()?;
                    let index = r.read_u16::<BigEndian>()?;

                    local_variable_type_table.push(LocalVariableType {
                        start_pc,
                        length,
                        name_index,
                        signature_index,
                        index,
                    });
                }

                Attribute::LocalVariableTypeTable(local_variable_type_table)
            }
            "Deprecated" => {
                assert_eq!(attribute_length, 0);
                Attribute::Deprecated
            }
            "RuntimeVisibleAnnotations" => {
                let annotations = read_annotations(r)?;
                Attribute::RuntimeVisibleAnnotations(annotations)
            }
            "RuntimeInvisibleAnnotations" => {
                let annotations = read_annotations(r)?;
                Attribute::RuntimeInvisibleAnnotations(annotations)
            }
            "RuntimeVisibleParameterAnnotations" => {
                let num_parameters = r.read_u8()?;

                let mut parameters_annotations = vec![];
                for _ in 0..num_parameters {
                    let annotations = read_annotations(r)?;
                    parameters_annotations.push(annotations);
                }

                Attribute::RuntimeVisibleParameterAnnotations(parameters_annotations)
            }
            "RuntimeInvisibleParameterAnnotations" => {
                let num_parameters = r.read_u8()?;

                let mut parameters_annotations = vec![];
                for _ in 0..num_parameters {
                    let annotations = read_annotations(r)?;
                    parameters_annotations.push(annotations);
                }

                Attribute::RuntimeInvisibleParameterAnnotations(parameters_annotations)
            }
            "AnnotationDefault" => {
                let element_value = read_element_value(r)?;
                Attribute::AnnotationDefault(element_value)
            }
            "BootstrapMethods" => {
                let num_bootstrap_methods = r.read_u16::<BigEndian>()?;

                let mut bootstrap_methods = vec![];

                for _ in 0..num_bootstrap_methods {
                    let bootstrap_method_ref = r.read_u16::<BigEndian>()?;
                    let num_bootstrap_arguments = r.read_u16::<BigEndian>()?;

                    let mut bootstrap_arguments = vec![];
                    for _ in 0..num_bootstrap_arguments {
                        let bootstrap_argument = r.read_u16::<BigEndian>()?;
                        bootstrap_arguments.push(bootstrap_argument);
                    }

                    bootstrap_methods.push(BootstrapMethod {
                        bootstrap_method_ref,
                        bootstrap_arguments,
                    });
                }

                Attribute::BootstrapMethods(bootstrap_methods)
            }
            "MethodParameters" => {
                let parameters_count = r.read_u8()?;

                let mut parameters = vec![];
                for _ in 0..parameters_count {
                    let name_index = r.read_u16::<BigEndian>()?;
                    let access_flags = r.read_u16::<BigEndian>()?;
                    let access_flags = extract_method_parameter_flags(access_flags);
                    parameters.push(MethodParameter {
                        name_index,
                        access_flags,
                    });
                }

                Attribute::MethodParameters(parameters)
            }
            "Module" => {
                let module_name_index = r.read_u16::<BigEndian>()?;
                let module_flags = r.read_u16::<BigEndian>()?;
                let module_flags = extract_module_flags(module_flags);
                let module_version_index = r.read_u16::<BigEndian>()?;
                let requires = read_module_requires(r)?;
                let exports = read_module_exports(r)?;
                let opens = read_module_opens(r)?;

                let uses_count = r.read_u16::<BigEndian>()?;
                let mut uses = vec![];
                for _ in 0..uses_count {
                    uses.push(r.read_u16::<BigEndian>()?);
                }

                let provides = read_module_provides(r)?;

                Attribute::Module {
                    module_name_index,
                    module_flags,
                    module_version_index,
                    requires,
                    exports,
                    opens,
                    uses,
                    provides,
                }
            }
            "ModuleMainClass" => {
                assert_eq!(attribute_length, 2);
                let main_class_index = r.read_u16::<BigEndian>()?;
                Attribute::ModuleMainClass(main_class_index)
            }
            "ModulePackages" => {
                let packages_count = r.read_u16::<BigEndian>()?;

                let mut packages_index = vec![];
                for _ in 0..packages_count {
                    let package_index = r.read_u16::<BigEndian>()?;
                    packages_index.push(package_index);
                }

                Attribute::ModulePackages(packages_index)
            }
            "NestHost" => {
                assert_eq!(attribute_length, 2);
                let host_class_index = r.read_u16::<BigEndian>()?;
                Attribute::NestHost(host_class_index)
            }
            "NestMembers" => {
                let number_of_classes = r.read_u16::<BigEndian>()?;

                let mut classes = vec![];
                for _ in 0..number_of_classes {
                    let class = r.read_u16::<BigEndian>()?;
                    classes.push(class);
                }

                Attribute::NestMembers(classes)
            }
            "PermittedSubclasses" => {
                let number_of_classes = r.read_u16::<BigEndian>()?;

                let mut classes = vec![];
                for _ in 0..number_of_classes {
                    let class = r.read_u16::<BigEndian>()?;
                    classes.push(class);
                }

                Attribute::PermittedSubclasses(classes)
            }
            "Record" => {
                let components_count = r.read_u16::<BigEndian>()?;

                let mut components = vec![];
                for _ in 0..components_count {
                    let name_index = r.read_u16::<BigEndian>()?;
                    let descriptor_index = r.read_u16::<BigEndian>()?;
                    let attributes = read_attributes(jvm, r)?;

                    components.push(RecordComponent {
                        name_index,
                        descriptor_index,
                        attributes,
                    });
                }

                Attribute::Record(components)
            }
            "RuntimeInvisibleTypeAnnotations" => {
                let num_annotations = r.read_u16::<BigEndian>()?;

                let mut annotations = vec![];
                for _ in 0..num_annotations {
                    let annotation = read_type_annotation(r)?;
                    annotations.push(annotation);
                }

                Attribute::RuntimeInvisibleTypeAnnotations(annotations)
            }
            "RuntimeVisibleTypeAnnotations" => {
                let num_annotations = r.read_u16::<BigEndian>()?;

                let mut annotations = vec![];
                for _ in 0..num_annotations {
                    let annotation = read_type_annotation(r)?;
                    annotations.push(annotation);
                }

                Attribute::RuntimeVisibleTypeAnnotations(annotations)
            }
            _ => {
                let mut data = vec![0u8; attribute_length as usize];
                r.read(&mut data)?;

                Attribute::Unknown {
                    name: name.into(),
                    data,
                }
            }
        };

        attributes.push(attr);
    }

    Ok(attributes)
}

fn read_type_annotation<R: Read>(r: &mut R) -> Result<TypeAnnotation, io::Error> {
    let target_info = read_target_info(r)?;

    let mut target_path = vec![];
    let path_length = r.read_u8()?;
    for _ in 0..path_length {
        let type_path_kind = r.read_u8()?;
        let type_argument_index = r.read_u8()?;
        target_path.push(TypePath {
            type_path_kind,
            type_argument_index,
        });
    }

    let annotation = read_annotation(r)?;
    Ok(TypeAnnotation {
        target_info,
        target_path,
        annotation,
    })
}

fn read_target_info<R: Read>(r: &mut R) -> Result<TargetInfo, io::Error> {
    let target_type = r.read_u8()?;

    Ok(match target_type {
        0x00 | 0x01 => {
            let type_parameter_index = r.read_u8()?;

            TargetInfo::TypeParameter {
                target_type,
                type_parameter_index,
            }
        }
        0x10 => {
            let supertype_index = r.read_u16::<BigEndian>()?;

            TargetInfo::Supertype { supertype_index }
        }
        0x11 | 0x12 => {
            let type_parameter_index = r.read_u8()?;
            let bound_index = r.read_u8()?;

            TargetInfo::TypeParameterBound {
                target_type,
                type_parameter_index,
                bound_index,
            }
        }
        0x13 | 0x14 | 0x15 => TargetInfo::Empty(target_type),
        0x16 => {
            let formal_parameter_index = r.read_u8()?;

            TargetInfo::FormalParameter {
                formal_parameter_index,
            }
        }
        0x17 => {
            let throws_type_index = r.read_u16::<BigEndian>()?;

            TargetInfo::Throws { throws_type_index }
        }
        0x40 | 0x41 => {
            let table_length = r.read_u16::<BigEndian>()?;

            let mut table = vec![];
            for _ in 0..table_length {
                let start_pc = r.read_u16::<BigEndian>()?;
                let length = r.read_u16::<BigEndian>()?;
                let index = r.read_u16::<BigEndian>()?;
                table.push(LocalVar {
                    start_pc,
                    length,
                    index,
                });
            }

            TargetInfo::Localvar { target_type, table }
        }
        0x42 => {
            let exception_table_index = r.read_u16::<BigEndian>()?;

            TargetInfo::Catch {
                exception_table_index,
            }
        }
        0x43 | 0x44 | 0x45 | 0x46 => {
            let offset = r.read_u16::<BigEndian>()?;

            TargetInfo::Offset {
                target_type,
                offset,
            }
        }
        0x47 | 0x48 | 0x49 | 0x4A | 0x4B => {
            let offset = r.read_u16::<BigEndian>()?;
            let type_argument_index = r.read_u8()?;

            TargetInfo::TypeArgument {
                target_type,
                offset,
                type_argument_index,
            }
        }
        _ => unreachable!(),
    })
}

fn read_module_requires<R: Read>(r: &mut R) -> Result<Vec<ModuleRequires>, io::Error> {
    let requires_count = r.read_u16::<BigEndian>()?;

    let mut requires = vec![];
    for _ in 0..requires_count {
        let requires_index = r.read_u16::<BigEndian>()?;
        let requires_flags = r.read_u16::<BigEndian>()?;
        let requires_flags = extract_module_requires_flags(requires_flags);
        let requires_version_index = r.read_u16::<BigEndian>()?;

        requires.push(ModuleRequires {
            requires_index,
            requires_flags,
            requires_version_index,
        });
    }

    Ok(requires)
}

fn read_module_exports<R: Read>(r: &mut R) -> Result<Vec<ModuleExports>, io::Error> {
    let exports_count = r.read_u16::<BigEndian>()?;

    let mut exports = vec![];
    for _ in 0..exports_count {
        let exports_index = r.read_u16::<BigEndian>()?;
        let exports_flags = r.read_u16::<BigEndian>()?;
        let exports_flags = extract_module_exports_flags(exports_flags);
        let exports_to_count = r.read_u16::<BigEndian>()?;

        let mut exports_to_index = vec![];
        for _ in 0..exports_to_count {
            let export_to_index = r.read_u16::<BigEndian>()?;
            exports_to_index.push(export_to_index);
        }

        exports.push(ModuleExports {
            exports_index,
            exports_flags,
            exports_to_index,
        });
    }

    Ok(exports)
}

fn read_module_opens<R: Read>(r: &mut R) -> Result<Vec<ModuleOpens>, io::Error> {
    let opens_count = r.read_u16::<BigEndian>()?;

    let mut opens = vec![];
    for _ in 0..opens_count {
        let opens_index = r.read_u16::<BigEndian>()?;
        let opens_flags = r.read_u16::<BigEndian>()?;
        let opens_flags = extract_module_opens_flags(opens_flags);
        let opens_to_count = r.read_u16::<BigEndian>()?;

        let mut opens_to_index = vec![];
        for _ in 0..opens_to_count {
            let open_to_index = r.read_u16::<BigEndian>()?;
            opens_to_index.push(open_to_index);
        }

        opens.push(ModuleOpens {
            opens_index,
            opens_flags,
            opens_to_index,
        });
    }

    Ok(opens)
}

fn read_module_provides<R: Read>(r: &mut R) -> Result<Vec<ModuleProvides>, io::Error> {
    let provides_count = r.read_u16::<BigEndian>()?;

    let mut provides = vec![];
    for _ in 0..provides_count {
        let provides_index = r.read_u16::<BigEndian>()?;
        let provides_with_count = r.read_u16::<BigEndian>()?;

        let mut provides_with_index = vec![];
        for _ in 0..provides_with_count {
            let provide_with_index = r.read_u16::<BigEndian>()?;
            provides_with_index.push(provide_with_index);
        }

        provides.push(ModuleProvides {
            provides_index,
            provides_with_index,
        });
    }

    Ok(provides)
}

fn read_verification_type<R: Read>(r: &mut R) -> Result<VerificationType, io::Error> {
    let tag = r.read_u8()?;

    Ok(match tag {
        0 => VerificationType::Top,
        1 => VerificationType::Integer,
        2 => VerificationType::Float,
        3 => VerificationType::Double,
        4 => VerificationType::Long,
        5 => VerificationType::Null,
        6 => VerificationType::UninitializedThis,
        7 => {
            let cpool_index = r.read_u16::<BigEndian>()?;
            VerificationType::Object { cpool_index }
        }
        8 => {
            let offset = r.read_u16::<BigEndian>()?;
            VerificationType::Uninitialized { offset }
        }
        _ => unreachable!(),
    })
}

fn decompile<R: Read>(r: &mut R) -> Result<Vec<Instruction>, io::Error> {
    let mut instructions = vec![];

    let code_length = r.read_u32::<BigEndian>()? as u64;
    let mut code = vec![0u8; code_length as usize];
    r.read(&mut code).unwrap();
    let mut cursor = Cursor::new(code);

    while cursor.seek(SeekFrom::Current(0))? < code_length {
        let opcode = cursor.read_u8()?;

        let inst = match opcode {
            0x00 => Instruction::Nop,
            0x01 => Instruction::ANull,
            0x02 => Instruction::IConst(-1),
            0x03 => Instruction::IConst(0),
            0x04 => Instruction::IConst(1),
            0x05 => Instruction::IConst(2),
            0x06 => Instruction::IConst(3),
            0x07 => Instruction::IConst(4),
            0x08 => Instruction::IConst(5),
            0x09 => Instruction::LConst(0),
            0x0A => Instruction::LConst(1),
            0x0B => Instruction::FConst(0.0),
            0x0C => Instruction::FConst(1.0),
            0x0D => Instruction::FConst(2.0),
            0x0E => Instruction::DConst(0.0),
            0x0F => Instruction::DConst(1.0),
            0x10 => {
                let byte = cursor.read_u8()?;
                Instruction::Bipush(byte)
            }
            0x11 => {
                let short = cursor.read_i16::<BigEndian>()?;
                Instruction::Sipush(short)
            }
            0x12 => {
                let index = cursor.read_u8()?;
                Instruction::Ldc(index)
            }
            0x13 => {
                let index = cursor.read_u16::<BigEndian>()?;
                Instruction::LdcW(index)
            }
            0x14 => {
                let index = cursor.read_u16::<BigEndian>()?;
                Instruction::Ldc2W(index)
            }
            0x15 => {
                let index = cursor.read_u8()?;
                Instruction::ILoad(index)
            }
            0x16 => {
                let index = cursor.read_u8()?;
                Instruction::LLoad(index)
            }
            0x17 => {
                let index = cursor.read_u8()?;
                Instruction::FLoad(index)
            }
            0x18 => {
                let index = cursor.read_u8()?;
                Instruction::DLoad(index)
            }
            0x19 => {
                let index = cursor.read_u8()?;
                Instruction::ALoad(index)
            }
            0x1A => Instruction::ILoad(0),
            0x1B => Instruction::ILoad(1),
            0x1C => Instruction::ILoad(2),
            0x1D => Instruction::ILoad(3),
            0x1E => Instruction::LLoad(0),
            0x1F => Instruction::LLoad(1),
            0x20 => Instruction::LLoad(2),
            0x21 => Instruction::LLoad(3),
            0x22 => Instruction::FLoad(0),
            0x23 => Instruction::FLoad(1),
            0x24 => Instruction::FLoad(2),
            0x25 => Instruction::FLoad(3),
            0x26 => Instruction::DLoad(0),
            0x27 => Instruction::DLoad(1),
            0x28 => Instruction::DLoad(2),
            0x29 => Instruction::DLoad(3),
            0x2A => Instruction::ALoad(0),
            0x2B => Instruction::ALoad(1),
            0x2C => Instruction::ALoad(2),
            0x2D => Instruction::ALoad(3),
            0x2E => Instruction::IALoad,
            0x2F => Instruction::LALoad,
            0x30 => Instruction::FALoad,
            0x31 => Instruction::DALoad,
            0x32 => Instruction::AALoad,
            0x33 => Instruction::BALoad,
            0x34 => Instruction::CALoad,
            0x35 => Instruction::SALoad,
            0x36 => {
                let index = cursor.read_u8()?;
                Instruction::IStore(index)
            }
            0x37 => {
                let index = cursor.read_u8()?;
                Instruction::LStore(index)
            }
            0x38 => {
                let index = cursor.read_u8()?;
                Instruction::FStore(index)
            }
            0x39 => {
                let index = cursor.read_u8()?;
                Instruction::DStore(index)
            }
            0x3A => {
                let index = cursor.read_u8()?;
                Instruction::AStore(index)
            }
            0x3B => Instruction::IStore(0),
            0x3C => Instruction::IStore(1),
            0x3D => Instruction::IStore(2),
            0x3E => Instruction::IStore(3),
            0x3F => Instruction::LStore(0),
            0x40 => Instruction::LStore(1),
            0x41 => Instruction::LStore(2),
            0x42 => Instruction::LStore(3),
            0x43 => Instruction::FStore(0),
            0x44 => Instruction::FStore(1),
            0x45 => Instruction::FStore(2),
            0x46 => Instruction::FStore(3),
            0x47 => Instruction::DStore(0),
            0x48 => Instruction::DStore(1),
            0x49 => Instruction::DStore(2),
            0x4A => Instruction::DStore(3),
            0x4B => Instruction::AStore(0),
            0x4C => Instruction::AStore(1),
            0x4D => Instruction::AStore(2),
            0x4E => Instruction::AStore(3),
            0x4F => Instruction::IAStore,
            0x50 => Instruction::LAStore,
            0x51 => Instruction::FAStore,
            0x52 => Instruction::DAStore,
            0x53 => Instruction::AAStore,
            0x54 => Instruction::BAStore,
            0x55 => Instruction::CAStore,
            0x56 => Instruction::SAStore,
            0x57 => Instruction::Pop,
            0x58 => Instruction::Pop2,
            0x59 => Instruction::Dup,
            0x5A => Instruction::DupX1,
            0x5B => Instruction::DupX2,
            0x5C => Instruction::Dup2,
            0x5D => Instruction::Dup2X1,
            0x5E => Instruction::Dup2X2,
            0x5F => Instruction::Swap,
            0x60 => Instruction::IAdd,
            0x61 => Instruction::LAdd,
            0x62 => Instruction::FAdd,
            0x63 => Instruction::DAdd,
            0x64 => Instruction::ISub,
            0x65 => Instruction::LSub,
            0x66 => Instruction::FSub,
            0x67 => Instruction::DSub,
            0x68 => Instruction::IMul,
            0x69 => Instruction::LMul,
            0x6A => Instruction::FMul,
            0x6B => Instruction::DMul,
            0x6C => Instruction::IDiv,
            0x6D => Instruction::LDiv,
            0x6E => Instruction::FDiv,
            0x6F => Instruction::DDiv,
            0x70 => Instruction::IRem,
            0x71 => Instruction::LRem,
            0x72 => Instruction::FRem,
            0x73 => Instruction::DRem,
            0x74 => Instruction::INeg,
            0x75 => Instruction::LNeg,
            0x76 => Instruction::FNeg,
            0x77 => Instruction::DNeg,
            0x78 => Instruction::IShl,
            0x79 => Instruction::LShl,
            0x7A => Instruction::IShr,
            0x7B => Instruction::LShr,
            0x7C => Instruction::IUShr,
            0x7D => Instruction::LUShr,
            0x7E => Instruction::IAnd,
            0x7F => Instruction::LAnd,
            0x80 => Instruction::IOr,
            0x81 => Instruction::LOr,
            0x82 => Instruction::IXor,
            0x83 => Instruction::LXor,
            0x84 => {
                let index = r.read_u8()?;
                let count = r.read_i8()?;
                Instruction::IInc(index, count)
            }
            0x85 => Instruction::I2L,
            0x86 => Instruction::I2F,
            0x87 => Instruction::I2D,
            0x88 => Instruction::L2I,
            0x89 => Instruction::L2F,
            0x8A => Instruction::L2D,
            0x8B => Instruction::F2I,
            0x8C => Instruction::F2L,
            0x8D => Instruction::F2D,
            0x8E => Instruction::D2I,
            0x8F => Instruction::D2L,
            0x90 => Instruction::D2F,
            0x91 => Instruction::I2B,
            0x92 => Instruction::I2C,
            0x93 => Instruction::I2S,
            0x94 => Instruction::LCmp,
            0x95 => Instruction::FCmpl,
            0x96 => Instruction::FCmpg,
            0x97 => Instruction::DCmpl,
            0x98 => Instruction::DCmpg,
            0x99 => {
                let branch = cursor.read_i16::<BigEndian>()?;
                Instruction::Ifeq(branch)
            }
            0x9A => {
                let branch = cursor.read_i16::<BigEndian>()?;
                Instruction::Ifne(branch)
            }
            0x9B => {
                let branch = cursor.read_i16::<BigEndian>()?;
                Instruction::Iflt(branch)
            }
            0x9C => {
                let branch = cursor.read_i16::<BigEndian>()?;
                Instruction::Ifge(branch)
            }
            0x9D => {
                let branch = cursor.read_i16::<BigEndian>()?;
                Instruction::Ifgt(branch)
            }
            0x9E => {
                let branch = cursor.read_i16::<BigEndian>()?;
                Instruction::Ifle(branch)
            }
            0x9F => {
                let branch = cursor.read_i16::<BigEndian>()?;
                Instruction::IfIcmpeq(branch)
            }
            0xA0 => {
                let branch = cursor.read_i16::<BigEndian>()?;
                Instruction::IfIcmpne(branch)
            }
            0xA1 => {
                let branch = cursor.read_i16::<BigEndian>()?;
                Instruction::IfIcmplt(branch)
            }
            0xA2 => {
                let branch = cursor.read_i16::<BigEndian>()?;
                Instruction::IfIcmpge(branch)
            }
            0xA3 => {
                let branch = cursor.read_i16::<BigEndian>()?;
                Instruction::IfIcmpgt(branch)
            }
            0xA4 => {
                let branch = cursor.read_i16::<BigEndian>()?;
                Instruction::IfIcmple(branch)
            }
            0xA5 => {
                let branch = cursor.read_i16::<BigEndian>()?;
                Instruction::IfAcmpeq(branch)
            }
            0xA6 => {
                let branch = cursor.read_i16::<BigEndian>()?;
                Instruction::IfAcmpne(branch)
            }
            0xA7 => {
                let branch = cursor.read_i16::<BigEndian>()?;
                Instruction::Goto(branch)
            }
            0xA8 => {
                let branch = cursor.read_i16::<BigEndian>()?;
                Instruction::Jsr(branch)
            }
            0xA9 => {
                let index = cursor.read_u8()?;
                Instruction::Ret(index)
            }
            0xAA => {
                let pos = cursor.seek(SeekFrom::Current(0))?;
                let offset = ((4 - (pos % 4)) % 4) as i64;
                let padding = offset as u32;
                cursor.seek(SeekFrom::Current(offset))?;

                let default = cursor.read_u32::<BigEndian>()?;
                let minimum = cursor.read_u32::<BigEndian>()?;
                let maximum = cursor.read_u32::<BigEndian>()?;

                let mut jump_targets = vec![];

                for _ in minimum..=maximum {
                    let jump_target = cursor.read_u32::<BigEndian>()?;
                    jump_targets.push(jump_target);
                }
                assert_eq!(jump_targets.len() as u32, maximum - minimum + 1);

                Instruction::TableSwitch {
                    padding,
                    minimum,
                    maximum,
                    jump_targets,
                    default,
                }
            }
            0xAB => {
                let pos = cursor.seek(SeekFrom::Current(0))?;
                let offset = ((4 - (pos % 4)) % 4) as i64;
                let padding = (offset as u64 - pos) as u32;
                cursor.seek(SeekFrom::Current(offset))?;

                let default = cursor.read_u32::<BigEndian>()?;
                let npairs = cursor.read_u32::<BigEndian>()?;

                let mut pairs = vec![];

                for _ in 0..npairs {
                    let value = cursor.read_u32::<BigEndian>()?;
                    let target = cursor.read_u32::<BigEndian>()?;

                    pairs.push(LookupSwitchPair { value, target });
                }

                Instruction::LookupSwitch {
                    padding,
                    default,
                    pairs,
                }
            }
            0xAC => Instruction::IReturn,
            0xAD => Instruction::LReturn,
            0xAE => Instruction::FReturn,
            0xAF => Instruction::DReturn,
            0xB0 => Instruction::AReturn,
            0xB1 => Instruction::Return,
            0xB2 => {
                let index = cursor.read_u16::<BigEndian>()?;
                Instruction::GetStatic(index)
            }
            0xB3 => {
                let index = cursor.read_u16::<BigEndian>()?;
                Instruction::PutStatic(index)
            }
            0xB4 => {
                let index = cursor.read_u16::<BigEndian>()?;
                Instruction::GetField(index)
            }
            0xB5 => {
                let index = cursor.read_u16::<BigEndian>()?;
                Instruction::PutField(index)
            }
            0xB6 => {
                let index = cursor.read_u16::<BigEndian>()?;
                Instruction::InvokeVirtual(index)
            }
            0xB7 => {
                let index = cursor.read_u16::<BigEndian>()?;
                Instruction::InvokeSpecial(index)
            }
            0xB8 => {
                let index = cursor.read_u16::<BigEndian>()?;
                Instruction::InvokeStatic(index)
            }
            0xB9 => {
                let index = cursor.read_u16::<BigEndian>()?;
                let count = cursor.read_u8()?;
                Instruction::InvokeInterface { index, count }
            }
            0xBA => {
                let index = cursor.read_u16::<BigEndian>()?;
                let zero = cursor.read_u16::<BigEndian>()?;
                assert_eq!(zero, 0);
                Instruction::InvokeDynamic(index)
            }
            0xBB => {
                let index = cursor.read_u16::<BigEndian>()?;
                Instruction::New(index)
            }
            0xBC => {
                let atype = cursor.read_u8()?;
                Instruction::NewArray(atype)
            }
            0xBD => {
                let index = cursor.read_u16::<BigEndian>()?;
                Instruction::ANewArray(index)
            }
            0xBE => Instruction::ArrayLength,
            0xBF => Instruction::AThrow,
            0xC0 => {
                let index = cursor.read_u16::<BigEndian>()?;
                Instruction::CheckCast(index)
            }
            0xC1 => {
                let index = cursor.read_u16::<BigEndian>()?;
                Instruction::InstanceOf(index)
            }
            0xC2 => Instruction::MonitorEnter,
            0xC3 => Instruction::MonitorExit,
            0xC4 => {
                let opcode = cursor.read_u8()?;
                let index = cursor.read_u16::<BigEndian>()?;

                match opcode {
                    0x15 => Instruction::ILoadW(index),
                    0x16 => Instruction::LLoadW(index),
                    0x17 => Instruction::FLoadW(index),
                    0x18 => Instruction::DLoadW(index),
                    0x19 => Instruction::ALoadW(index),
                    0x36 => Instruction::IStoreW(index),
                    0x37 => Instruction::LStoreW(index),
                    0x38 => Instruction::FStoreW(index),
                    0x39 => Instruction::DStoreW(index),
                    0x3A => Instruction::AStoreW(index),
                    0xA9 => Instruction::RetW(index),
                    0x84 => {
                        let count = cursor.read_u16::<BigEndian>()?;
                        Instruction::IIncW(index, count)
                    }
                    _ => unreachable!(),
                }
            }
            0xC5 => {
                let index = cursor.read_u16::<BigEndian>()?;
                let dimensions = cursor.read_u8()?;
                Instruction::MultiANewArray(index, dimensions)
            }
            0xC6 => {
                let index = cursor.read_u16::<BigEndian>()?;
                Instruction::IfNull(index)
            }
            0xC7 => {
                let index = cursor.read_u16::<BigEndian>()?;
                Instruction::IfNonNull(index)
            }
            0xC8 => {
                let branch = cursor.read_u32::<BigEndian>()?;
                Instruction::GotoW(branch)
            }
            0xC9 => {
                let branch = cursor.read_u32::<BigEndian>()?;
                Instruction::JsrW(branch)
            }
            _ => panic!("Invalid opcode: {opcode:#X}"),
        };

        instructions.push(inst);
    }

    Ok(instructions)
}
