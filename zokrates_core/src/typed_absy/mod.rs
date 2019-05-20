//! Module containing structs and enums to represent a program.
//!
//! @file absy.rs
//! @author Dennis Kuhnert <dennis.kuhnert@campus.tu-berlin.de>
//! @author Jacob Eberhardt <jacob.eberhardt@tu-berlin.de>
//! @date 2017

pub mod folder;
mod parameter;
mod variable;

pub use crate::typed_absy::parameter::Parameter;
pub use crate::typed_absy::variable::Variable;
use crate::types::{FunctionKey, Signature};
use std::collections::HashMap;

use crate::flat_absy::*;
use crate::imports::Import;
use crate::types::Type;
use std::fmt;
use zokrates_field::field::Field;

pub use self::folder::Folder;

type Identifier = String;

pub type TypedModuleId = String;

pub type TypedModules<T> = HashMap<TypedModuleId, TypedModule<T>>;

pub type TypedFunctionSymbols<T> = HashMap<FunctionKey, TypedFunctionSymbol<T>>;

#[derive(PartialEq, Debug)]
pub struct TypedProgram<T: Field> {
    pub modules: TypedModules<T>,
    pub main: TypedModule<T>,
}

impl<T: Field> fmt::Display for TypedProgram<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "| main: |")?;
        writeln!(f, "{}", "-".repeat(100))?;
        writeln!(f, "{}", self.main)?;
        writeln!(f, "{}", "-".repeat(100))?;
        writeln!(f, "")?;
        for (module_id, module) in &self.modules {
            writeln!(f, "| {}: |", module_id)?;
            writeln!(f, "{}", "-".repeat(100))?;
            writeln!(f, "{}", module)?;
            writeln!(f, "{}", "-".repeat(100))?;
            writeln!(f, "")?;
        }
        write!(f, "")
    }
}

#[derive(PartialEq, Clone)]
pub struct TypedModule<T: Field> {
    /// Functions of the program
    pub functions: TypedFunctionSymbols<T>,
    pub imports: Vec<Import>,
    pub imported_functions: Vec<FlatFunction<T>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypedFunctionSymbol<T: Field> {
    Here(TypedFunction<T>),
    There(FunctionKey, TypedModuleId),
}

impl<T: Field> TypedFunctionSymbol<T> {
    pub fn signature(&self, modules: &TypedModules<T>) -> Signature {
        match self {
            TypedFunctionSymbol::Here(f) => f.signature.clone(),
            TypedFunctionSymbol::There(key, module_id) => modules
                .get(module_id)
                .unwrap()
                .functions
                .get(key)
                .unwrap()
                .signature(&modules),
        }
    }
}

impl<T: Field> fmt::Display for TypedFunctionSymbol<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TypedFunctionSymbol::Here(fun) => write!(f, "{}", fun),
            TypedFunctionSymbol::There(key, module_id) => write!(
                f,
                "import {} from {} // with signature {}",
                key.id, module_id, key.signature
            ),
        }
    }
}

impl<T: Field> fmt::Display for TypedModule<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut res = vec![];
        res.extend(
            self.imports
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<_>>(),
        );
        res.extend(
            self.imported_functions
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<_>>(),
        );
        res.extend(
            self.functions
                .values()
                .map(|x| format!("{}", x))
                .collect::<Vec<_>>(),
        );
        write!(f, "{}", res.join("\n"))
    }
}

impl<T: Field> fmt::Debug for TypedModule<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "program(\n\timports:\n\t\t{}\n\tfunctions:\n\t\t{}{}\n)",
            self.imports
                .iter()
                .map(|x| format!("{:?}", x))
                .collect::<Vec<_>>()
                .join("\n\t\t"),
            self.imported_functions
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<_>>()
                .join("\n\t\t"),
            self.functions
                .iter()
                .map(|x| format!("{:?}", x))
                .collect::<Vec<_>>()
                .join("\n\t\t")
        )
    }
}

#[derive(Clone, PartialEq)]
pub struct TypedFunction<T: Field> {
    /// Name of the program
    pub id: Identifier,
    /// Arguments of the function
    pub arguments: Vec<Parameter>,
    /// Vector of statements that are executed when running the function
    pub statements: Vec<TypedStatement<T>>,
    /// function signature
    pub signature: Signature,
}

