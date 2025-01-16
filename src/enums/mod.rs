use crate::structs::{
    Annotation, BootstrapMethod, InnerClass, LineNumber, LocalVar, LocalVariable,
    LocalVariableType, MethodParameter, ModuleExports, ModuleOpens, ModuleProvides, ModuleRequires,
    RecordComponent, StackMapFrame, TypeAnnotation,
};

mod instructions;
pub use instructions::Instruction;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccessFlag {
    /// - class: Declared abstract; must not be instantiated.
    /// - inner class: Marked or implicitly abstract in source.
    /// - method: Declared abstract; no implementation is provided.
    Abstract,
    /// - class: Declared as an annotation type.
    /// - inner class: Declared as an annotation type.
    Annotation,
    /// - method: A bridge method, generated by the compiler.
    Bridge,
    /// - class: Declared as an enum type.
    /// - inner class: Declared as an enum type.
    /// - field: Declared as an element of an enum.
    Enum,
    /// - class: Declared final; no subclasses allowed.
    /// - inner class: Marked final in source.
    /// - field: Declared final; never directly assigned to after object construction.
    /// - method: Declared final; must not be overridden
    /// - formal parameter: Indicates that the formal parameter was declared final.
    Final,
    /// - class: Is an interface, not a class.
    /// - inner class: Was an interface in source.
    Interface,
    /// - formal parameter: Indicates that the formal parameter was implicitly declared in source code, according to the specification of the language in which the source code was written.
    /// - module: Indicates that this module was implicitly declared.
    /// - module requires/exports/opens flag: Indicates that this dependence was implicitly declared in the source of the module declaration.
    Mandated,
    /// - class: Is a module, not a class or interface.
    Module,
    /// - method: Declared native; implemented in a language other than Java.
    Native,
    /// - module: Indicates that this module is open.
    Open,
    /// - class: Declared public; may be accessed from outside its package.
    /// - inner class: Marked or implicitly public in source.
    /// - field, method: Declared public; may be accessed from outside its package.
    Public,
    /// - inner class: Marked private in source.
    /// - field, method: Declared private; accessible only within the defining class.
    Private,
    /// - inner class: Marked protected in source.
    /// - field, method: Declared protected; may be accessed within subclasses.
    Protected,
    /// - inner class: Marked or implicitly static in source.
    /// - field, method: Declared static.
    Static,
    /// - module requires flag: Indicates that this dependence is mandatory in the static phase, i.e., at compile time, but is optional in the dynamic phase, i.e., at run time.
    StaticPhase,
    /// - method: Declared strictfp; floating-point mode is FP-strict.
    Strict,
    /// - class: Treat superclass methods specially when invoked by the invokespecial instruction.
    Super,
    /// - method: Declared synchronized; invocation is wrapped by a monitor use.
    Synchronized,
    /// Declared synthetic; not present in the source code.
    Synthetic,
    /// - field: Declared transient; not written or read by a persistent object manager.
    Transient,
    /// - module requires flag: Indicates that any module which depends on the current module, implicitly declares a dependence on the module indicated by this entry.
    Transitive,
    /// - method: Declared with variable number of arguments.
    VarArgs,
    /// - field: Declared volatile; cannot be cached.
    Volatile,
}

#[derive(Debug)]
pub enum Constant {
    Class {
        name_index: u16,
    },
    Double(f64),
    Dynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    Fieldref {
        class_index: u16,
        name_and_type_index: u16,
    },
    Float(f32),
    Integer(i32),
    InterfaceMethodref {
        class_index: u16,
        name_and_type_index: u16,
    },
    /// Used for the first entry in the constant pool, and the second entry of doubles and longs.
    Invalid,
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    Long(i64),
    MethodHandle {
        reference_kind: u8,
        reference_index: u16,
    },
    Methodref {
        class_index: u16,
        name_and_type_index: u16,
    },
    MethodType {
        descriptor_index: u16,
    },
    Module {
        name_index: u16,
    },
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
    Package {
        name_index: u16,
    },
    String {
        string_index: u16,
    },
    /// ⚠️ It is using Rust's String type and not the JVM's modified UTF-8. If you have a string that makes that crate panic, open an issue.
    Utf8(String),
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Constant::Class { name_index } => write!(f, "Constant::Class #{name_index}"),
            Constant::Double(double) => write!(f, "Constant::Double {double}"),
            Constant::Dynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => write!(
                f,
                "Constant::Dynamic #{bootstrap_method_attr_index}, #{name_and_type_index}"
            ),
            Constant::Fieldref {
                class_index,
                name_and_type_index,
            } => write!(
                f,
                "Constant::Fieldref #{class_index}, #{name_and_type_index}"
            ),
            Constant::Float(float) => write!(f, "Constant::Float {float}"),
            Constant::Integer(int) => write!(f, "Constant::Integer {int}"),
            Constant::InterfaceMethodref {
                class_index,
                name_and_type_index,
            } => write!(
                f,
                "Constant::InterfaceMethodref #{class_index}, #{name_and_type_index}"
            ),
            Constant::Invalid => write!(f, "Constant::Invalid"),
            Constant::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => write!(
                f,
                "Constant::InvokeDynamic #{bootstrap_method_attr_index}, #{name_and_type_index}"
            ),
            Constant::Long(long) => write!(f, "Constant::Long {long}"),
            Constant::MethodHandle {
                reference_kind,
                reference_index,
            } => write!(
                f,
                "Constant::MethodHandle #{reference_kind}, #{reference_index}"
            ),
            Constant::Methodref {
                class_index,
                name_and_type_index,
            } => write!(
                f,
                "Constant::Methodref #{class_index}, #{name_and_type_index}"
            ),
            Constant::MethodType { descriptor_index } => {
                write!(f, "Constant::MethodType #{descriptor_index}")
            }
            Constant::Module { name_index } => write!(f, "Constant::Module #{name_index}"),
            Constant::NameAndType {
                name_index,
                descriptor_index,
            } => write!(
                f,
                "Constant::NameAndType #{name_index}, #{descriptor_index}"
            ),
            Constant::Package { name_index } => write!(f, "Constant::Package #{name_index}"),
            Constant::String { string_index } => write!(f, "Constant::String #{string_index}"),
            Constant::Utf8(s) => write!(f, "Constant::Utf8({s})"),
        }
    }
}

