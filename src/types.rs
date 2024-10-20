use std::any::Any;
use std::cell::Cell;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id {
    id: u64,
}

// Implement Deref and DerefMut for Id to allow us to deref Id to u64 and set
// the value of Id to a u64
impl std::ops::Deref for Id {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl std::ops::DerefMut for Id {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.id
    }
}

impl From<u64> for Id {
    fn from(id: u64) -> Self {
        Self { id }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TypeName {
    name: String,
}

impl TypeName {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

impl From<&str> for TypeName {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

pub trait Value: Any + std::fmt::Debug {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn typename(&self) -> TypeName;
}

pub struct Registry {
    variables: HashMap<Id, (TypeName, Box<dyn Value>)>,
    id: Id,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            id: 0.into(),
        }
    }
}

// Functions to register variables, functions, and classes
impl Registry {
    pub fn register_variable<T: Value>(&mut self, value: T) -> Id {
        let boxed_value: Box<dyn Value> = Box::new(value);
        self.variables
            .insert(self.id.clone(), (boxed_value.typename(), boxed_value));
        // Increment the id
        *(self.id) += 1;
        (*(self).id - 1).into()
    }
}

// Functions to get variables, functions, and classes
impl Registry {
    pub fn get_variable(&self, id: &Id) -> Option<&(TypeName, Box<dyn Value>)> {
        self.variables.get(id)
    }
}

// Functions to get mutable variables, functions, and classes
impl Registry {
    fn get_variable_mut(&mut self, id: Id) -> Option<&mut (TypeName, Box<dyn Value>)> {
        self.variables.get_mut(&id)
    }
}

// Functions to remove variables, functions, and classes
impl Registry {
    fn remove_variable(&mut self, id: Id) -> Option<(TypeName, Box<dyn Value>)> {
        self.variables.remove(&id)
    }
}

// Functions to set variables, functions, and classes
impl Registry {
    pub fn set_variable<T: Value>(&mut self, id: &Id, value: T) {
        self.variables
            .insert(id.clone(), (value.typename(), Box::new(value)));
    }
}

type Type = Box<dyn Fn(Vec<Box<dyn Value>>) -> Box<dyn Value>>;

pub struct Function {
    pub function: Type,
}

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Function {}()", std::any::type_name::<Self>())
    }
}

#[derive(Debug, Clone)]
pub struct ClassBase {
    typename: TypeName,
    functable: HashMap<String, Id>,
    field_names: Vec<String>,
    parent: Option<Id>, // Parent Class Base ID
}

impl ClassBase {
    pub fn new(typename: impl Into<TypeName>, field_names: Vec<String>) -> Self {
        Self {
            typename: typename.into(),
            field_names,
            functable: HashMap::new(),
            parent: None,
        }
    }
}

#[derive(Debug)]
pub struct ClassInstance {
    class_base: Id,
    variables: HashMap<String, Id>,
}

impl ClassInstance {
    // New function takes the class base id, values of the fields
    pub fn new(class_base: Id, values: Vec<Box<dyn Value>>, registry: &mut Registry) -> Self {
        // Register all the spaces for the fields but don't set the values
        let mut variables = HashMap::new();
        // use a while loop to iterate over the field names and values to set the values
        let mut i = 0;
        while i < values.len() {}
        Self {
            class_base,
            variables,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TslString {
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct TslNull;

impl TslString {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct TslInt {
    pub value: i128,
}

#[macro_export]
macro_rules! register_variable {
    ($registry:ident, $value:expr, $store:ident) => {
        let $store = $registry.register_variable($value)
    };
}

#[macro_export]
macro_rules! int {
    ($value:expr) => {
        $crate::TslInt { value: $value }
    };
}

#[macro_export]
macro_rules! string {
    ($value:expr) => {
        $crate::TslString {
            value: $value.to_string(),
        }
    };
}

pub trait TslType {
    fn typename() -> TypeName;
}

macro_rules! implement_many {
    ($($t:ty),*) => {
        $(impl Value for $t {
            fn as_any(&self) -> &dyn Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
            fn typename(&self) -> TypeName {
                <$t as TslType>::typename()
            }
        })*
    };
}

macro_rules! implement_names {
    //every group of 2 elements is a type and its name
    ($($t:ty, $name:expr),*) => {
        $(impl TslType for $t {
            fn typename() -> TypeName {
                $name.into()
            }
        })*
    };
}

implement_names!(
    TslInt,
    "Integer",
    TslString,
    "String",
    ClassBase,
    "ClassBase",
    ClassInstance,
    "ClassInstance",
    Function,
    "Function",
    Box<dyn Value>,
    "Value",
    TslNull,
    "Null"
);

implement_many!(
    TslInt,
    TslString,
    ClassBase,
    ClassInstance,
    Function,
    Box<dyn Value>,
    TslNull
);
