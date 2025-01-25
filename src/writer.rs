use byteorder::{BigEndian, WriteBytesExt};
use std::io::SeekFrom;
use std::io::{self, Seek, Write};

use crate::enums::{
    AccessFlag, Attribute, Constant, ElementValue, Instruction, StackMapFrameType, TargetInfo,
    VerificationType,
};
use crate::mapping::{
    CLASS_FLAGS, FIELD_FLAGS, INNER_CLASS_FLAGS, METHOD_FLAGS, METHOD_PARAMETER_FLAGS,
    MODULE_EXPORTS_FLAGS, MODULE_FLAGS, MODULE_OPENS_FLAGS, MODULE_REQUIRES_FLAGS,
};
use crate::structs::{
    Annotation, Field, Method, ModuleExports, ModuleOpens, ModuleProvides, ModuleRequires,
    TypeAnnotation,
};
use crate::JVMClass;

pub fn write_constant_pool<W: Write>(
    w: &mut W,
    constants: &Vec<Constant>,
) -> Result<(), io::Error> {
    w.write_u16::<BigEndian>(constants.len() as u16)?;

    for cnst in constants.iter() {
        match cnst {
            Constant::Utf8(string) => {
                w.write_u8(1)?;

                let bytes = string.as_bytes();
                w.write_u16::<BigEndian>(bytes.len() as u16)?;
                w.write(&bytes).unwrap();
            }
            Constant::Integer(value) => {
                w.write_u8(3)?;

                w.write_i32::<BigEndian>(*value)?;
            }
            Constant::Float(value) => {
                w.write_u8(4)?;

                w.write_f32::<BigEndian>(*value)?;
            }
            Constant::Long(value) => {
                w.write_u8(5)?;

                w.write_i64::<BigEndian>(*value)?;
            }
            Constant::Double(value) => {
                w.write_u8(6)?;

                w.write_f64::<BigEndian>(*value)?;
            }
            Constant::Class { name_index } => {
                w.write_u8(7)?;

                w.write_u16::<BigEndian>(*name_index)?;
            }
            Constant::String { string_index } => {
                w.write_u8(8)?;

                w.write_u16::<BigEndian>(*string_index)?;
            }
            Constant::Fieldref {
                class_index,
                name_and_type_index,
            } => {
                w.write_u8(9)?;

                w.write_u16::<BigEndian>(*class_index)?;
                w.write_u16::<BigEndian>(*name_and_type_index)?;
            }
            Constant::Methodref {
                class_index,
                name_and_type_index,
            } => {
                w.write_u8(10)?;

                w.write_u16::<BigEndian>(*class_index)?;
                w.write_u16::<BigEndian>(*name_and_type_index)?;
            }
            Constant::InterfaceMethodref {
                class_index,
                name_and_type_index,
            } => {
                w.write_u8(11)?;

                w.write_u16::<BigEndian>(*class_index)?;
                w.write_u16::<BigEndian>(*name_and_type_index)?;
            }
            Constant::NameAndType {
                name_index,
                descriptor_index,
            } => {
                w.write_u8(12)?;

                w.write_u16::<BigEndian>(*name_index)?;
                w.write_u16::<BigEndian>(*descriptor_index)?;
            }
            Constant::MethodHandle {
                reference_kind,
                reference_index,
            } => {
                w.write_u8(15)?;

                w.write_u8(*reference_kind)?;
                w.write_u16::<BigEndian>(*reference_index)?;
            }
            Constant::MethodType { descriptor_index } => {
                w.write_u8(16)?;

                w.write_u16::<BigEndian>(*descriptor_index)?;
            }
            Constant::Dynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => {
                w.write_u8(17)?;

                w.write_u16::<BigEndian>(*bootstrap_method_attr_index)?;
                w.write_u16::<BigEndian>(*name_and_type_index)?;
            }
            Constant::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => {
                w.write_u8(18)?;

                w.write_u16::<BigEndian>(*bootstrap_method_attr_index)?;
                w.write_u16::<BigEndian>(*name_and_type_index)?;
            }
            Constant::Module { name_index } => {
                w.write_u8(19)?;

                w.write_u16::<BigEndian>(*name_index)?;
            }
            Constant::Package { name_index } => {
                w.write_u8(20)?;

                w.write_u16::<BigEndian>(*name_index)?;
            }
            Constant::Invalid => {
                // skipped
            }
        }
    }

    Ok(())
}

