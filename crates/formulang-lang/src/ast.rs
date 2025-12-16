use crate::lexer::Span;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Import(Import),
    Nutrient(Nutrient),
    Ingredient(Ingredient),
    Formula(Formula),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub span: Span,
    pub path: String,
    pub alias: Option<String>,
    pub selections: Option<ImportSelections>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum ImportSelections {
    All,                  // { * }
    Named(Vec<String>),   // { protein, energy }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Nutrient {
    pub span: Span,
    pub name: String,
    pub properties: Vec<Property>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Ingredient {
    pub span: Span,
    pub name: String,
    pub properties: Vec<Property>,
    pub nutrients: Vec<NutrientValue>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct NutrientValue {
    pub span: Span,
    pub nutrient: Reference,
    pub value: f64,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Formula {
    pub span: Span,
    pub name: String,
    pub properties: Vec<Property>,
    pub nutrients: Vec<NutrientConstraint>,
    pub ingredients: Vec<IngredientConstraint>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub span: Span,
    pub name: String,
    pub value: PropertyValue,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    String(String),
    Number(f64),
    Ident(String),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct NutrientConstraint {
    pub span: Span,
    pub expr: Expr,
    pub bounds: Bounds,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct IngredientConstraint {
    pub span: Span,
    pub expr: Expr,
    pub bounds: Bounds,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Bounds {
    pub min: Option<BoundValue>,
    pub max: Option<BoundValue>,
}

impl Bounds {
    pub fn none() -> Self {
        Self { min: None, max: None }
    }

    pub fn min(value: BoundValue) -> Self {
        Self { min: Some(value), max: None }
    }

    pub fn max(value: BoundValue) -> Self {
        Self { min: None, max: Some(value) }
    }

    pub fn range(min: BoundValue, max: BoundValue) -> Self {
        Self { min: Some(min), max: Some(max) }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct BoundValue {
    pub value: f64,
    pub is_percent: bool,
}

impl BoundValue {
    pub fn absolute(value: f64) -> Self {
        Self { value, is_percent: false }
    }

    pub fn percent(value: f64) -> Self {
        Self { value, is_percent: true }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Reference(Reference),
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Paren(Box<Expr>),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Reference {
    pub span: Span,
    pub parts: Vec<ReferencePart>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum ReferencePart {
    Ident(String),
    Selection(Vec<String>),  // [protein, energy]
    Min,
    Max,
}

impl Reference {
    pub fn simple(span: Span, name: impl Into<String>) -> Self {
        Self {
            span,
            parts: vec![ReferencePart::Ident(name.into())],
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
        }
    }
}