impl<T: Field> fmt::Display for TypedFunction<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "def {}({}) -> ({}):\n{}",
            self.id,
            self.arguments
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<_>>()
                .join(", "),
            self.signature
                .outputs
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<_>>()
                .join(", "),
            self.statements
                .iter()
                .map(|x| format!("\t{}", x))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

impl<T: Field> fmt::Debug for TypedFunction<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TypedFunction(id: {:?}, arguments: {:?}, ...):\n{}",
            self.id,
            self.arguments,
            self.statements
                .iter()
                .map(|x| format!("\t{:?}", x))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[derive(Clone, PartialEq, Hash, Eq)]
pub enum TypedAssignee<T: Field> {
    Identifier(Variable),
    ArrayElement(Box<TypedAssignee<T>>, Box<FieldElementExpression<T>>),
}

impl<T: Field> Typed for TypedAssignee<T> {
    fn get_type(&self) -> Type {
        match *self {
            TypedAssignee::Identifier(ref v) => v.get_type(),
            TypedAssignee::ArrayElement(ref a, _) => {
                let a_type = a.get_type();
                match a_type {
                    Type::FieldElementArray(_) => Type::FieldElement,
                    _ => panic!("array element has to take array"),
                }
            }
        }
    }
}

impl<T: Field> fmt::Debug for TypedAssignee<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TypedAssignee::Identifier(ref s) => write!(f, "{}", s.id),
            TypedAssignee::ArrayElement(ref a, ref e) => write!(f, "{}[{}]", a, e),
        }
    }
}

impl<T: Field> fmt::Display for TypedAssignee<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, PartialEq)]
pub enum TypedStatement<T: Field> {
    Return(Vec<TypedExpression<T>>),
    Definition(TypedAssignee<T>, TypedExpression<T>),
    Declaration(Variable),
    Condition(TypedExpression<T>, TypedExpression<T>),
    For(Variable, T, T, Vec<TypedStatement<T>>),
    MultipleDefinition(Vec<Variable>, TypedExpressionList<T>),
}

impl<T: Field> fmt::Debug for TypedStatement<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TypedStatement::Return(ref exprs) => {
                r#try!(write!(f, "Return("));
                for (i, expr) in exprs.iter().enumerate() {
                    r#try!(write!(f, "{}", expr));
                    if i < exprs.len() - 1 {
                        r#try!(write!(f, ", "));
                    }
                }
                write!(f, ")")
            }
            TypedStatement::Declaration(ref var) => write!(f, "Declaration({:?})", var),
            TypedStatement::Definition(ref lhs, ref rhs) => {
                write!(f, "Definition({:?}, {:?})", lhs, rhs)
            }
            TypedStatement::Condition(ref lhs, ref rhs) => {
                write!(f, "Condition({:?}, {:?})", lhs, rhs)
            }
            TypedStatement::For(ref var, ref start, ref stop, ref list) => {
                r#try!(write!(f, "for {:?} in {:?}..{:?} do\n", var, start, stop));
                for l in list {
                    r#try!(write!(f, "\t\t{:?}\n", l));
                }
                write!(f, "\tendfor")
            }
            TypedStatement::MultipleDefinition(ref lhs, ref rhs) => {
                write!(f, "MultipleDefinition({:?}, {:?})", lhs, rhs)
            }
        }
    }
}

impl<T: Field> fmt::Display for TypedStatement<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TypedStatement::Return(ref exprs) => {
                r#try!(write!(f, "return "));
                for (i, expr) in exprs.iter().enumerate() {
                    r#try!(write!(f, "{}", expr));
                    if i < exprs.len() - 1 {
                        r#try!(write!(f, ", "));
                    }
                }
                write!(f, "")
            }
            TypedStatement::Declaration(ref var) => write!(f, "{}", var),
            TypedStatement::Definition(ref lhs, ref rhs) => write!(f, "{} = {}", lhs, rhs),
            TypedStatement::Condition(ref lhs, ref rhs) => write!(f, "{} == {}", lhs, rhs),
            TypedStatement::For(ref var, ref start, ref stop, ref list) => {
                r#try!(write!(f, "for {} in {}..{} do\n", var, start, stop));
                for l in list {
                    r#try!(write!(f, "\t\t{}\n", l));
                }
                write!(f, "\tendfor")
            }
            TypedStatement::MultipleDefinition(ref ids, ref rhs) => {
                for (i, id) in ids.iter().enumerate() {
                    r#try!(write!(f, "{}", id));
                    if i < ids.len() - 1 {
                        r#try!(write!(f, ", "));
                    }
                }
                write!(f, " = {}", rhs)
            }
        }
    }
}