pub fn write_attributes<W: Write + Seek>(
    w: &mut W,
    attributes: &Vec<Attribute>,
    jvm: &JVMClass,
) -> Result<(), io::Error> {
    w.write_u16::<BigEndian>(attributes.len() as u16)?;

    for attribute in attributes {
        let attr_start = w.seek(SeekFrom::Current(0))?;
        w.write_u16::<BigEndian>(0)?;
        w.write_u32::<BigEndian>(0)?;

        let attr_name = match attribute {
            Attribute::Code {
                code,
                max_stack,
                max_locals,
                attributes,
            } => {
                w.write_u16::<BigEndian>(*max_stack)?;
                w.write_u16::<BigEndian>(*max_locals)?;
                compile(w, code)?;
                w.write_u16::<BigEndian>(0)?;
                write_attributes(w, attributes, jvm)?;

                "Code"
            }
            Attribute::LineNumberTable(line_number_table) => {
                w.write_u16::<BigEndian>(line_number_table.len() as u16)?;

                for line in line_number_table {
                    w.write_u16::<BigEndian>(line.start_pc)?;
                    w.write_u16::<BigEndian>(line.line_number)?;
                }

                "LineNumberTable"
            }
            Attribute::StackMapTable(frames) => {
                w.write_u16::<BigEndian>(frames.len() as u16)?;

                for frame in frames {
                    match frame.frame_type {
                        StackMapFrameType::SameFrame(frame_type) => {
                            w.write_u8(frame_type)?;
                        }
                        StackMapFrameType::SameLocals1StackItemFrame(frame_type) => {
                            w.write_u8(frame_type)?;
                            write_verification_type(w, &frame.stack[0])?;
                        }
                        StackMapFrameType::SameLocals1StackItemFrameExtended => {
                            w.write_u8(247)?;
                            write_verification_type(w, &frame.stack[0])?;
                        }
                        StackMapFrameType::ChopFrame(frame_type) => {
                            w.write_u8(frame_type)?;
                            w.write_u16::<BigEndian>(frame.offset_delta)?;
                        }
                        StackMapFrameType::SameFrameExtended => {
                            w.write_u8(251)?;
                            w.write_u16::<BigEndian>(frame.offset_delta)?;
                        }
                        StackMapFrameType::AppendFrame(frame_type) => {
                            w.write_u8(frame_type)?;
                            w.write_u16::<BigEndian>(frame.offset_delta)?;
                            for verification_type in &frame.locals {
                                write_verification_type(w, verification_type)?;
                            }
                        }
                        StackMapFrameType::FullFrame => {
                            w.write_u8(255)?;
                            w.write_u16::<BigEndian>(frame.locals.len() as u16)?;
                            for verification_type in &frame.locals {
                                write_verification_type(w, verification_type)?;
                            }
                            w.write_u16::<BigEndian>(frame.stack.len() as u16)?;
                            for verification_type in &frame.stack {
                                write_verification_type(w, verification_type)?;
                            }
                        }
                    }
                }

                "StackMapTable"
            }
            Attribute::Exceptions(exceptions) => {
                w.write_u16::<BigEndian>(exceptions.len() as u16)?;

                for exception in exceptions {
                    w.write_u16::<BigEndian>(*exception)?;
                }

                "Exceptions"
            }
            Attribute::SourceFile { sourcefile_index } => {
                w.write_u16::<BigEndian>(*sourcefile_index)?;

                "SourceFile"
            }
            Attribute::BootstrapMethods(bootstrap_methods) => {
                w.write_u16::<BigEndian>(bootstrap_methods.len() as u16)?;

                for bootstrap_method in bootstrap_methods {
                    w.write_u16::<BigEndian>(bootstrap_method.bootstrap_method_ref)?;
                    w.write_u16::<BigEndian>(bootstrap_method.bootstrap_arguments.len() as u16)?;

                    for arg in &bootstrap_method.bootstrap_arguments {
                        w.write_u16::<BigEndian>(*arg)?;
                    }
                }

                "BootstrapMethods"
            }
            Attribute::InnerClasses(inner_classes) => {
                w.write_u16::<BigEndian>(inner_classes.len() as u16)?;

                for inner_class in inner_classes {
                    let inner_class_info_index = &inner_class.inner_class_info_index;
                    let outer_class_info_index = &inner_class.outer_class_info_index;
                    let inner_name_index = &inner_class.inner_name_index;
                    let inner_class_access_flags = &inner_class.inner_class_access_flags;
                    let inner_class_access_flags =
                        compact_inner_class_flags(inner_class_access_flags);

                    w.write_u16::<BigEndian>(*inner_class_info_index)?;
                    w.write_u16::<BigEndian>(*outer_class_info_index)?;
                    w.write_u16::<BigEndian>(*inner_name_index)?;
                    w.write_u16::<BigEndian>(inner_class_access_flags)?;
                }

                "InnerClasses"
            }
            Attribute::RuntimeVisibleAnnotations(annotations) => {
                write_annotations(w, annotations)?;

                "RuntimeVisibleAnnotations"
            }
            Attribute::RuntimeInvisibleAnnotations(annotations) => {
                write_annotations(w, annotations)?;

                "RuntimeInvisibleAnnotations"
            }
            Attribute::ConstantValue {
                constantvalue_index,
            } => {
                w.write_u16::<BigEndian>(*constantvalue_index)?;

                "ConstantValue"
            }
            Attribute::EnclosingMethod {
                class_index,
                method_index,
            } => {
                w.write_u16::<BigEndian>(*class_index)?;
                w.write_u16::<BigEndian>(*method_index)?;

                "EnclosingMethod"
            }
            Attribute::Synthetic => "Synthetic",
            Attribute::Signature { signature_index } => {
                w.write_u16::<BigEndian>(*signature_index)?;

                "Signature"
            }
            Attribute::SourceDebugExtension { debug_extension } => {
                w.write(&debug_extension)?;

                "SourceDebugExtension"
            }
            Attribute::Deprecated => "Deprecated",
            Attribute::ModuleMainClass(main_class_index) => {
                w.write_u16::<BigEndian>(*main_class_index)?;

                "ModuleMainClass"
            }
            Attribute::NestHost(host_class_index) => {
                w.write_u16::<BigEndian>(*host_class_index)?;

                "NestHost"
            }
            Attribute::LocalVariableTable(local_variable_table) => {
                w.write_u16::<BigEndian>(local_variable_table.len() as u16)?;

                for local_variable in local_variable_table {
                    w.write_u16::<BigEndian>(local_variable.start_pc)?;
                    w.write_u16::<BigEndian>(local_variable.length)?;
                    w.write_u16::<BigEndian>(local_variable.name_index)?;
                    w.write_u16::<BigEndian>(local_variable.descriptor_index)?;
                    w.write_u16::<BigEndian>(local_variable.index)?;
                }

                "LocalVariableTable"
            }
            Attribute::LocalVariableTypeTable(local_variable_type_table) => {
                w.write_u16::<BigEndian>(local_variable_type_table.len() as u16)?;

                for local_variable_type in local_variable_type_table {
                    w.write_u16::<BigEndian>(local_variable_type.start_pc)?;
                    w.write_u16::<BigEndian>(local_variable_type.length)?;
                    w.write_u16::<BigEndian>(local_variable_type.name_index)?;
                    w.write_u16::<BigEndian>(local_variable_type.signature_index)?;
                    w.write_u16::<BigEndian>(local_variable_type.index)?;
                }

                "LocalVariableTypeTable"
            }
            Attribute::RuntimeVisibleParameterAnnotations(parameters_annotations) => {
                w.write_u8(parameters_annotations.len() as u8)?;

                for parameters_annotation in parameters_annotations {
                    write_annotations(w, &parameters_annotation)?;
                }

                "RuntimeVisibleParameterAnnotations"
            }
            Attribute::RuntimeInvisibleParameterAnnotations(parameters_annotations) => {
                w.write_u8(parameters_annotations.len() as u8)?;

                for parameters_annotation in parameters_annotations {
                    write_annotations(w, &parameters_annotation)?;
                }

                "RuntimeInvisibleParameterAnnotations"
            }
            Attribute::AnnotationDefault(element_value) => {
                write_element_value(w, element_value)?;

                "AnnotationDefault"
            }
            Attribute::MethodParameters(parameters) => {
                w.write_u8(parameters.len() as u8)?;

                for parameter in parameters {
                    let access_flags = compact_method_parameter_flags(&parameter.access_flags);

                    w.write_u16::<BigEndian>(parameter.name_index)?;
                    w.write_u16::<BigEndian>(access_flags)?;
                }

                "MethodParameters"
            }
            Attribute::Module {
                module_name_index,
                module_flags,
                module_version_index,
                requires,
                exports,
                opens,
                uses,
                provides,
            } => {
                w.write_u16::<BigEndian>(*module_name_index)?;
                let module_flags = compact_module_flags(module_flags);
                w.write_u16::<BigEndian>(module_flags)?;
                w.write_u16::<BigEndian>(*module_version_index)?;
                write_module_requires(w, requires)?;
                write_module_exports(w, exports)?;
                write_module_opens(w, opens)?;

                for used in uses {
                    w.write_u16::<BigEndian>(*used)?;
                }

                write_module_provides(w, provides)?;

                "Module"
            }
            Attribute::ModulePackages(packages_index) => {
                w.write_u16::<BigEndian>(packages_index.len() as u16)?;

                for package_index in packages_index {
                    w.write_u16::<BigEndian>(*package_index)?;
                }

                "ModulePackages"
            }
            Attribute::NestMembers(classes) => {
                w.write_u16::<BigEndian>(classes.len() as u16)?;

                for class in classes {
                    w.write_u16::<BigEndian>(*class)?;
                }

                "NestMembers"
            }
            Attribute::PermittedSubclasses(classes) => {
                w.write_u16::<BigEndian>(classes.len() as u16)?;

                for class in classes {
                    w.write_u16::<BigEndian>(*class)?;
                }

                "PermittedSubclasses"
            }
            Attribute::Record(components) => {
                w.write_u16::<BigEndian>(components.len() as u16)?;

                for component in components {
                    w.write_u16::<BigEndian>(component.name_index)?;
                    w.write_u16::<BigEndian>(component.descriptor_index)?;
                    write_attributes(w, &component.attributes, jvm)?;
                }

                "Record"
            }
            Attribute::RuntimeInvisibleTypeAnnotations(annotations) => {
                w.write_u16::<BigEndian>(annotations.len() as u16)?;

                for annotation in annotations {
                    write_type_annotation(w, &annotation)?;
                }

                "RuntimeInvisibleTypeAnnotations"
            }
            Attribute::RuntimeVisibleTypeAnnotations(annotations) => {
                w.write_u16::<BigEndian>(annotations.len() as u16)?;

                for annotation in annotations {
                    write_type_annotation(w, &annotation)?;
                }

                "RuntimeVisibleTypeAnnotations"
            }
            Attribute::Unknown { name, data } => {
                w.write(&data)?;
                name
            }
        };

        let string_index = jvm.get_string_index(attr_name).unwrap();

        let attr_end = w.seek(SeekFrom::Current(0))?;
        let attr_len = attr_end - attr_start - 6;
        w.seek(SeekFrom::Start(attr_start))?;
        w.write_u16::<BigEndian>(string_index)?;
        w.write_u32::<BigEndian>(attr_len as u32)?;
        w.seek(SeekFrom::Start(attr_end))?;
    }

    Ok(())
}

