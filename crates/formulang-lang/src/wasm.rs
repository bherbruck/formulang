//! WASM bindings for Formulang
//!
//! This module provides JavaScript-friendly APIs for use in Monaco editors
//! and other web-based tooling.

use wasm_bindgen::prelude::*;

use crate::ast::*;
use crate::compiler::Compiler;
use crate::lexer::{Lexer, TokenKind};
use crate::parser::Parser;
use formulang_solver::{Solver, SolutionStatus};

/// Parse source code and return the AST as JSON
#[wasm_bindgen]
pub fn parse(source: &str) -> Result<JsValue, JsValue> {
    let program = Parser::parse(source).map_err(|e| JsValue::from_str(&e.to_string()))?;
    serde_wasm_bindgen::to_value(&program).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Tokenize source code and return tokens as JSON
#[wasm_bindgen]
pub fn tokenize(source: &str) -> Result<JsValue, JsValue> {
    let tokens: Vec<TokenInfo> = Lexer::tokenize(source)
        .into_iter()
        .map(|t| TokenInfo {
            kind: format!("{:?}", t.kind),
            text: t.text.to_string(),
            start: t.span.start,
            end: t.span.end,
        })
        .collect();
    serde_wasm_bindgen::to_value(&tokens).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Token information for JavaScript
#[derive(serde::Serialize)]
struct TokenInfo {
    kind: String,
    text: String,
    start: usize,
    end: usize,
}

/// Validate source code and return diagnostics as JSON
#[wasm_bindgen]
pub fn validate(source: &str) -> JsValue {
    let diagnostics = get_diagnostics(source);
    serde_wasm_bindgen::to_value(&diagnostics).unwrap_or(JsValue::NULL)
}

/// Get semantic tokens for syntax highlighting
#[wasm_bindgen]
pub fn get_semantic_tokens(source: &str) -> Result<JsValue, JsValue> {
    let tokens: Vec<SemanticToken> = Lexer::tokenize(source)
        .into_iter()
        .map(|t| {
            let token_type = match t.kind {
                TokenKind::Nutrient
                | TokenKind::Ingredient
                | TokenKind::Formula
                | TokenKind::Import => "keyword",
                TokenKind::Min | TokenKind::Max => "keyword",
                TokenKind::Ident => "variable",
                TokenKind::Number => "number",
                TokenKind::String => "string",
                TokenKind::Comment => "comment",
                TokenKind::Colon | TokenKind::Comma => "delimiter",
                TokenKind::LBrace | TokenKind::RBrace => "delimiter",
                TokenKind::LBracket | TokenKind::RBracket => "delimiter",
                TokenKind::LParen | TokenKind::RParen => "delimiter",
                TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash => {
                    "operator"
                }
                TokenKind::Percent => "operator",
                TokenKind::Dot => "delimiter",
                TokenKind::Newline => "whitespace",
                TokenKind::Error | TokenKind::Eof => "error",
            };
            SemanticToken {
                start: t.span.start,
                end: t.span.end,
                token_type: token_type.to_string(),
            }
        })
        .collect();
    serde_wasm_bindgen::to_value(&tokens).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[derive(serde::Serialize)]
struct SemanticToken {
    start: usize,
    end: usize,
    token_type: String,
}

/// Get completions at a given position
#[wasm_bindgen]
pub fn get_completions(source: &str, position: usize) -> Result<JsValue, JsValue> {
    let completions = compute_completions(source, position);
    serde_wasm_bindgen::to_value(&completions).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[derive(serde::Serialize)]
struct Completion {
    label: String,
    kind: String,
    detail: Option<String>,
    insert_text: String,
}

/// Context for completions based on cursor position
#[derive(Debug)]
enum CompletionContext {
    /// At top-level (suggest nutrient, ingredient, formula, import)
    TopLevel,
    /// After a formula name and dot (e.g., "base.")
    AfterFormulaDot(String),
    /// After formula.nutrients. or formula.ingredients. (e.g., "base.nutrients.")
    AfterBlockDot(String, String), // (formula_name, block_type)
    /// After formula.block.item. (e.g., "base.nutrients.protein.")
    AfterItemDot(String, String, String), // (formula_name, block_type, item_name)
    /// Inside a nutrients block (suggest nutrients and formulas for composition)
    InNutrientsBlock,
    /// Inside an ingredients block (suggest ingredients and formulas for composition)
    InIngredientsBlock,
    /// General context (suggest all symbols)
    General,
}

/// Parse the text before cursor to determine completion context
fn get_completion_context(source: &str, position: usize) -> CompletionContext {
    let prefix = &source[..position.min(source.len())];

    // Find what block we're in by looking for unmatched braces
    let mut brace_depth = 0;
    let mut last_block_type: Option<&str> = None;
    let mut in_formula = false;

    for (i, c) in prefix.char_indices() {
        if c == '{' {
            brace_depth += 1;
            // Check what keyword preceded this brace
            let before = prefix[..i].trim_end();
            if before.ends_with("nutrients") || before.ends_with("nuts") {
                if brace_depth >= 2 && in_formula {
                    last_block_type = Some("nutrients");
                }
            } else if before.ends_with("ingredients") || before.ends_with("ings") {
                if brace_depth >= 2 && in_formula {
                    last_block_type = Some("ingredients");
                }
            } else if before.split_whitespace().last() == Some("formula") ||
                     before.split_whitespace().rev().nth(1) == Some("formula") {
                in_formula = true;
            }
        } else if c == '}' {
            brace_depth -= 1;
            if brace_depth < 2 {
                last_block_type = None;
            }
            if brace_depth < 1 {
                in_formula = false;
            }
        }
    }

    // Check if we're after a dot (dot completion)
    let line_start = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line = prefix[line_start..].trim_start();

    if let Some(dot_pos) = line.rfind('.') {
        let before_dot = &line[..dot_pos];

        // Parse the reference parts before the dot
        let parts: Vec<&str> = before_dot.split('.').collect();

        match parts.len() {
            1 => {
                // formula_name. -> suggest nutrients, ingredients
                let formula_name = parts[0].trim();
                if !formula_name.is_empty() && formula_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    return CompletionContext::AfterFormulaDot(formula_name.to_string());
                }
            }
            2 => {
                // formula_name.block. -> suggest items from that block
                let formula_name = parts[0].trim();
                let block_name = parts[1].trim();
                if !formula_name.is_empty() && !block_name.is_empty() {
                    let block_type = match block_name {
                        "nutrients" | "nuts" => "nutrients",
                        "ingredients" | "ings" => "ingredients",
                        _ => block_name,
                    };
                    return CompletionContext::AfterBlockDot(
                        formula_name.to_string(),
                        block_type.to_string(),
                    );
                }
            }
            3 => {
                // formula_name.block.item. -> suggest min, max
                let formula_name = parts[0].trim();
                let block_name = parts[1].trim();
                let item_name = parts[2].trim();
                if !formula_name.is_empty() && !block_name.is_empty() && !item_name.is_empty() {
                    let block_type = match block_name {
                        "nutrients" | "nuts" => "nutrients",
                        "ingredients" | "ings" => "ingredients",
                        _ => block_name,
                    };
                    return CompletionContext::AfterItemDot(
                        formula_name.to_string(),
                        block_type.to_string(),
                        item_name.to_string(),
                    );
                }
            }
            _ => {}
        }
    }

    // Determine context based on block
    match last_block_type {
        Some("nutrients") => CompletionContext::InNutrientsBlock,
        Some("ingredients") => CompletionContext::InIngredientsBlock,
        _ => {
            // Check if at top level (outside any braces)
            if brace_depth == 0 && line.is_empty() {
                CompletionContext::TopLevel
            } else {
                CompletionContext::General
            }
        }
    }
}

fn compute_completions(source: &str, position: usize) -> Vec<Completion> {
    let mut completions = Vec::new();
    let context = get_completion_context(source, position);

    // Parse current document to get defined symbols
    let program = Parser::parse(source).ok();

    match context {
        CompletionContext::TopLevel => {
            // Suggest top-level keywords
            completions.push(Completion {
                label: "nutrient".to_string(),
                kind: "keyword".to_string(),
                detail: Some("Define a nutrient".to_string()),
                insert_text: "nutrient ${1:name} {\n  name \"${2:Display Name}\"\n  code \"${3}\"\n  unit \"${4:%}\"\n}".to_string(),
            });
            completions.push(Completion {
                label: "ingredient".to_string(),
                kind: "keyword".to_string(),
                detail: Some("Define an ingredient".to_string()),
                insert_text: "ingredient ${1:name} {\n  name \"${2:Display Name}\"\n  code \"${3}\"\n  cost ${4:0}\n  nuts {\n    ${5:nutrient} ${6:0}\n  }\n}".to_string(),
            });
            completions.push(Completion {
                label: "formula".to_string(),
                kind: "keyword".to_string(),
                detail: Some("Define a formula".to_string()),
                insert_text: "formula ${1:name} {\n  name \"${2:Display Name}\"\n  code \"${3}\"\n  desc \"${4}\"\n  batch ${5:1000}\n  \n  nuts {\n    ${6:nutrient} min ${7:0}\n  }\n  \n  ings {\n    ${8:ingredient}\n  }\n}".to_string(),
            });
            completions.push(Completion {
                label: "import".to_string(),
                kind: "keyword".to_string(),
                detail: Some("Import from another file".to_string()),
                insert_text: "import \"${1:./file.fm}\"".to_string(),
            });
        }

        CompletionContext::AfterFormulaDot(formula_name) => {
            // Check if this formula exists
            if let Some(ref prog) = program {
                let formula_exists = prog.items.iter().any(|item| {
                    matches!(item, Item::Formula(f) if f.name == formula_name)
                });
                if formula_exists {
                    completions.push(Completion {
                        label: "nutrients".to_string(),
                        kind: "property".to_string(),
                        detail: Some("Include all nutrient constraints".to_string()),
                        insert_text: "nutrients".to_string(),
                    });
                    completions.push(Completion {
                        label: "ingredients".to_string(),
                        kind: "property".to_string(),
                        detail: Some("Include all ingredient constraints".to_string()),
                        insert_text: "ingredients".to_string(),
                    });
                    completions.push(Completion {
                        label: "nuts".to_string(),
                        kind: "property".to_string(),
                        detail: Some("Include all nutrient constraints (short form)".to_string()),
                        insert_text: "nuts".to_string(),
                    });
                    completions.push(Completion {
                        label: "ings".to_string(),
                        kind: "property".to_string(),
                        detail: Some("Include all ingredient constraints (short form)".to_string()),
                        insert_text: "ings".to_string(),
                    });
                }
            }
        }

        CompletionContext::AfterBlockDot(formula_name, block_type) => {
            if let Some(ref prog) = program {
                // Find the formula
                for item in &prog.items {
                    if let Item::Formula(f) = item {
                        if f.name == formula_name {
                            if block_type == "nutrients" {
                                // Suggest nutrients from this formula's constraints
                                for nc in &f.nutrients {
                                    if let Some(name) = get_expr_name(&nc.expr) {
                                        completions.push(Completion {
                                            label: name.clone(),
                                            kind: "variable".to_string(),
                                            detail: Some(format!("Include {} constraint", name)),
                                            insert_text: name,
                                        });
                                    }
                                }
                                // Also suggest all defined nutrients
                                for item2 in &prog.items {
                                    if let Item::Nutrient(n) = item2 {
                                        if !completions.iter().any(|c| c.label == n.name) {
                                            completions.push(Completion {
                                                label: n.name.clone(),
                                                kind: "variable".to_string(),
                                                detail: Some("Nutrient".to_string()),
                                                insert_text: n.name.clone(),
                                            });
                                        }
                                    }
                                }
                            } else if block_type == "ingredients" {
                                // Suggest ingredients from this formula's constraints
                                for ic in &f.ingredients {
                                    if let Some(name) = get_expr_name(&ic.expr) {
                                        completions.push(Completion {
                                            label: name.clone(),
                                            kind: "variable".to_string(),
                                            detail: Some(format!("Include {} constraint", name)),
                                            insert_text: name,
                                        });
                                    }
                                }
                                // Also suggest all defined ingredients
                                for item2 in &prog.items {
                                    if let Item::Ingredient(i) = item2 {
                                        if !completions.iter().any(|c| c.label == i.name) {
                                            completions.push(Completion {
                                                label: i.name.clone(),
                                                kind: "variable".to_string(),
                                                detail: Some("Ingredient".to_string()),
                                                insert_text: i.name.clone(),
                                            });
                                        }
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }

        CompletionContext::AfterItemDot(_formula_name, _block_type, _item_name) => {
            // After formula.block.item. -> suggest min, max
            completions.push(Completion {
                label: "min".to_string(),
                kind: "keyword".to_string(),
                detail: Some("Include only the minimum bound".to_string()),
                insert_text: "min".to_string(),
            });
            completions.push(Completion {
                label: "max".to_string(),
                kind: "keyword".to_string(),
                detail: Some("Include only the maximum bound".to_string()),
                insert_text: "max".to_string(),
            });
        }

        CompletionContext::InNutrientsBlock => {
            // Suggest nutrients
            if let Some(ref prog) = program {
                for item in &prog.items {
                    match item {
                        Item::Nutrient(n) => {
                            completions.push(Completion {
                                label: n.name.clone(),
                                kind: "variable".to_string(),
                                detail: Some("Nutrient".to_string()),
                                insert_text: n.name.clone(),
                            });
                        }
                        Item::Formula(f) => {
                            // Suggest formulas for composition
                            completions.push(Completion {
                                label: f.name.clone(),
                                kind: "variable".to_string(),
                                detail: Some("Formula (for composition)".to_string()),
                                insert_text: f.name.clone(),
                            });
                        }
                        _ => {}
                    }
                }
            }
            // Add constraint keywords
            completions.push(Completion {
                label: "min".to_string(),
                kind: "keyword".to_string(),
                detail: Some("Minimum constraint".to_string()),
                insert_text: "min ${1:0}".to_string(),
            });
            completions.push(Completion {
                label: "max".to_string(),
                kind: "keyword".to_string(),
                detail: Some("Maximum constraint".to_string()),
                insert_text: "max ${1:0}".to_string(),
            });
        }

        CompletionContext::InIngredientsBlock => {
            // Suggest ingredients
            if let Some(ref prog) = program {
                for item in &prog.items {
                    match item {
                        Item::Ingredient(i) => {
                            completions.push(Completion {
                                label: i.name.clone(),
                                kind: "variable".to_string(),
                                detail: Some("Ingredient".to_string()),
                                insert_text: i.name.clone(),
                            });
                        }
                        Item::Formula(f) => {
                            // Suggest formulas for composition
                            completions.push(Completion {
                                label: f.name.clone(),
                                kind: "variable".to_string(),
                                detail: Some("Formula (for composition)".to_string()),
                                insert_text: f.name.clone(),
                            });
                        }
                        _ => {}
                    }
                }
            }
            // Add constraint keywords
            completions.push(Completion {
                label: "min".to_string(),
                kind: "keyword".to_string(),
                detail: Some("Minimum constraint".to_string()),
                insert_text: "min ${1:0}%".to_string(),
            });
            completions.push(Completion {
                label: "max".to_string(),
                kind: "keyword".to_string(),
                detail: Some("Maximum constraint".to_string()),
                insert_text: "max ${1:0}%".to_string(),
            });
        }

        CompletionContext::General => {
            // Suggest all symbols
            if let Some(ref prog) = program {
                for item in &prog.items {
                    match item {
                        Item::Nutrient(n) => {
                            completions.push(Completion {
                                label: n.name.clone(),
                                kind: "variable".to_string(),
                                detail: Some("Nutrient".to_string()),
                                insert_text: n.name.clone(),
                            });
                        }
                        Item::Ingredient(i) => {
                            completions.push(Completion {
                                label: i.name.clone(),
                                kind: "variable".to_string(),
                                detail: Some("Ingredient".to_string()),
                                insert_text: i.name.clone(),
                            });
                        }
                        Item::Formula(f) => {
                            completions.push(Completion {
                                label: f.name.clone(),
                                kind: "variable".to_string(),
                                detail: Some("Formula".to_string()),
                                insert_text: f.name.clone(),
                            });
                        }
                        _ => {}
                    }
                }
            }
            // Add constraint keywords
            completions.push(Completion {
                label: "min".to_string(),
                kind: "keyword".to_string(),
                detail: Some("Minimum constraint".to_string()),
                insert_text: "min ${1:0}".to_string(),
            });
            completions.push(Completion {
                label: "max".to_string(),
                kind: "keyword".to_string(),
                detail: Some("Maximum constraint".to_string()),
                insert_text: "max ${1:0}".to_string(),
            });
        }
    }

    completions
}

/// Get the first identifier name from an expression
fn get_expr_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Reference(r) => {
            if let Some(ReferencePart::Ident(name)) = r.parts.first() {
                Some(name.clone())
            } else {
                None
            }
        }
        _ => None,
    }
}

#[derive(serde::Serialize)]
struct Diagnostic {
    start: usize,
    end: usize,
    severity: String,
    message: String,
}

fn get_diagnostics(source: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Parse and collect errors
    match Parser::parse(source) {
        Err(e) => {
            diagnostics.push(Diagnostic {
                start: 0,
                end: source.len(),
                severity: "error".to_string(),
                message: e.to_string(),
            });
        }
        Ok(program) => {
            // Semantic validation
            validate_program(&program, &mut diagnostics);
        }
    }

    diagnostics
}

fn validate_program(program: &Program, diagnostics: &mut Vec<Diagnostic>) {
    use std::collections::HashSet;

    // Collect all defined names
    let mut nutrients: HashSet<&str> = HashSet::new();
    let mut ingredients: HashSet<&str> = HashSet::new();
    let mut formulas: HashSet<&str> = HashSet::new();

    // First pass: collect definitions and check for duplicates
    for item in &program.items {
        match item {
            Item::Nutrient(n) => {
                if !nutrients.insert(&n.name) {
                    diagnostics.push(Diagnostic {
                        start: n.span.start,
                        end: n.span.end,
                        severity: "error".to_string(),
                        message: format!("Duplicate nutrient definition: '{}'", n.name),
                    });
                }
            }
            Item::Ingredient(i) => {
                if !ingredients.insert(&i.name) {
                    diagnostics.push(Diagnostic {
                        start: i.span.start,
                        end: i.span.end,
                        severity: "error".to_string(),
                        message: format!("Duplicate ingredient definition: '{}'", i.name),
                    });
                }
            }
            Item::Formula(f) => {
                if !formulas.insert(&f.name) {
                    diagnostics.push(Diagnostic {
                        start: f.span.start,
                        end: f.span.end,
                        severity: "error".to_string(),
                        message: format!("Duplicate formula definition: '{}'", f.name),
                    });
                }
            }
            Item::Import(_) => {}
        }
    }

    // Valid properties for each declaration type
    let nutrient_props = ["name", "code", "desc", "description", "unit"];
    let ingredient_props = ["name", "code", "desc", "description", "cost"];
    let formula_props = ["name", "code", "desc", "description", "batch", "batch_size"];

    // Second pass: check references and property scopes
    for item in &program.items {
        match item {
            Item::Nutrient(n) => {
                // Validate nutrient properties
                for prop in &n.properties {
                    if !nutrient_props.contains(&prop.name.as_str()) {
                        diagnostics.push(Diagnostic {
                            start: prop.span.start,
                            end: prop.span.end,
                            severity: "error".to_string(),
                            message: format!("'{}' is not a valid property for nutrient. Valid properties: name, code, desc, unit", prop.name),
                        });
                    }
                }
            }
            Item::Ingredient(ing) => {
                // Validate ingredient properties
                for prop in &ing.properties {
                    if !ingredient_props.contains(&prop.name.as_str()) {
                        diagnostics.push(Diagnostic {
                            start: prop.span.start,
                            end: prop.span.end,
                            severity: "error".to_string(),
                            message: format!("'{}' is not a valid property for ingredient. Valid properties: name, code, desc, cost", prop.name),
                        });
                    }
                }

                // Check for missing cost
                let has_cost = ing.properties.iter().any(|p| p.name == "cost");
                if !has_cost {
                    diagnostics.push(Diagnostic {
                        start: ing.span.start,
                        end: ing.span.start + 10, // "ingredient"
                        severity: "warning".to_string(),
                        message: format!("Ingredient '{}' is missing required 'cost' property", ing.name),
                    });
                }

                // Ingredient nuts block: only nutrients allowed
                for nv in &ing.nutrients {
                    if let Some(name) = get_reference_name(&nv.nutrient) {
                        if ingredients.contains(name.as_str()) {
                            diagnostics.push(Diagnostic {
                                start: nv.span.start,
                                end: nv.span.end,
                                severity: "error".to_string(),
                                message: format!("'{}' is an ingredient, not a nutrient. Only nutrients can be referenced in an ingredient's nuts block.", name),
                            });
                        } else if !nutrients.contains(name.as_str()) {
                            diagnostics.push(Diagnostic {
                                start: nv.span.start,
                                end: nv.span.end,
                                severity: "error".to_string(),
                                message: format!("Undefined nutrient: '{}'", name),
                            });
                        }
                    }
                }
            }
            Item::Formula(formula) => {
                // Validate formula properties
                for prop in &formula.properties {
                    if !formula_props.contains(&prop.name.as_str()) {
                        diagnostics.push(Diagnostic {
                            start: prop.span.start,
                            end: prop.span.end,
                            severity: "error".to_string(),
                            message: format!("'{}' is not a valid property for formula. Valid properties: name, code, desc, batch", prop.name),
                        });
                    }
                }

                // Formula nuts block: only nutrients allowed
                for nc in &formula.nutrients {
                    check_nutrient_expr(&nc.expr, &nutrients, &ingredients, &formulas, diagnostics);
                }

                // Formula ings block: only ingredients allowed
                for ic in &formula.ingredients {
                    check_ingredient_expr(&ic.expr, &nutrients, &ingredients, &formulas, diagnostics);
                }

                // Check for missing batch_size
                let has_batch = formula.properties.iter().any(|p| {
                    p.name == "batch_size" || p.name == "batch"
                });
                if !has_batch {
                    diagnostics.push(Diagnostic {
                        start: formula.span.start,
                        end: formula.span.start + 7, // "formula"
                        severity: "warning".to_string(),
                        message: format!("Formula '{}' is missing required 'batch' property", formula.name),
                    });
                }
            }
            Item::Import(_) => {}
        }
    }
}

fn get_reference_name(reference: &Reference) -> Option<String> {
    if let Some(ReferencePart::Ident(name)) = reference.parts.first() {
        Some(name.clone())
    } else {
        None
    }
}

/// Check if a reference is a composition reference (e.g., base.nutrients, base.nutrients.protein)
fn is_composition_reference(r: &Reference, formulas: &std::collections::HashSet<&str>) -> bool {
    // Composition references have at least 2 parts: formula.nutrients or formula.ingredients
    if r.parts.len() >= 2 {
        if let (ReferencePart::Ident(formula_name), ReferencePart::Ident(block_type)) =
            (&r.parts[0], &r.parts[1])
        {
            if formulas.contains(formula_name.as_str()) {
                return matches!(
                    block_type.as_str(),
                    "nutrients" | "nuts" | "ingredients" | "ings"
                );
            }
        }
    }
    false
}

/// Check that an expression in a formula's nuts block only references nutrients
fn check_nutrient_expr(
    expr: &Expr,
    nutrients: &std::collections::HashSet<&str>,
    ingredients: &std::collections::HashSet<&str>,
    formulas: &std::collections::HashSet<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match expr {
        Expr::Reference(r) => {
            // Check for composition reference first
            if is_composition_reference(r, formulas) {
                // Valid composition reference, no error
                return;
            }

            if let Some(name) = get_reference_name(r) {
                if ingredients.contains(name.as_str()) {
                    diagnostics.push(Diagnostic {
                        start: r.span.start,
                        end: r.span.end,
                        severity: "error".to_string(),
                        message: format!("'{}' is an ingredient, not a nutrient. Only nutrients can be referenced in a formula's nuts block.", name),
                    });
                } else if !nutrients.contains(name.as_str()) && !formulas.contains(name.as_str()) {
                    diagnostics.push(Diagnostic {
                        start: r.span.start,
                        end: r.span.end,
                        severity: "error".to_string(),
                        message: format!("Undefined nutrient: '{}'", name),
                    });
                }
            }
        }
        Expr::BinaryOp { left, right, .. } => {
            check_nutrient_expr(left, nutrients, ingredients, formulas, diagnostics);
            check_nutrient_expr(right, nutrients, ingredients, formulas, diagnostics);
        }
        Expr::Paren(inner) => {
            check_nutrient_expr(inner, nutrients, ingredients, formulas, diagnostics);
        }
        Expr::Number(_) => {}
    }
}

/// Check that an expression in a formula's ings block only references ingredients
fn check_ingredient_expr(
    expr: &Expr,
    nutrients: &std::collections::HashSet<&str>,
    ingredients: &std::collections::HashSet<&str>,
    formulas: &std::collections::HashSet<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match expr {
        Expr::Reference(r) => {
            // Check for composition reference first
            if is_composition_reference(r, formulas) {
                // Valid composition reference, no error
                return;
            }

            if let Some(name) = get_reference_name(r) {
                if nutrients.contains(name.as_str()) {
                    diagnostics.push(Diagnostic {
                        start: r.span.start,
                        end: r.span.end,
                        severity: "error".to_string(),
                        message: format!("'{}' is a nutrient, not an ingredient. Only ingredients can be referenced in a formula's ings block.", name),
                    });
                } else if !ingredients.contains(name.as_str()) && !formulas.contains(name.as_str()) {
                    diagnostics.push(Diagnostic {
                        start: r.span.start,
                        end: r.span.end,
                        severity: "error".to_string(),
                        message: format!("Undefined ingredient: '{}'", name),
                    });
                }
            }
        }
        Expr::BinaryOp { left, right, .. } => {
            check_ingredient_expr(left, nutrients, ingredients, formulas, diagnostics);
            check_ingredient_expr(right, nutrients, ingredients, formulas, diagnostics);
        }
        Expr::Paren(inner) => {
            check_ingredient_expr(inner, nutrients, ingredients, formulas, diagnostics);
        }
        Expr::Number(_) => {}
    }
}

/// Get hover information at a position
#[wasm_bindgen]
pub fn get_hover(source: &str, position: usize) -> Result<JsValue, JsValue> {
    let hover = compute_hover(source, position);
    serde_wasm_bindgen::to_value(&hover).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[derive(serde::Serialize)]
struct HoverInfo {
    contents: String,
    start: usize,
    end: usize,
}

fn compute_hover(source: &str, position: usize) -> Option<HoverInfo> {
    // Find the token at the position
    let tokens = Lexer::tokenize(source);
    let token = tokens
        .into_iter()
        .find(|t| t.span.start <= position && position <= t.span.end)?;

    // Return info based on token type
    let contents = match token.kind {
        TokenKind::Nutrient => "**nutrient**\n\nDefines a nutrient that can be tracked in ingredients and constrained in formulas.".to_string(),
        TokenKind::Ingredient => "**ingredient**\n\nDefines a feed ingredient with cost and nutrient composition.".to_string(),
        TokenKind::Formula => "**formula**\n\nDefines a feed formula with nutrient requirements and ingredient constraints.".to_string(),
        TokenKind::Import => "**import**\n\nImports definitions from another .fm file.".to_string(),
        TokenKind::Min => "**min**\n\nSets a minimum bound for a constraint.".to_string(),
        TokenKind::Max => "**max**\n\nSets a maximum bound for a constraint.".to_string(),
        TokenKind::Ident => {
            // Try to find this identifier in the parsed program
            if let Ok(program) = Parser::parse(source) {
                for item in &program.items {
                    match item {
                        Item::Nutrient(n) if n.name == token.text => {
                            return Some(HoverInfo {
                                contents: format!("**Nutrient** `{}`", n.name),
                                start: token.span.start,
                                end: token.span.end,
                            });
                        }
                        Item::Ingredient(i) if i.name == token.text => {
                            return Some(HoverInfo {
                                contents: format!("**Ingredient** `{}`", i.name),
                                start: token.span.start,
                                end: token.span.end,
                            });
                        }
                        Item::Formula(f) if f.name == token.text => {
                            return Some(HoverInfo {
                                contents: format!("**Formula** `{}`", f.name),
                                start: token.span.start,
                                end: token.span.end,
                            });
                        }
                        _ => {}
                    }
                }
            }
            return None;
        }
        _ => return None,
    };

    Some(HoverInfo {
        contents,
        start: token.span.start,
        end: token.span.end,
    })
}

/// Solve a formula and return the solution as JSON
#[wasm_bindgen]
pub fn solve(source: &str, formula_name: &str) -> Result<JsValue, JsValue> {
    // Parse
    let program = Parser::parse(source).map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Compile
    let mut compiler = Compiler::new();
    compiler
        .load(&program)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let compiled = compiler
        .compile_formula(formula_name)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Solve
    let solver = Solver::new();
    let solution = solver.solve(&compiled.lp_problem);

    // Calculate ingredient results with costs
    let mut ingredients_result: Vec<IngredientResult> = Vec::new();
    for (i, ing_id) in compiled.ingredients.iter().enumerate() {
        let amount = solution.values.get(i).copied().unwrap_or(0.0);
        if amount > 0.001 {
            let unit_cost = compiled.ingredient_costs.get(i).copied().unwrap_or(0.0);
            let cost = amount * unit_cost;

            // Look up display name and code from symbol table
            let ing_meta = compiler.symbols.ingredients.get(ing_id);

            ingredients_result.push(IngredientResult {
                id: ing_id.clone(),
                name: ing_meta.and_then(|m| m.display_name.clone()),
                code: ing_meta.and_then(|m| m.code.clone()),
                amount,
                percentage: amount / compiled.batch_size * 100.0,
                unit_cost,
                cost,
                cost_percentage: if solution.objective_value > 0.0 {
                    cost / solution.objective_value * 100.0
                } else {
                    0.0
                },
            });
        }
    }

    // Calculate nutrient values achieved
    let mut nutrients_result: Vec<NutrientResult> = Vec::new();
    for (i, nut_id) in compiled.nutrient_names.iter().enumerate() {
        let mut total_value = 0.0;
        for (j, amount) in solution.values.iter().enumerate() {
            if *amount > 0.001 {
                if let Some(ing_nuts) = compiled.ingredient_nutrients.get(j) {
                    if let Some(nut_value) = ing_nuts.get(nut_id) {
                        // nutrient value is per 100 units, so scale by amount/100
                        total_value += nut_value * amount / 100.0;
                    }
                }
            }
        }
        // Convert to percentage of batch
        let value_pct = total_value / compiled.batch_size * 100.0;

        // Look up display name and code from symbol table
        let nut_meta = compiler.symbols.nutrients.get(nut_id);

        nutrients_result.push(NutrientResult {
            id: nut_id.clone(),
            name: nut_meta.and_then(|m| m.display_name.clone()),
            code: nut_meta.and_then(|m| m.code.clone()),
            value: value_pct,
            unit: compiled.nutrient_units.get(i).cloned().flatten(),
        });
    }

    // Sort nutrients alphabetically by id
    nutrients_result.sort_by(|a, b| a.id.cmp(&b.id));

    // Build violations list
    let violations_result: Vec<ViolationResult> = solution
        .violations
        .iter()
        .map(|v| ViolationResult {
            constraint: v.constraint.clone(),
            required: v.required,
            actual: v.actual,
            violation_amount: v.violation_amount,
            description: v.description.clone(),
        })
        .collect();

    // Build result
    let result = SolveResult {
        status: match solution.status {
            SolutionStatus::Optimal => "optimal".to_string(),
            SolutionStatus::Infeasible => "infeasible".to_string(),
            SolutionStatus::Unbounded => "unbounded".to_string(),
            SolutionStatus::Error => "error".to_string(),
        },
        formula: compiled.display_name.unwrap_or(compiled.name),
        description: compiled.description,
        batch_size: compiled.batch_size,
        total_cost: if solution.objective_value.is_finite() {
            solution.objective_value
        } else {
            0.0
        },
        ingredients: ingredients_result,
        nutrients: nutrients_result,
        analysis: if solution.status == SolutionStatus::Optimal {
            Some(AnalysisResult {
                binding_constraints: solution.analysis.binding_constraints.clone(),
                shadow_prices: solution
                    .analysis
                    .shadow_prices
                    .iter()
                    .map(|sp| ShadowPriceResult {
                        constraint: sp.constraint.clone(),
                        value: sp.value,
                        interpretation: sp.interpretation.clone(),
                    })
                    .collect(),
            })
        } else {
            None
        },
        violations: violations_result,
    };

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[derive(serde::Serialize)]
struct SolveResult {
    status: String,
    formula: String,
    description: Option<String>,
    batch_size: f64,
    total_cost: f64,
    ingredients: Vec<IngredientResult>,
    nutrients: Vec<NutrientResult>,
    analysis: Option<AnalysisResult>,
    violations: Vec<ViolationResult>,
}

#[derive(serde::Serialize)]
struct IngredientResult {
    id: String,
    name: Option<String>,
    code: Option<String>,
    amount: f64,
    percentage: f64,
    unit_cost: f64,
    cost: f64,
    cost_percentage: f64,
}

#[derive(serde::Serialize)]
struct NutrientResult {
    id: String,
    name: Option<String>,
    code: Option<String>,
    value: f64,
    unit: Option<String>,
}

#[derive(serde::Serialize)]
struct AnalysisResult {
    binding_constraints: Vec<String>,
    shadow_prices: Vec<ShadowPriceResult>,
}

#[derive(serde::Serialize)]
struct ShadowPriceResult {
    constraint: String,
    value: f64,
    interpretation: String,
}

#[derive(serde::Serialize)]
struct ViolationResult {
    constraint: String,
    required: f64,
    actual: f64,
    violation_amount: f64,
    description: String,
}