pub trait Typed {
    fn get_type(&self) -> Type;
}

#[derive(Clone, PartialEq, Hash, Eq)]
pub enum TypedExpression<T: Field> {
    Boolean(BooleanExpression<T>),
    FieldElement(FieldElementExpression<T>),
    FieldElementArray(FieldElementArrayExpression<T>),
}

impl<T: Field> From<BooleanExpression<T>> for TypedExpression<T> {
    fn from(e: BooleanExpression<T>) -> TypedExpression<T> {
        TypedExpression::Boolean(e)
    }
}

impl<T: Field> From<FieldElementExpression<T>> for TypedExpression<T> {
    fn from(e: FieldElementExpression<T>) -> TypedExpression<T> {
        TypedExpression::FieldElement(e)
    }
}

impl<T: Field> From<FieldElementArrayExpression<T>> for TypedExpression<T> {
    fn from(e: FieldElementArrayExpression<T>) -> TypedExpression<T> {
        TypedExpression::FieldElementArray(e)
    }
}

impl<T: Field> fmt::Display for TypedExpression<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TypedExpression::Boolean(ref e) => write!(f, "{}", e),
            TypedExpression::FieldElement(ref e) => write!(f, "{}", e),
            TypedExpression::FieldElementArray(ref e) => write!(f, "{}", e),
        }
    }
}

impl<T: Field> fmt::Debug for TypedExpression<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TypedExpression::Boolean(ref e) => write!(f, "{:?}", e),
            TypedExpression::FieldElement(ref e) => write!(f, "{:?}", e),
            TypedExpression::FieldElementArray(ref e) => write!(f, "{:?}", e),
        }
    }
}

impl<T: Field> Typed for TypedExpression<T> {
    fn get_type(&self) -> Type {
        match *self {
            TypedExpression::Boolean(_) => Type::Boolean,
            TypedExpression::FieldElement(_) => Type::FieldElement,
            TypedExpression::FieldElementArray(ref e) => e.get_type(),
        }
    }
}

impl<T: Field> Typed for FieldElementArrayExpression<T> {
    fn get_type(&self) -> Type {
        match *self {
            FieldElementArrayExpression::Identifier(n, _) => Type::FieldElementArray(n),
            FieldElementArrayExpression::Value(n, _) => Type::FieldElementArray(n),
            FieldElementArrayExpression::FunctionCall(n, _, _) => Type::FieldElementArray(n),
            FieldElementArrayExpression::IfElse(_, ref consequence, _) => consequence.get_type(),
        }
    }
}

pub trait MultiTyped {
    fn get_types(&self) -> &Vec<Type>;
}

#[derive(Clone, PartialEq)]
pub enum TypedExpressionList<T: Field> {
    FunctionCall(FunctionKey, Vec<TypedExpression<T>>, Vec<Type>),
}

impl<T: Field> MultiTyped for TypedExpressionList<T> {
    fn get_types(&self) -> &Vec<Type> {
        match *self {
            TypedExpressionList::FunctionCall(_, _, ref types) => types,
        }
    }
}

#[derive(Clone, PartialEq, Hash, Eq)]
pub enum FieldElementExpression<T: Field> {
    Number(T),
    Identifier(Identifier),
    Add(
        Box<FieldElementExpression<T>>,
        Box<FieldElementExpression<T>>,
    ),
    Sub(
        Box<FieldElementExpression<T>>,
        Box<FieldElementExpression<T>>,
    ),
    Mult(
        Box<FieldElementExpression<T>>,
        Box<FieldElementExpression<T>>,
    ),
    Div(
        Box<FieldElementExpression<T>>,
        Box<FieldElementExpression<T>>,
    ),
    Pow(
        Box<FieldElementExpression<T>>,
        Box<FieldElementExpression<T>>,
    ),
    IfElse(
        Box<BooleanExpression<T>>,
        Box<FieldElementExpression<T>>,
        Box<FieldElementExpression<T>>,
    ),
    FunctionCall(FunctionKey, Vec<TypedExpression<T>>),
    Select(
        Box<FieldElementArrayExpression<T>>,
        Box<FieldElementExpression<T>>,
    ),
}