pub fn write_fields<W: Write + Seek>(
    w: &mut W,
    fields: &Vec<Field>,
    jvm: &JVMClass,
) -> Result<(), io::Error> {
    w.write_u16::<BigEndian>(fields.len() as u16)?;

    for field in fields {
        let member_data = &field.0;
        let access_flags = compact_field_flags(&member_data.access_flags);
        w.write_u16::<BigEndian>(access_flags)?;
        w.write_u16::<BigEndian>(member_data.name)?;
        w.write_u16::<BigEndian>(member_data.descriptor)?;
        write_attributes(w, &member_data.attributes, jvm)?;
    }

    Ok(())
}

pub fn write_interfaces<W: Write>(w: &mut W, interfaces: &Vec<u16>) -> Result<(), io::Error> {
    w.write_u16::<BigEndian>(interfaces.len() as u16)?;

    for interface in interfaces {
        w.write_u16::<BigEndian>(*interface)?;
    }

    Ok(())
}

pub fn write_methods<W: Write + Seek>(
    w: &mut W,
    methods: &Vec<Method>,
    jvm: &JVMClass,
) -> Result<(), io::Error> {
    w.write_u16::<BigEndian>(methods.len() as u16)?;

    for method in methods {
        let member_data = &method.0;
        let access_flags = compact_method_flags(&member_data.access_flags);
        w.write_u16::<BigEndian>(access_flags)?;
        w.write_u16::<BigEndian>(member_data.name)?;
        w.write_u16::<BigEndian>(member_data.descriptor)?;
        write_attributes(w, &member_data.attributes, jvm)?;
    }

    Ok(())
}