#[derive(Debug)]
pub enum Attribute {
    AnnotationDefault(ElementValue),
    BootstrapMethods(Vec<BootstrapMethod>),
    Code {
        code: Vec<Instruction>,
        max_stack: u16,
        max_locals: u16,
        attributes: Vec<Attribute>,
    },
    ConstantValue {
        constantvalue_index: u16,
    },
    Deprecated,
    EnclosingMethod {
        class_index: u16,
        method_index: u16,
    },
    Exceptions(Vec<u16>),
    InnerClasses(Vec<InnerClass>),
    LineNumberTable(Vec<LineNumber>),
    LocalVariableTable(Vec<LocalVariable>),
    LocalVariableTypeTable(Vec<LocalVariableType>),
    MethodParameters(Vec<MethodParameter>),
    Module {
        module_name_index: u16,
        module_flags: Vec<AccessFlag>,
        module_version_index: u16,
        requires: Vec<ModuleRequires>,
        exports: Vec<ModuleExports>,
        opens: Vec<ModuleOpens>,
        uses: Vec<u16>,
        provides: Vec<ModuleProvides>,
    },
    ModuleMainClass(u16),
    ModulePackages(Vec<u16>),
    NestHost(u16),
    NestMembers(Vec<u16>),
    PermittedSubclasses(Vec<u16>),
    Record(Vec<RecordComponent>),
    RuntimeInvisibleAnnotations(Vec<Annotation>),
    RuntimeInvisibleParameterAnnotations(Vec<Vec<Annotation>>),
    RuntimeInvisibleTypeAnnotations(Vec<TypeAnnotation>),
    RuntimeVisibleAnnotations(Vec<Annotation>),
    RuntimeVisibleParameterAnnotations(Vec<Vec<Annotation>>),
    RuntimeVisibleTypeAnnotations(Vec<TypeAnnotation>),
    Signature {
        signature_index: u16,
    },
    SourceDebugExtension {
        debug_extension: Vec<u8>,
    },
    SourceFile {
        sourcefile_index: u16,
    },
    StackMapTable(Vec<StackMapFrame>),
    Synthetic,
    Unknown {
        name: String,
        data: Vec<u8>,
    },
}

#[derive(Debug)]
pub enum StackMapFrameType {
    AppendFrame(u8),
    ChopFrame(u8),
    FullFrame,
    SameFrame(u8),
    SameFrameExtended,
    SameLocals1StackItemFrame(u8),
    SameLocals1StackItemFrameExtended,
}

#[derive(Debug)]
pub enum VerificationType {
    Double,
    Float,
    Integer,
    Long,
    Null,
    Object { cpool_index: u16 },
    Top,
    Uninitialized { offset: u16 },
    UninitializedThis,
}

#[derive(Debug)]
pub enum ElementValue {
    AnnotationValue(Annotation),
    ArrayValue(Vec<ElementValue>),
    ClassInfoIndex(u16),
    ConstValueIndex {
        tag: u8,
        const_value_index: u16,
    },
    EnumConstValue {
        type_name_index: u16,
        const_name_index: u16,
    },
}

#[derive(Debug)]
pub enum TargetInfo {
    TypeParameter {
        target_type: u8,
        type_parameter_index: u8,
    },
    Supertype {
        supertype_index: u16,
    },
    TypeParameterBound {
        target_type: u8,
        type_parameter_index: u8,
        bound_index: u8,
    },
    Empty(u8),
    FormalParameter {
        formal_parameter_index: u8,
    },
    Throws {
        throws_type_index: u16,
    },
    Localvar {
        target_type: u8,
        table: Vec<LocalVar>,
    },
    Catch {
        exception_table_index: u16,
    },
    Offset {
        target_type: u8,
        offset: u16,
    },
    TypeArgument {
        target_type: u8,
        offset: u16,
        type_argument_index: u8,
    },
}