#[derive(Clone, PartialEq, Hash, Eq)]
pub enum BooleanExpression<T: Field> {
    Identifier(Identifier),
    Value(bool),
    Lt(
        Box<FieldElementExpression<T>>,
        Box<FieldElementExpression<T>>,
    ),
    Le(
        Box<FieldElementExpression<T>>,
        Box<FieldElementExpression<T>>,
    ),
    Eq(
        Box<FieldElementExpression<T>>,
        Box<FieldElementExpression<T>>,
    ),
    Ge(
        Box<FieldElementExpression<T>>,
        Box<FieldElementExpression<T>>,
    ),
    Gt(
        Box<FieldElementExpression<T>>,
        Box<FieldElementExpression<T>>,
    ),
    Or(Box<BooleanExpression<T>>, Box<BooleanExpression<T>>),
    And(Box<BooleanExpression<T>>, Box<BooleanExpression<T>>),
    Not(Box<BooleanExpression<T>>),
}

// for now we store the array size in the variants
#[derive(Clone, PartialEq, Hash, Eq)]
pub enum FieldElementArrayExpression<T: Field> {
    Identifier(usize, Identifier),
    Value(usize, Vec<FieldElementExpression<T>>),
    FunctionCall(usize, FunctionKey, Vec<TypedExpression<T>>),
    IfElse(
        Box<BooleanExpression<T>>,
        Box<FieldElementArrayExpression<T>>,
        Box<FieldElementArrayExpression<T>>,
    ),
}

impl<T: Field> FieldElementArrayExpression<T> {
    pub fn size(&self) -> usize {
        match *self {
            FieldElementArrayExpression::Identifier(s, _)
            | FieldElementArrayExpression::Value(s, _)
            | FieldElementArrayExpression::FunctionCall(s, ..) => s,
            FieldElementArrayExpression::IfElse(_, ref consequence, _) => consequence.size(),
        }
    }
}

impl<T: Field> fmt::Display for FieldElementExpression<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FieldElementExpression::Number(ref i) => write!(f, "{}", i),
            FieldElementExpression::Identifier(ref var) => write!(f, "{}", var),
            FieldElementExpression::Add(ref lhs, ref rhs) => write!(f, "({} + {})", lhs, rhs),
            FieldElementExpression::Sub(ref lhs, ref rhs) => write!(f, "({} - {})", lhs, rhs),
            FieldElementExpression::Mult(ref lhs, ref rhs) => write!(f, "({} * {})", lhs, rhs),
            FieldElementExpression::Div(ref lhs, ref rhs) => write!(f, "({} / {})", lhs, rhs),
            FieldElementExpression::Pow(ref lhs, ref rhs) => write!(f, "{}**{}", lhs, rhs),
            FieldElementExpression::IfElse(ref condition, ref consequent, ref alternative) => {
                write!(
                    f,
                    "if {} then {} else {} fi",
                    condition, consequent, alternative
                )
            }
            FieldElementExpression::FunctionCall(ref k, ref p) => {
                r#try!(write!(f, "{}(", k.id,));
                for (i, param) in p.iter().enumerate() {
                    r#try!(write!(f, "{}", param));
                    if i < p.len() - 1 {
                        r#try!(write!(f, ", "));
                    }
                }
                write!(f, ")")
            }
            FieldElementExpression::Select(ref id, ref index) => write!(f, "{}[{}]", id, index),
        }
    }
}

impl<T: Field> fmt::Display for BooleanExpression<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BooleanExpression::Identifier(ref var) => write!(f, "{}", var),
            BooleanExpression::Lt(ref lhs, ref rhs) => write!(f, "{} < {}", lhs, rhs),
            BooleanExpression::Le(ref lhs, ref rhs) => write!(f, "{} <= {}", lhs, rhs),
            BooleanExpression::Eq(ref lhs, ref rhs) => write!(f, "{} == {}", lhs, rhs),
            BooleanExpression::Ge(ref lhs, ref rhs) => write!(f, "{} >= {}", lhs, rhs),
            BooleanExpression::Gt(ref lhs, ref rhs) => write!(f, "{} > {}", lhs, rhs),
            BooleanExpression::Or(ref lhs, ref rhs) => write!(f, "{} || {}", lhs, rhs),
            BooleanExpression::And(ref lhs, ref rhs) => write!(f, "{} && {}", lhs, rhs),
            BooleanExpression::Not(ref exp) => write!(f, "!{}", exp),
            BooleanExpression::Value(b) => write!(f, "{}", b),
        }
    }
}