pub fn compact_class_flags(flags: &Vec<AccessFlag>) -> u16 {
    compact_flags(flags, &CLASS_FLAGS)
}

fn compact_inner_class_flags(flags: &Vec<AccessFlag>) -> u16 {
    compact_flags(flags, &INNER_CLASS_FLAGS)
}

fn compact_field_flags(flags: &Vec<AccessFlag>) -> u16 {
    compact_flags(flags, &FIELD_FLAGS)
}

fn compact_method_flags(flags: &Vec<AccessFlag>) -> u16 {
    compact_flags(flags, &METHOD_FLAGS)
}

fn compact_method_parameter_flags(flags: &Vec<AccessFlag>) -> u16 {
    compact_flags(flags, &METHOD_PARAMETER_FLAGS)
}

fn compact_module_flags(flags: &Vec<AccessFlag>) -> u16 {
    compact_flags(flags, &MODULE_FLAGS)
}

fn compact_module_requires_flags(flags: &Vec<AccessFlag>) -> u16 {
    compact_flags(flags, &MODULE_REQUIRES_FLAGS)
}

fn compact_module_opens_flags(flags: &Vec<AccessFlag>) -> u16 {
    compact_flags(flags, &MODULE_OPENS_FLAGS)
}

fn compact_module_exports_flags(flags: &Vec<AccessFlag>) -> u16 {
    compact_flags(flags, &MODULE_EXPORTS_FLAGS)
}

fn compact_flags<T: Copy + std::cmp::PartialEq>(flags: &Vec<T>, mapping: &[(u16, T)]) -> u16 {
    mapping
        .iter()
        .filter(|(_, flag)| flags.contains(flag))
        .map(|(value, _)| *value)
        .sum()
}

fn write_verification_type<W: Write>(
    w: &mut W,
    verification_type: &VerificationType,
) -> Result<(), io::Error> {
    match verification_type {
        VerificationType::Top => w.write_u8(0)?,
        VerificationType::Integer => w.write_u8(1)?,
        VerificationType::Float => w.write_u8(2)?,
        VerificationType::Double => w.write_u8(3)?,
        VerificationType::Long => w.write_u8(4)?,
        VerificationType::Null => w.write_u8(5)?,
        VerificationType::UninitializedThis => w.write_u8(6)?,
        VerificationType::Object { cpool_index } => {
            w.write_u8(7)?;
            w.write_u16::<BigEndian>(*cpool_index)?;
        }
        VerificationType::Uninitialized { offset } => {
            w.write_u8(8)?;
            w.write_u16::<BigEndian>(*offset)?;
        }
    }

    Ok(())
}

fn write_annotations<W: Write + Seek>(
    w: &mut W,
    annotations: &Vec<Annotation>,
) -> Result<(), io::Error> {
    w.write_u16::<BigEndian>(annotations.len() as u16)?;

    for annotation in annotations {
        write_annotation(w, annotation)?;
    }

    Ok(())
}

fn write_type_annotation<W: Write + Seek>(
    w: &mut W,
    type_annotation: &TypeAnnotation,
) -> Result<(), io::Error> {
    write_target_info(w, &type_annotation.target_info)?;

    for path in &type_annotation.target_path {
        w.write_u8(path.type_path_kind)?;
        w.write_u8(path.type_argument_index)?;
    }

    write_annotation(w, &type_annotation.annotation)?;

    Ok(())
}

fn write_target_info<W: Write + Seek>(
    w: &mut W,
    target_info: &TargetInfo,
) -> Result<(), io::Error> {
    match target_info {
        TargetInfo::TypeParameter {
            target_type,
            type_parameter_index,
        } => {
            w.write_u8(*target_type)?;
            w.write_u8(*type_parameter_index)?;
        }
        TargetInfo::Supertype { supertype_index } => {
            w.write_u8(0x10)?;
            w.write_u16::<BigEndian>(*supertype_index)?;
        }
        TargetInfo::TypeParameterBound {
            target_type,
            type_parameter_index,
            bound_index,
        } => {
            w.write_u8(*target_type)?;
            w.write_u8(*type_parameter_index)?;
            w.write_u8(*bound_index)?;
        }
        TargetInfo::Empty(target_type) => {
            w.write_u8(*target_type)?;
        }
        TargetInfo::FormalParameter {
            formal_parameter_index,
        } => {
            w.write_u8(0x16)?;
            w.write_u8(*formal_parameter_index)?;
        }
        TargetInfo::Throws { throws_type_index } => {
            w.write_u16::<BigEndian>(*throws_type_index)?;
        }
        TargetInfo::Localvar { target_type, table } => {
            w.write_u8(*target_type)?;
            w.write_u16::<BigEndian>(table.len() as u16)?;

            for local_var in table {
                w.write_u16::<BigEndian>(local_var.start_pc)?;
                w.write_u16::<BigEndian>(local_var.length)?;
                w.write_u16::<BigEndian>(local_var.index)?;
            }
        }
        TargetInfo::Catch {
            exception_table_index,
        } => {
            w.write_u8(0x42)?;
            w.write_u16::<BigEndian>(*exception_table_index)?;
        }
        TargetInfo::Offset {
            target_type,
            offset,
        } => {
            w.write_u8(*target_type)?;
            w.write_u16::<BigEndian>(*offset)?;
        }
        TargetInfo::TypeArgument {
            target_type,
            offset,
            type_argument_index,
        } => {
            w.write_u8(*target_type)?;
            w.write_u16::<BigEndian>(*offset)?;
            w.write_u8(*type_argument_index)?;
        }
    }
    Ok(())
}

fn write_annotation<W: Write + Seek>(w: &mut W, annotation: &Annotation) -> Result<(), io::Error> {
    w.write_u16::<BigEndian>(annotation.type_index)?;
    w.write_u16::<BigEndian>(annotation.element_value_pairs.len() as u16)?;

    for pair in &annotation.element_value_pairs {
        w.write_u16::<BigEndian>(pair.element_name_index)?;
        write_element_value(w, &pair.value)?;
    }

    Ok(())
}

fn write_element_value<W: Write + Seek>(
    w: &mut W,
    element_value: &ElementValue,
) -> Result<(), io::Error> {
    match element_value {
        ElementValue::ConstValueIndex {
            tag,
            const_value_index,
        } => {
            w.write_u8(*tag)?;
            w.write_u16::<BigEndian>(*const_value_index)?;
        }
        ElementValue::ClassInfoIndex(class_info_index) => {
            w.write_u8(b'c')?;
            w.write_u16::<BigEndian>(*class_info_index)?;
        }
        ElementValue::EnumConstValue {
            type_name_index,
            const_name_index,
        } => {
            w.write_u8(b'e')?;
            w.write_u16::<BigEndian>(*type_name_index)?;
            w.write_u16::<BigEndian>(*const_name_index)?;
        }
        ElementValue::AnnotationValue(annotation) => {
            w.write_u8(b'@')?;
            write_annotation(w, annotation)?;
        }
        ElementValue::ArrayValue(values) => {
            w.write_u8(b'[')?;
            w.write_u16::<BigEndian>(values.len() as u16)?;

            for value in values {
                write_element_value(w, &value)?;
            }
        }
    }

    Ok(())
}

fn write_module_requires<W: Write + Seek>(
    w: &mut W,
    requires: &Vec<ModuleRequires>,
) -> Result<(), io::Error> {
    w.write_u16::<BigEndian>(requires.len() as u16)?;

    for require in requires {
        w.write_u16::<BigEndian>(require.requires_index)?;
        let requires_flags = compact_module_requires_flags(&require.requires_flags);
        w.write_u16::<BigEndian>(requires_flags)?;
        w.write_u16::<BigEndian>(require.requires_version_index)?;
    }

    Ok(())
}

fn write_module_exports<W: Write + Seek>(
    w: &mut W,
    exports: &Vec<ModuleExports>,
) -> Result<(), io::Error> {
    w.write_u16::<BigEndian>(exports.len() as u16)?;

    for export in exports {
        w.write_u16::<BigEndian>(export.exports_index)?;
        let exports_flags = compact_module_exports_flags(&export.exports_flags);
        w.write_u16::<BigEndian>(exports_flags)?;

        for export_to_index in &export.exports_to_index {
            w.write_u16::<BigEndian>(*export_to_index)?;
        }
    }

    Ok(())
}

fn write_module_opens<W: Write + Seek>(
    w: &mut W,
    opens: &Vec<ModuleOpens>,
) -> Result<(), io::Error> {
    w.write_u16::<BigEndian>(opens.len() as u16)?;

    for open in opens {
        w.write_u16::<BigEndian>(open.opens_index)?;
        let opens_flags = compact_module_opens_flags(&open.opens_flags);
        w.write_u16::<BigEndian>(opens_flags)?;

        for open_to_index in &open.opens_to_index {
            w.write_u16::<BigEndian>(*open_to_index)?;
        }
    }

    Ok(())
}

fn write_module_provides<W: Write + Seek>(
    w: &mut W,
    provides: &Vec<ModuleProvides>,
) -> Result<(), io::Error> {
    w.write_u16::<BigEndian>(provides.len() as u16)?;

    for provide in provides {
        w.write_u16::<BigEndian>(provide.provides_index)?;

        for provide_with_index in &provide.provides_with_index {
            w.write_u16::<BigEndian>(*provide_with_index)?;
        }
    }

    Ok(())
}