impl<T: Field> fmt::Display for FieldElementArrayExpression<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FieldElementArrayExpression::Identifier(_, ref var) => write!(f, "{}", var),
            FieldElementArrayExpression::Value(_, ref values) => write!(
                f,
                "[{}]",
                values
                    .iter()
                    .map(|o| o.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            FieldElementArrayExpression::FunctionCall(_, ref key, ref p) => {
                r#try!(write!(f, "{}(", key.id,));
                for (i, param) in p.iter().enumerate() {
                    r#try!(write!(f, "{}", param));
                    if i < p.len() - 1 {
                        r#try!(write!(f, ", "));
                    }
                }
                write!(f, ")")
            }
            FieldElementArrayExpression::IfElse(ref condition, ref consequent, ref alternative) => {
                write!(
                    f,
                    "if {} then {} else {} fi",
                    condition, consequent, alternative
                )
            }
        }
    }
}

impl<T: Field> fmt::Debug for BooleanExpression<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl<T: Field> fmt::Debug for FieldElementExpression<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FieldElementExpression::Number(ref i) => write!(f, "Num({})", i),
            FieldElementExpression::Identifier(ref var) => write!(f, "Ide({})", var),
            FieldElementExpression::Add(ref lhs, ref rhs) => write!(f, "Add({:?}, {:?})", lhs, rhs),
            FieldElementExpression::Sub(ref lhs, ref rhs) => write!(f, "Sub({:?}, {:?})", lhs, rhs),
            FieldElementExpression::Mult(ref lhs, ref rhs) => {
                write!(f, "Mult({:?}, {:?})", lhs, rhs)
            }
            FieldElementExpression::Div(ref lhs, ref rhs) => write!(f, "Div({:?}, {:?})", lhs, rhs),
            FieldElementExpression::Pow(ref lhs, ref rhs) => write!(f, "Pow({:?}, {:?})", lhs, rhs),
            FieldElementExpression::IfElse(ref condition, ref consequent, ref alternative) => {
                write!(
                    f,
                    "IfElse({:?}, {:?}, {:?})",
                    condition, consequent, alternative
                )
            }
            FieldElementExpression::FunctionCall(ref i, ref p) => {
                r#try!(write!(f, "FunctionCall({:?}, (", i));
                r#try!(f.debug_list().entries(p.iter()).finish());
                write!(f, ")")
            }
            FieldElementExpression::Select(ref id, ref index) => {
                write!(f, "Select({:?}, {:?})", id, index)
            }
        }
    }
}

impl<T: Field> fmt::Debug for FieldElementArrayExpression<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FieldElementArrayExpression::Identifier(_, ref var) => write!(f, "{:?}", var),
            FieldElementArrayExpression::Value(_, ref values) => write!(f, "{:?}", values),
            FieldElementArrayExpression::FunctionCall(_, ref i, ref p) => {
                r#try!(write!(f, "FunctionCall({:?}, (", i));
                r#try!(f.debug_list().entries(p.iter()).finish());
                write!(f, ")")
            }
            FieldElementArrayExpression::IfElse(ref condition, ref consequent, ref alternative) => {
                write!(
                    f,
                    "IfElse({:?}, {:?}, {:?})",
                    condition, consequent, alternative
                )
            }
        }
    }
}

impl<T: Field> fmt::Display for TypedExpressionList<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TypedExpressionList::FunctionCall(ref key, ref p, _) => {
                r#try!(write!(f, "{}(", key.id,));
                for (i, param) in p.iter().enumerate() {
                    r#try!(write!(f, "{}", param));
                    if i < p.len() - 1 {
                        r#try!(write!(f, ", "));
                    }
                }
                write!(f, ")")
            }
        }
    }
}

impl<T: Field> fmt::Debug for TypedExpressionList<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TypedExpressionList::FunctionCall(ref i, ref p, _) => {
                r#try!(write!(f, "FunctionCall({:?}, (", i));
                r#try!(f.debug_list().entries(p.iter()).finish());
                write!(f, ")")
            }
        }
    }
}