fn compile<W: Write + Seek>(w: &mut W, code: &Vec<Instruction>) -> Result<(), io::Error> {
    let code_start = w.seek(SeekFrom::Current(0))?;
    w.write_u32::<BigEndian>(0)?;

    for inst in code {
        match inst {
            Instruction::Nop => w.write_u8(0x00)?,
            Instruction::ANull => w.write_u8(0x01)?,
            Instruction::IConst(i) => match i {
                -1 => w.write_u8(0x02)?,
                0 => w.write_u8(0x03)?,
                1 => w.write_u8(0x04)?,
                2 => w.write_u8(0x05)?,
                3 => w.write_u8(0x06)?,
                4 => w.write_u8(0x07)?,
                5 => w.write_u8(0x08)?,
                _ => unreachable!(),
            },
            Instruction::LConst(l) => match l {
                0 => w.write_u8(0x09)?,
                1 => w.write_u8(0x0A)?,
                _ => unreachable!(),
            },
            Instruction::FConst(f) => {
                if *f == 0.0 {
                    w.write_u8(0x0B)?;
                } else if *f == 1.0 {
                    w.write_u8(0x0C)?;
                } else if *f == 2.0 {
                    w.write_u8(0x0D)?;
                } else {
                    unreachable!();
                }
            }
            Instruction::DConst(d) => {
                if *d == 0.0 {
                    w.write_u8(0x0E)?;
                } else if *d == 1.0 {
                    w.write_u8(0x0F)?;
                } else {
                    unreachable!();
                }
            }
            Instruction::Bipush(index) => {
                w.write_u8(0x10)?;
                w.write_u8(*index)?;
            }
            Instruction::Sipush(index) => {
                w.write_u8(0x11)?;
                w.write_i16::<BigEndian>(*index)?;
            }
            Instruction::Ldc(index) => {
                w.write_u8(0x12)?;
                w.write_u8(*index)?;
            }
            Instruction::LdcW(index) => {
                w.write_u8(0x13)?;
                w.write_u16::<BigEndian>(*index)?;
            }
            Instruction::Ldc2W(index) => {
                w.write_u8(0x14)?;
                w.write_u16::<BigEndian>(*index)?;
            }
            Instruction::ILoad(index) => match index {
                0 => w.write_u8(0x1A)?,
                1 => w.write_u8(0x1B)?,
                2 => w.write_u8(0x1C)?,
                3 => w.write_u8(0x1D)?,
                _ => {
                    w.write_u8(0x15)?;
                    w.write_u8(*index)?;
                }
            },
            Instruction::LLoad(index) => match index {
                0 => w.write_u8(0x1E)?,
                1 => w.write_u8(0x1F)?,
                2 => w.write_u8(0x20)?,
                3 => w.write_u8(0x21)?,
                _ => {
                    w.write_u8(0x16)?;
                    w.write_u8(*index)?;
                }
            },
            Instruction::FLoad(index) => match index {
                0 => w.write_u8(0x22)?,
                1 => w.write_u8(0x23)?,
                2 => w.write_u8(0x24)?,
                3 => w.write_u8(0x25)?,
                _ => {
                    w.write_u8(0x17)?;
                    w.write_u8(*index)?;
                }
            },
            Instruction::DLoad(index) => match index {
                0 => w.write_u8(0x26)?,
                1 => w.write_u8(0x27)?,
                2 => w.write_u8(0x28)?,
                3 => w.write_u8(0x29)?,
                _ => {
                    w.write_u8(0x18)?;
                    w.write_u8(*index)?;
                }
            },
            Instruction::ALoad(index) => match index {
                0 => w.write_u8(0x2A)?,
                1 => w.write_u8(0x2B)?,
                2 => w.write_u8(0x2C)?,
                3 => w.write_u8(0x2D)?,
                _ => {
                    w.write_u8(0x19)?;
                    w.write_u8(*index)?;
                }
            },
            Instruction::IALoad => w.write_u8(0x2E)?,
            Instruction::LALoad => w.write_u8(0x2F)?,
            Instruction::FALoad => w.write_u8(0x30)?,
            Instruction::DALoad => w.write_u8(0x31)?,
            Instruction::AALoad => w.write_u8(0x32)?,
            Instruction::BALoad => w.write_u8(0x33)?,
            Instruction::CALoad => w.write_u8(0x34)?,
            Instruction::SALoad => w.write_u8(0x35)?,
            Instruction::IStore(index) => match index {
                0 => w.write_u8(0x3B)?,
                1 => w.write_u8(0x3C)?,
                2 => w.write_u8(0x3D)?,
                3 => w.write_u8(0x3E)?,
                _ => {
                    w.write_u8(0x36)?;
                    w.write_u8(*index)?;
                }
            },
            Instruction::LStore(index) => match index {
                0 => w.write_u8(0x3F)?,
                1 => w.write_u8(0x40)?,
                2 => w.write_u8(0x41)?,
                3 => w.write_u8(0x42)?,
                _ => {
                    w.write_u8(0x37)?;
                    w.write_u8(*index)?;
                }
            },
            Instruction::FStore(index) => match index {
                0 => w.write_u8(0x43)?,
                1 => w.write_u8(0x44)?,
                2 => w.write_u8(0x45)?,
                3 => w.write_u8(0x46)?,
                _ => {
                    w.write_u8(0x38)?;
                    w.write_u8(*index)?;
                }
            },
            Instruction::DStore(index) => match index {
                0 => w.write_u8(0x47)?,
                1 => w.write_u8(0x48)?,
                2 => w.write_u8(0x49)?,
                3 => w.write_u8(0x4A)?,
                _ => {
                    w.write_u8(0x39)?;
                    w.write_u8(*index)?;
                }
            },
            Instruction::AStore(index) => match index {
                0 => w.write_u8(0x4B)?,
                1 => w.write_u8(0x4C)?,
                2 => w.write_u8(0x4D)?,
                3 => w.write_u8(0x4E)?,
                _ => {
                    w.write_u8(0x3A)?;
                    w.write_u8(*index)?;
                }
            },
            Instruction::IAStore => w.write_u8(0x4F)?,
            Instruction::LAStore => w.write_u8(0x50)?,
            Instruction::FAStore => w.write_u8(0x51)?,
            Instruction::DAStore => w.write_u8(0x52)?,
            Instruction::AAStore => w.write_u8(0x53)?,
            Instruction::BAStore => w.write_u8(0x54)?,
            Instruction::CAStore => w.write_u8(0x55)?,
            Instruction::SAStore => w.write_u8(0x56)?,
            Instruction::Pop => w.write_u8(0x57)?,
            Instruction::Pop2 => w.write_u8(0x58)?,
            Instruction::Dup => w.write_u8(0x59)?,
            Instruction::DupX1 => w.write_u8(0x5A)?,
            Instruction::DupX2 => w.write_u8(0x5B)?,
            Instruction::Dup2 => w.write_u8(0x5C)?,
            Instruction::Dup2X1 => w.write_u8(0x5D)?,
            Instruction::Dup2X2 => w.write_u8(0x5E)?,
            Instruction::Swap => w.write_u8(0x5F)?,
            Instruction::IAdd => w.write_u8(0x60)?,
            Instruction::LAdd => w.write_u8(0x61)?,
            Instruction::FAdd => w.write_u8(0x62)?,
            Instruction::DAdd => w.write_u8(0x63)?,
            Instruction::ISub => w.write_u8(0x64)?,
            Instruction::LSub => w.write_u8(0x65)?,
            Instruction::FSub => w.write_u8(0x66)?,
            Instruction::DSub => w.write_u8(0x67)?,
            Instruction::IMul => w.write_u8(0x68)?,
            Instruction::LMul => w.write_u8(0x69)?,
            Instruction::FMul => w.write_u8(0x6A)?,
            Instruction::DMul => w.write_u8(0x6B)?,
            Instruction::IDiv => w.write_u8(0x6C)?,
            Instruction::LDiv => w.write_u8(0x6D)?,
            Instruction::FDiv => w.write_u8(0x6E)?,
            Instruction::DDiv => w.write_u8(0x6F)?,
            Instruction::IRem => w.write_u8(0x70)?,
            Instruction::LRem => w.write_u8(0x71)?,
            Instruction::FRem => w.write_u8(0x72)?,
            Instruction::DRem => w.write_u8(0x73)?,
            Instruction::INeg => w.write_u8(0x74)?,
            Instruction::LNeg => w.write_u8(0x75)?,
            Instruction::FNeg => w.write_u8(0x76)?,
            Instruction::DNeg => w.write_u8(0x77)?,
            Instruction::IShl => w.write_u8(0x78)?,
            Instruction::LShl => w.write_u8(0x79)?,
            Instruction::IShr => w.write_u8(0x7A)?,
            Instruction::LShr => w.write_u8(0x7B)?,
            Instruction::IUShr => w.write_u8(0x7C)?,
            Instruction::LUShr => w.write_u8(0x7D)?,
            Instruction::IAnd => w.write_u8(0x7E)?,
            Instruction::LAnd => w.write_u8(0x7F)?,
            Instruction::IOr => w.write_u8(0x80)?,
            Instruction::LOr => w.write_u8(0x81)?,
            Instruction::IXor => w.write_u8(0x82)?,
            Instruction::LXor => w.write_u8(0x83)?,
            Instruction::IInc(index, count) => {
                w.write_u8(0x84)?;
                w.write_u8(*index)?;
                w.write_i8(*count)?;
            }
            Instruction::I2L => w.write_u8(0x85)?,
            Instruction::I2F => w.write_u8(0x86)?,
            Instruction::I2D => w.write_u8(0x87)?,
            Instruction::L2I => w.write_u8(0x88)?,
            Instruction::L2F => w.write_u8(0x89)?,
            Instruction::L2D => w.write_u8(0x8A)?,
            Instruction::F2I => w.write_u8(0x8B)?,
            Instruction::F2L => w.write_u8(0x8C)?,
            Instruction::F2D => w.write_u8(0x8D)?,
            Instruction::D2I => w.write_u8(0x8E)?,
            Instruction::D2L => w.write_u8(0x8F)?,
            Instruction::D2F => w.write_u8(0x90)?,
            Instruction::I2B => w.write_u8(0x91)?,
            Instruction::I2C => w.write_u8(0x92)?,
            Instruction::I2S => w.write_u8(0x93)?,
            Instruction::LCmp => w.write_u8(0x94)?,
            Instruction::FCmpl => w.write_u8(0x95)?,
            Instruction::FCmpg => w.write_u8(0x96)?,
            Instruction::DCmpl => w.write_u8(0x97)?,
            Instruction::DCmpg => w.write_u8(0x98)?,
            Instruction::Ifeq(branch) => {
                w.write_u8(0x99)?;
                w.write_i16::<BigEndian>(*branch)?;
            }
            Instruction::Ifne(branch) => {
                w.write_u8(0x9A)?;
                w.write_i16::<BigEndian>(*branch)?;
            }
            Instruction::Iflt(branch) => {
                w.write_u8(0x9B)?;
                w.write_i16::<BigEndian>(*branch)?;
            }
            Instruction::Ifge(branch) => {
                w.write_u8(0x9C)?;
                w.write_i16::<BigEndian>(*branch)?;
            }
            Instruction::Ifgt(branch) => {
                w.write_u8(0x9D)?;
                w.write_i16::<BigEndian>(*branch)?;
            }
            Instruction::Ifle(branch) => {
                w.write_u8(0x9E)?;
                w.write_i16::<BigEndian>(*branch)?;
            }
            Instruction::IfIcmpeq(branch) => {
                w.write_u8(0x9F)?;
                w.write_i16::<BigEndian>(*branch)?;
            }
            Instruction::IfIcmpne(branch) => {
                w.write_u8(0xA0)?;
                w.write_i16::<BigEndian>(*branch)?;
            }
            Instruction::IfIcmplt(branch) => {
                w.write_u8(0xA1)?;
                w.write_i16::<BigEndian>(*branch)?;
            }
            Instruction::IfIcmpge(branch) => {
                w.write_u8(0xA2)?;
                w.write_i16::<BigEndian>(*branch)?;
            }
            Instruction::IfIcmpgt(branch) => {
                w.write_u8(0xA3)?;
                w.write_i16::<BigEndian>(*branch)?;
            }
            Instruction::IfIcmple(branch) => {
                w.write_u8(0xA4)?;
                w.write_i16::<BigEndian>(*branch)?;
            }
            Instruction::IfAcmpeq(branch) => {
                w.write_u8(0xA5)?;
                w.write_i16::<BigEndian>(*branch)?;
            }
            Instruction::IfAcmpne(branch) => {
                w.write_u8(0xA6)?;
                w.write_i16::<BigEndian>(*branch)?;
            }
            Instruction::Goto(branch) => {
                w.write_u8(0xA7)?;
                w.write_i16::<BigEndian>(*branch)?;
            }
            Instruction::Jsr(branch) => {
                w.write_u8(0xA8)?;
                w.write_i16::<BigEndian>(*branch)?;
            }
            Instruction::TableSwitch {
                padding,
                minimum,
                maximum,
                jump_targets,
                default,
            } => {
                w.write_u8(0xA9)?;

                for _ in 0..*padding {
                    w.write_u8(0)?;
                }

                w.write_u32::<BigEndian>(*default)?;
                w.write_u32::<BigEndian>(*minimum)?;
                w.write_u32::<BigEndian>(*maximum)?;

                for jump_target in jump_targets {
                    w.write_u32::<BigEndian>(*jump_target)?;
                }
            }
            Instruction::LookupSwitch { padding, default, pairs } => {
                w.write_u8(0xAB)?;

                for _ in 0..*padding {
                    w.write_u8(0)?;
                }

                w.write_u32::<BigEndian>(*default)?;
                w.write_u32::<BigEndian>(pairs.len() as u32)?;

                for pair in pairs {
                    w.write_u32::<BigEndian>(pair.value)?;
                    w.write_u32::<BigEndian>(pair.target)?;
                }
            }
            Instruction::IReturn => w.write_u8(0xAC)?,
            Instruction::LReturn => w.write_u8(0xAD)?,
            Instruction::FReturn => w.write_u8(0xAE)?,
            Instruction::DReturn => w.write_u8(0xAF)?,
            Instruction::AReturn => w.write_u8(0xB0)?,
            Instruction::Return => w.write_u8(0xB1)?,
            Instruction::GetStatic(index) => {
                w.write_u8(0xB2)?;
                w.write_u16::<BigEndian>(*index)?;
            }
            Instruction::PutStatic(index) => {
                w.write_u8(0xB3)?;
                w.write_u16::<BigEndian>(*index)?;
            }
            Instruction::GetField(index) => {
                w.write_u8(0xB4)?;
                w.write_u16::<BigEndian>(*index)?;
            }
            Instruction::PutField(index) => {
                w.write_u8(0xB5)?;
                w.write_u16::<BigEndian>(*index)?;
            }
            Instruction::InvokeVirtual(index) => {
                w.write_u8(0xB6)?;
                w.write_u16::<BigEndian>(*index)?;
            }
            Instruction::InvokeSpecial(index) => {
                w.write_u8(0xB7)?;
                w.write_u16::<BigEndian>(*index)?;
            }
            Instruction::InvokeStatic(index) => {
                w.write_u8(0xB8)?;
                w.write_u16::<BigEndian>(*index)?;
            }
            Instruction::InvokeInterface { index, count } => {
                w.write_u8(0xB9)?;
                w.write_u16::<BigEndian>(*index)?;
                w.write_u8(*count)?;
            }
            Instruction::InvokeDynamic(index) => {
                w.write_u8(0xBA)?;
                w.write_u16::<BigEndian>(*index)?;
                w.write_u16::<BigEndian>(0)?;
            }
            Instruction::New(index) => {
                w.write_u8(0xBB)?;
                w.write_u16::<BigEndian>(*index)?;
            }
            Instruction::NewArray(atype) => {
                w.write_u8(0xBC)?;
                w.write_u8(*atype)?;
            }
            Instruction::ANewArray(index) => {
                w.write_u8(0xBD)?;
                w.write_u16::<BigEndian>(*index)?;
            }
            Instruction::ArrayLength => w.write_u8(0xBE)?,
            Instruction::AThrow => w.write_u8(0xBF)?,
            Instruction::CheckCast(index) => {
                w.write_u8(0xC0)?;
                w.write_u16::<BigEndian>(*index)?;
            }
            Instruction::InstanceOf(index) => {
                w.write_u8(0xC1)?;
                w.write_u16::<BigEndian>(*index)?;
            }
            Instruction::MonitorEnter => w.write_u8(0xC2)?,
            Instruction::MonitorExit => w.write_u8(0xC3)?,
            Instruction::MultiANewArray(index, dimensions) => {
                w.write_u8(0xC5)?;
                w.write_u16::<BigEndian>(*index)?;
                w.write_u8(*dimensions)?;
            }
            Instruction::IfNull(index) => {
                w.write_u8(0xC6)?;
                w.write_u16::<BigEndian>(*index)?;
            }
            Instruction::IfNonNull(index) => {
                w.write_u8(0xC7)?;
                w.write_u16::<BigEndian>(*index)?;
            }
            Instruction::GotoW(branch) => {
                w.write_u8(0xC8)?;
                w.write_u32::<BigEndian>(*branch)?;
            }
            Instruction::JsrW(branch) => {
                w.write_u8(0xC9)?;
                w.write_u32::<BigEndian>(*branch)?;
            }
            _ => { // 0xC4
                w.write_u8(0xC9)?;

                let (opcode, index, count) = match inst {
                    Instruction::ILoadW(index) => (0x15, *index, None),
                    Instruction::LLoadW(index) => (0x16, *index, None),
                    Instruction::FLoadW(index) => (0x17, *index, None),
                    Instruction::DLoadW(index) => (0x18, *index, None),
                    Instruction::ALoadW(index) => (0x19, *index, None),
                    Instruction::IStoreW(index) => (0x36, *index, None),
                    Instruction::LStoreW(index) => (0x37, *index, None),
                    Instruction::FStoreW(index) => (0x38, *index, None),
                    Instruction::DStoreW(index) => (0x39, *index, None),
                    Instruction::AStoreW(index) => (0x3A, *index, None),
                    Instruction::RetW(index) => (0xA9, *index, None),
                    Instruction::IIncW(index, count) => (0x84, *index, Some(*count)),
                    _ => unreachable!(),
                };

                w.write_u8(opcode)?;
                w.write_u16::<BigEndian>(index)?;

                if let Some(count) = count {
                    w.write_u16::<BigEndian>(count)?;
                }
            },
        }
    }

    let code_end = w.seek(SeekFrom::Current(0))?;
    w.seek(SeekFrom::Start(code_start))?;
    let code_len = code_end - code_start - 4;
    w.write_u32::<BigEndian>(code_len as u32)?;
    w.seek(SeekFrom::Start(code_end))?;

    Ok(())
}
