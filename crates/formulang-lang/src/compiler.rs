use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use formulang_solver::{ConstraintOp, LpProblem};
use thiserror::Error;

use crate::ast::*;
use crate::Parser;

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Unknown nutrient: {0}")]
    UnknownNutrient(String),
    #[error("Unknown ingredient: {0}")]
    UnknownIngredient(String),
    #[error("Unknown formula: {0}")]
    UnknownFormula(String),
    #[error("Missing batch_size in formula {0}")]
    MissingBatchSize(String),
    #[error("Missing cost in ingredient {0}")]
    MissingCost(String),
    #[error("Circular reference detected: {0}")]
    CircularReference(String),
    #[error("Percentage not allowed in nutrient constraints")]
    PercentInNutrientConstraint,
    #[error("Invalid reference: {0}")]
    InvalidReference(String),
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Parse error in {0}: {1}")]
    ParseError(String, String),
    #[error("Import cycle detected: {0}")]
    ImportCycle(String),
}

/// Compiled representation of a nutrient
#[derive(Debug, Clone)]
pub struct CompiledNutrient {
    pub name: String,
    pub display_name: Option<String>,
    pub code: Option<String>,
    pub unit: Option<String>,
}

/// Compiled representation of an ingredient
#[derive(Debug, Clone)]
pub struct CompiledIngredient {
    pub name: String,
    pub display_name: Option<String>,
    pub code: Option<String>,
    pub cost: f64,
    pub nutrients: HashMap<String, f64>,
}

/// Compiled representation of a formula ready for solving
#[derive(Debug, Clone)]
pub struct CompiledFormula {
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub batch_size: f64,
    pub ingredients: Vec<String>,
    pub ingredient_costs: Vec<f64>,
    pub ingredient_nutrients: Vec<HashMap<String, f64>>,
    pub nutrient_names: Vec<String>,
    pub nutrient_units: Vec<Option<String>>,
    pub lp_problem: LpProblem,
}

/// Symbol table for resolving references
#[derive(Debug, Default, Clone)]
pub struct SymbolTable {
    pub nutrients: HashMap<String, CompiledNutrient>,
    pub ingredients: HashMap<String, CompiledIngredient>,
    pub formulas: HashMap<String, Formula>,
    /// Resolved nutrient constraints from base formulas
    pub nutrient_constraints: HashMap<String, Vec<NutrientConstraint>>,
    /// Resolved ingredient constraints from base formulas
    pub ingredient_constraints: HashMap<String, Vec<IngredientConstraint>>,
}

/// Compiler for converting AST to LP problems
pub struct Compiler {
    /// Symbol table with all nutrients, ingredients, and formulas
    pub symbols: SymbolTable,
    /// Base directory for resolving imports
    base_dir: Option<PathBuf>,
    /// Track loaded files to prevent cycles
    loaded_files: HashSet<PathBuf>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            symbols: SymbolTable::default(),
            base_dir: None,
            loaded_files: HashSet::new(),
        }
    }

    /// Create a compiler with a base directory for imports
    pub fn with_base_dir(base_dir: impl AsRef<Path>) -> Self {
        Self {
            symbols: SymbolTable::default(),
            base_dir: Some(base_dir.as_ref().to_path_buf()),
            loaded_files: HashSet::new(),
        }
    }

    /// Load a file and all its imports
    pub fn load_file(&mut self, path: impl AsRef<Path>) -> Result<(), CompileError> {
        let path = path.as_ref();
        let canonical = path.canonicalize().map_err(|e| {
            CompileError::IoError(format!("Cannot resolve path {}: {}", path.display(), e))
        })?;

        if self.loaded_files.contains(&canonical) {
            return Ok(()); // Already loaded
        }
        self.loaded_files.insert(canonical.clone());

        // Set base dir if not set
        if self.base_dir.is_none() {
            self.base_dir = canonical.parent().map(|p| p.to_path_buf());
        }

        let source = std::fs::read_to_string(path)
            .map_err(|e| CompileError::IoError(format!("{}: {}", path.display(), e)))?;

        let program = Parser::parse(&source)
            .map_err(|e| CompileError::ParseError(path.display().to_string(), e.to_string()))?;

        self.load_with_base(&program, canonical.parent())?;

        Ok(())
    }

    /// Load a program with a specific base directory for imports
    fn load_with_base(&mut self, program: &Program, base: Option<&Path>) -> Result<(), CompileError> {
        // First, process imports
        for item in &program.items {
            if let Item::Import(import) = item {
                self.process_import(import, base)?;
            }
        }

        // Then load declarations
        self.load(program)
    }

    fn process_import(&mut self, import: &Import, base: Option<&Path>) -> Result<(), CompileError> {
        let import_path = self.resolve_import_path(&import.path, base)?;

        if self.loaded_files.contains(&import_path) {
            return Ok(()); // Already loaded, skip
        }

        self.load_file(&import_path)?;

        Ok(())
    }

    fn resolve_import_path(&self, path: &str, base: Option<&Path>) -> Result<PathBuf, CompileError> {
        let base = base
            .or(self.base_dir.as_deref())
            .ok_or_else(|| CompileError::IoError("No base directory for import".to_string()))?;

        let import_path = if path.starts_with("./") || path.starts_with("../") {
            base.join(path)
        } else {
            base.join(path)
        };

        import_path.canonicalize().map_err(|e| {
            CompileError::IoError(format!("Cannot resolve import {}: {}", path, e))
        })
    }

    /// Load a program into the symbol table
    pub fn load(&mut self, program: &Program) -> Result<(), CompileError> {
        for item in &program.items {
            match item {
                Item::Nutrient(n) => {
                    self.symbols.nutrients.insert(
                        n.name.clone(),
                        CompiledNutrient {
                            name: n.name.clone(),
                            display_name: get_string_property(&n.properties, "name"),
                            code: get_string_property(&n.properties, "code"),
                            unit: get_string_property(&n.properties, "unit"),
                        },
                    );
                }
                Item::Ingredient(i) => {
                    let cost = get_number_property(&i.properties, "cost")
                        .ok_or_else(|| CompileError::MissingCost(i.name.clone()))?;

                    let mut nutrients = HashMap::new();
                    for nv in &i.nutrients {
                        let nutrient_name = reference_to_string(&nv.nutrient);
                        nutrients.insert(nutrient_name, nv.value);
                    }

                    self.symbols.ingredients.insert(
                        i.name.clone(),
                        CompiledIngredient {
                            name: i.name.clone(),
                            display_name: get_string_property(&i.properties, "name"),
                            code: get_string_property(&i.properties, "code"),
                            cost,
                            nutrients,
                        },
                    );
                }
                Item::Formula(f) => {
                    self.symbols.formulas.insert(f.name.clone(), f.clone());
                }
                Item::Import(_) => {
                    // Already processed in load_with_base
                }
            }
        }
        Ok(())
    }

    /// Compile a formula by name into an LP problem
    pub fn compile_formula(&self, name: &str) -> Result<CompiledFormula, CompileError> {
        let formula = self
            .symbols
            .formulas
            .get(name)
            .ok_or_else(|| CompileError::UnknownFormula(name.to_string()))?
            .clone();

        let batch_size = get_number_property(&formula.properties, "batch_size")
            .ok_or_else(|| CompileError::MissingBatchSize(name.to_string()))?;

        // Resolve all nutrient constraints (including from base formulas)
        let resolved_nutrients = self.resolve_nutrient_constraints(&formula.nutrients)?;

        // Resolve all ingredient constraints (including from base formulas)
        let resolved_ingredients = self.resolve_ingredient_constraints(&formula.ingredients)?;

        // Collect all ingredients used in this formula
        let mut ingredient_names: Vec<String> = Vec::new();
        for ic in &resolved_ingredients {
            self.collect_ingredients_from_expr(&ic.expr, &mut ingredient_names)?;
        }

        // Remove duplicates while preserving order
        let mut seen = std::collections::HashSet::new();
        ingredient_names.retain(|x| seen.insert(x.clone()));

        // Build LP problem
        let mut lp = LpProblem::new(ingredient_names.clone());

        // Collect ingredient costs and nutrient data
        let ingredient_costs: Vec<f64> = ingredient_names
            .iter()
            .map(|name| {
                self.symbols
                    .ingredients
                    .get(name)
                    .map(|i| i.cost)
                    .unwrap_or(0.0)
            })
            .collect();

        let ingredient_nutrients: Vec<HashMap<String, f64>> = ingredient_names
            .iter()
            .map(|name| {
                self.symbols
                    .ingredients
                    .get(name)
                    .map(|i| i.nutrients.clone())
                    .unwrap_or_default()
            })
            .collect();

        // Collect all unique nutrients used
        let mut nutrient_set: std::collections::HashSet<String> = std::collections::HashSet::new();
        for ing_nuts in &ingredient_nutrients {
            for nut in ing_nuts.keys() {
                nutrient_set.insert(nut.clone());
            }
        }
        let nutrient_names: Vec<String> = nutrient_set.into_iter().collect();

        let nutrient_units: Vec<Option<String>> = nutrient_names
            .iter()
            .map(|name| {
                self.symbols
                    .nutrients
                    .get(name)
                    .and_then(|n| n.unit.clone())
            })
            .collect();

        // Set objective: minimize cost
        lp.set_objective(ingredient_costs.clone(), true);

        // Add nutrient constraints
        for nc in &resolved_nutrients {
            self.add_nutrient_constraint(&mut lp, nc, &ingredient_names, batch_size)?;
        }

        // Add ingredient constraints
        for ic in &resolved_ingredients {
            self.add_ingredient_constraint(&mut lp, ic, &ingredient_names, batch_size)?;
        }

        // Add batch size constraint: sum of all ingredients = batch_size
        let ones = vec![1.0; ingredient_names.len()];
        lp.add_constraint("batch_size", ones, ConstraintOp::Eq, batch_size);

        // Add non-negativity (implicit in most solvers, but explicit here)
        for (i, name) in ingredient_names.iter().enumerate() {
            let mut coeffs = vec![0.0; ingredient_names.len()];
            coeffs[i] = 1.0;
            lp.add_constraint(format!("{}_nonneg", name), coeffs, ConstraintOp::Ge, 0.0);
        }

        Ok(CompiledFormula {
            name: formula.name.clone(),
            display_name: get_string_property(&formula.properties, "name"),
            description: get_string_property(&formula.properties, "description"),
            batch_size,
            ingredients: ingredient_names,
            ingredient_costs,
            ingredient_nutrients,
            nutrient_names,
            nutrient_units,
            lp_problem: lp,
        })
    }

    /// Resolve nutrient constraints, expanding base formula references
    fn resolve_nutrient_constraints(
        &self,
        constraints: &[NutrientConstraint],
    ) -> Result<Vec<NutrientConstraint>, CompileError> {
        let mut resolved = Vec::new();
        let mut overrides: HashMap<String, NutrientConstraint> = HashMap::new();

        for nc in constraints {
            if let Some(base_ref) = self.get_base_reference(&nc.expr) {
                // This is a reference like `poultry_base.nutrients`
                let base_constraints = self.resolve_base_nutrient_reference(&base_ref)?;
                for bc in base_constraints {
                    let key = self.constraint_key(&bc.expr);
                    if !overrides.contains_key(&key) {
                        resolved.push(bc);
                    }
                }
            } else {
                // Regular constraint - may override base
                let key = self.constraint_key(&nc.expr);
                overrides.insert(key.clone(), nc.clone());
                resolved.push(nc.clone());
            }
        }

        // Apply overrides (replace matching constraints)
        for constraint in &mut resolved {
            let key = self.constraint_key(&constraint.expr);
            if let Some(override_c) = overrides.get(&key) {
                // Merge bounds: override wins
                if override_c.bounds.min.is_some() || override_c.bounds.max.is_some() {
                    *constraint = override_c.clone();
                }
            }
        }

        Ok(resolved)
    }

    /// Resolve ingredient constraints, expanding base formula references
    fn resolve_ingredient_constraints(
        &self,
        constraints: &[IngredientConstraint],
    ) -> Result<Vec<IngredientConstraint>, CompileError> {
        let mut resolved = Vec::new();
        let mut overrides: HashMap<String, IngredientConstraint> = HashMap::new();

        for ic in constraints {
            if let Some(base_ref) = self.get_base_reference(&ic.expr) {
                // This is a reference like `starter.ingredients`
                let base_constraints = self.resolve_base_ingredient_reference(&base_ref)?;
                for bc in base_constraints {
                    let key = self.constraint_key(&bc.expr);
                    if !overrides.contains_key(&key) {
                        resolved.push(bc);
                    }
                }
            } else {
                // Regular constraint - may override base
                let key = self.constraint_key(&ic.expr);
                overrides.insert(key.clone(), ic.clone());
                resolved.push(ic.clone());
            }
        }

        // Apply overrides
        for constraint in &mut resolved {
            let key = self.constraint_key(&constraint.expr);
            if let Some(override_c) = overrides.get(&key) {
                if override_c.bounds.min.is_some() || override_c.bounds.max.is_some() {
                    *constraint = override_c.clone();
                }
            }
        }

        Ok(resolved)
    }

    /// Check if an expression is a base formula reference (e.g., `base.nutrients`)
    fn get_base_reference(&self, expr: &Expr) -> Option<(String, String)> {
        if let Expr::Reference(r) = expr {
            if r.parts.len() >= 2 {
                if let (ReferencePart::Ident(formula), ReferencePart::Ident(block)) =
                    (&r.parts[0], &r.parts[1])
                {
                    if block == "nutrients" || block == "ingredients" {
                        return Some((formula.clone(), block.clone()));
                    }
                }
            }
        }
        None
    }

    /// Resolve a base formula's nutrient constraints
    fn resolve_base_nutrient_reference(
        &self,
        base_ref: &(String, String),
    ) -> Result<Vec<NutrientConstraint>, CompileError> {
        let (formula_name, _) = base_ref;
        let formula = self
            .symbols
            .formulas
            .get(formula_name)
            .ok_or_else(|| CompileError::UnknownFormula(formula_name.clone()))?;

        // Recursively resolve (to support chained inheritance)
        self.resolve_nutrient_constraints(&formula.nutrients)
    }

    /// Resolve a base formula's ingredient constraints
    fn resolve_base_ingredient_reference(
        &self,
        base_ref: &(String, String),
    ) -> Result<Vec<IngredientConstraint>, CompileError> {
        let (formula_name, _) = base_ref;
        let formula = self
            .symbols
            .formulas
            .get(formula_name)
            .ok_or_else(|| CompileError::UnknownFormula(formula_name.clone()))?;

        self.resolve_ingredient_constraints(&formula.ingredients)
    }

    /// Get a unique key for a constraint (for override matching)
    fn constraint_key(&self, expr: &Expr) -> String {
        match expr {
            Expr::Reference(r) => reference_to_string(r),
            Expr::BinaryOp { left, op, right } => {
                format!(
                    "{}{}{}",
                    self.constraint_key(left),
                    op,
                    self.constraint_key(right)
                )
            }
            Expr::Paren(inner) => self.constraint_key(inner),
            Expr::Number(n) => n.to_string(),
        }
    }

    fn collect_ingredients_from_expr(
        &self,
        expr: &Expr,
        ingredients: &mut Vec<String>,
    ) -> Result<(), CompileError> {
        match expr {
            Expr::Number(_) => {}
            Expr::Reference(r) => {
                if let Some(ReferencePart::Ident(name)) = r.parts.first() {
                    // Check if it's an ingredient
                    if self.symbols.ingredients.contains_key(name) {
                        ingredients.push(name.clone());
                    }
                    // Could also be a base formula reference - TODO
                }
            }
            Expr::BinaryOp { left, right, .. } => {
                self.collect_ingredients_from_expr(left, ingredients)?;
                self.collect_ingredients_from_expr(right, ingredients)?;
            }
            Expr::Paren(inner) => {
                self.collect_ingredients_from_expr(inner, ingredients)?;
            }
        }
        Ok(())
    }

    fn add_nutrient_constraint(
        &self,
        lp: &mut LpProblem,
        constraint: &NutrientConstraint,
        ingredients: &[String],
        batch_size: f64,
    ) -> Result<(), CompileError> {
        // Check if this is a ratio constraint (e.g., calcium / phosphorus)
        if let Expr::BinaryOp { left, op: BinaryOp::Div, right } = &constraint.expr {
            return self.add_ratio_constraint(lp, left, right, &constraint.bounds, ingredients, batch_size);
        }

        // For simple nutrient constraints
        let nutrient_name = self.expr_to_nutrient_name(&constraint.expr)?;

        // Build coefficient vector: each ingredient's contribution to this nutrient
        let coeffs: Vec<f64> = ingredients
            .iter()
            .map(|ing_name| {
                self.symbols
                    .ingredients
                    .get(ing_name)
                    .and_then(|ing| ing.nutrients.get(&nutrient_name))
                    .copied()
                    .unwrap_or(0.0)
            })
            .collect();

        // Add min constraint if present
        if let Some(ref min_bound) = constraint.bounds.min {
            if min_bound.is_percent {
                return Err(CompileError::PercentInNutrientConstraint);
            }
            // Nutrient values in ingredients are percentages (e.g., protein: 8 means 8%)
            // Constraint protein min 20 means: final formula should have >= 20% protein
            // Formula: sum(amount_i * nutrient_pct_i) / batch_size >= required_pct
            // Rearranged: sum(amount_i * nutrient_pct_i) >= required_pct * batch_size
            let rhs = min_bound.value * batch_size;
            lp.add_constraint(
                format!("{}_min", nutrient_name),
                coeffs.clone(),
                ConstraintOp::Ge,
                rhs,
            );
        }

        // Add max constraint if present
        if let Some(ref max_bound) = constraint.bounds.max {
            if max_bound.is_percent {
                return Err(CompileError::PercentInNutrientConstraint);
            }
            let rhs = max_bound.value * batch_size;
            lp.add_constraint(
                format!("{}_max", nutrient_name),
                coeffs,
                ConstraintOp::Le,
                rhs,
            );
        }

        Ok(())
    }

    /// Add a ratio constraint like `calcium / phosphorus min 1.5 max 2.0`
    /// This is linearized as:
    /// - For min R: calcium >= R * phosphorus => calcium - R*phosphorus >= 0
    /// - For max R: calcium <= R * phosphorus => calcium - R*phosphorus <= 0
    fn add_ratio_constraint(
        &self,
        lp: &mut LpProblem,
        numerator: &Expr,
        denominator: &Expr,
        bounds: &Bounds,
        ingredients: &[String],
        _batch_size: f64,
    ) -> Result<(), CompileError> {
        let num_name = self.expr_to_nutrient_name(numerator)?;
        let den_name = self.expr_to_nutrient_name(denominator)?;

        // Get nutrient coefficients for numerator and denominator
        let num_coeffs: Vec<f64> = ingredients
            .iter()
            .map(|ing_name| {
                self.symbols
                    .ingredients
                    .get(ing_name)
                    .and_then(|ing| ing.nutrients.get(&num_name))
                    .copied()
                    .unwrap_or(0.0)
            })
            .collect();

        let den_coeffs: Vec<f64> = ingredients
            .iter()
            .map(|ing_name| {
                self.symbols
                    .ingredients
                    .get(ing_name)
                    .and_then(|ing| ing.nutrients.get(&den_name))
                    .copied()
                    .unwrap_or(0.0)
            })
            .collect();

        // For min constraint: num/den >= R => num - R*den >= 0
        if let Some(ref min_bound) = bounds.min {
            let r = min_bound.value;
            let coeffs: Vec<f64> = num_coeffs
                .iter()
                .zip(den_coeffs.iter())
                .map(|(n, d)| n - r * d)
                .collect();
            lp.add_constraint(
                format!("{}/{}_min", num_name, den_name),
                coeffs,
                ConstraintOp::Ge,
                0.0,
            );
        }

        // For max constraint: num/den <= R => num - R*den <= 0
        if let Some(ref max_bound) = bounds.max {
            let r = max_bound.value;
            let coeffs: Vec<f64> = num_coeffs
                .iter()
                .zip(den_coeffs.iter())
                .map(|(n, d)| n - r * d)
                .collect();
            lp.add_constraint(
                format!("{}/{}_max", num_name, den_name),
                coeffs,
                ConstraintOp::Le,
                0.0,
            );
        }

        Ok(())
    }

    fn add_ingredient_constraint(
        &self,
        lp: &mut LpProblem,
        constraint: &IngredientConstraint,
        ingredients: &[String],
        batch_size: f64,
    ) -> Result<(), CompileError> {
        // Build coefficient vector from expression
        let coeffs = self.expr_to_ingredient_coeffs(&constraint.expr, ingredients)?;

        // Add min constraint if present
        if let Some(ref min_bound) = constraint.bounds.min {
            let rhs = if min_bound.is_percent {
                min_bound.value * batch_size / 100.0
            } else {
                min_bound.value
            };

            let constraint_name = format!("{}_min", self.expr_to_name(&constraint.expr));
            lp.add_constraint(constraint_name, coeffs.clone(), ConstraintOp::Ge, rhs);
        }

        // Add max constraint if present
        if let Some(ref max_bound) = constraint.bounds.max {
            let rhs = if max_bound.is_percent {
                max_bound.value * batch_size / 100.0
            } else {
                max_bound.value
            };

            let constraint_name = format!("{}_max", self.expr_to_name(&constraint.expr));
            lp.add_constraint(constraint_name, coeffs, ConstraintOp::Le, rhs);
        }

        Ok(())
    }

    fn expr_to_nutrient_name(&self, expr: &Expr) -> Result<String, CompileError> {
        match expr {
            Expr::Reference(r) => {
                // Simple case: just a nutrient name
                if r.parts.len() == 1 {
                    if let ReferencePart::Ident(name) = &r.parts[0] {
                        return Ok(name.clone());
                    }
                }
                // TODO: Handle ratio expressions like calcium / phosphorus
                Ok(reference_to_string(r))
            }
            Expr::BinaryOp { left, op, right } => {
                // For ratio constraints like calcium / phosphorus
                let left_name = self.expr_to_nutrient_name(left)?;
                let right_name = self.expr_to_nutrient_name(right)?;
                Ok(format!("{}_{:?}_{}", left_name, op, right_name))
            }
            _ => Err(CompileError::InvalidReference(
                "Expected nutrient reference".to_string(),
            )),
        }
    }

    fn expr_to_ingredient_coeffs(
        &self,
        expr: &Expr,
        ingredients: &[String],
    ) -> Result<Vec<f64>, CompileError> {
        let mut coeffs = vec![0.0; ingredients.len()];

        match expr {
            Expr::Reference(r) => {
                if let Some(ReferencePart::Ident(name)) = r.parts.first() {
                    if let Some(idx) = ingredients.iter().position(|x| x == name) {
                        coeffs[idx] = 1.0;
                    }
                }
            }
            Expr::BinaryOp { left, op, right } => {
                let left_coeffs = self.expr_to_ingredient_coeffs(left, ingredients)?;
                let right_coeffs = self.expr_to_ingredient_coeffs(right, ingredients)?;

                match op {
                    BinaryOp::Add => {
                        for i in 0..coeffs.len() {
                            coeffs[i] = left_coeffs[i] + right_coeffs[i];
                        }
                    }
                    BinaryOp::Sub => {
                        for i in 0..coeffs.len() {
                            coeffs[i] = left_coeffs[i] - right_coeffs[i];
                        }
                    }
                    _ => {
                        return Err(CompileError::InvalidReference(
                            "Only + and - allowed in ingredient expressions".to_string(),
                        ));
                    }
                }
            }
            Expr::Paren(inner) => {
                return self.expr_to_ingredient_coeffs(inner, ingredients);
            }
            Expr::Number(_) => {
                // Numbers in ingredient constraints are unusual but allowed
            }
        }

        Ok(coeffs)
    }

    fn expr_to_name(&self, expr: &Expr) -> String {
        match expr {
            Expr::Reference(r) => reference_to_string(r),
            Expr::BinaryOp { left, op, right } => {
                format!(
                    "{}{:?}{}",
                    self.expr_to_name(left),
                    op,
                    self.expr_to_name(right)
                )
            }
            Expr::Paren(inner) => self.expr_to_name(inner),
            Expr::Number(n) => n.to_string(),
        }
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions

/// Property name aliases for shorter syntax
fn property_matches(property_name: &str, target: &str) -> bool {
    if property_name == target {
        return true;
    }
    // Check aliases
    match (property_name, target) {
        ("batch", "batch_size") => true,
        ("desc", "description") => true,
        _ => false,
    }
}

fn get_string_property(properties: &[Property], name: &str) -> Option<String> {
    properties.iter().find_map(|p| {
        if property_matches(&p.name, name) {
            match &p.value {
                PropertyValue::String(s) => Some(s.clone()),
                PropertyValue::Ident(s) => Some(s.clone()),
                _ => None,
            }
        } else {
            None
        }
    })
}

fn get_number_property(properties: &[Property], name: &str) -> Option<f64> {
    properties.iter().find_map(|p| {
        if property_matches(&p.name, name) {
            match &p.value {
                PropertyValue::Number(n) => Some(*n),
                _ => None,
            }
        } else {
            None
        }
    })
}

fn reference_to_string(r: &Reference) -> String {
    r.parts
        .iter()
        .filter_map(|p| match p {
            ReferencePart::Ident(s) => Some(s.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(".")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    #[test]
    fn test_compile_simple_formula() {
        let source = r#"
            nutrient protein {}
            nutrient energy {}

            ingredient corn {
                cost 150
                nutrients {
                    protein 8.5
                    energy 3350
                }
            }

            ingredient soybean_meal {
                cost 450
                nutrients {
                    protein 48.0
                    energy 2230
                }
            }

            formula starter {
                batch_size 1000

                nutrients {
                    protein min 18 max 24
                    energy min 2800
                }

                ingredients {
                    corn max 60%
                    soybean_meal min 10%
                }
            }
        "#;

        let program = Parser::parse(source).unwrap();
        let mut compiler = Compiler::new();
        compiler.load(&program).unwrap();

        let compiled = compiler.compile_formula("starter").unwrap();

        assert_eq!(compiled.name, "starter");
        assert_eq!(compiled.batch_size, 1000.0);
        assert_eq!(compiled.ingredients.len(), 2);
        assert!(compiled.ingredients.contains(&"corn".to_string()));
        assert!(compiled.ingredients.contains(&"soybean_meal".to_string()));

        // Check LP problem structure
        assert_eq!(compiled.lp_problem.num_variables(), 2);
        // Constraints: protein_min, protein_max, energy_min, corn_max, soybean_meal_min,
        //              batch_size, corn_min (non-neg), soybean_meal_min (non-neg)
        assert!(compiled.lp_problem.num_constraints() >= 5);
    }

    #[test]
    fn test_compile_and_solve() {
        let source = r#"
            nutrient protein {}

            ingredient corn {
                cost 100
                nutrients { protein 8.0 }
            }

            ingredient soy {
                cost 300
                nutrients { protein 45.0 }
            }

            formula test {
                batch_size 100

                nutrients {
                    protein min 20
                }

                ingredients {
                    corn
                    soy
                }
            }
        "#;

        let program = Parser::parse(source).unwrap();
        let mut compiler = Compiler::new();
        compiler.load(&program).unwrap();

        let compiled = compiler.compile_formula("test").unwrap();

        // Solve it
        let solver = formulang_solver::Solver::new();
        let solution = solver.solve(&compiled.lp_problem);

        assert_eq!(solution.status, formulang_solver::SolutionStatus::Optimal);

        // Should find a solution where protein >= 20%
        let corn = solution.values[0];
        let soy = solution.values[1];

        // Verify batch size
        assert!((corn + soy - 100.0).abs() < 1e-6, "Batch size should be 100");

        // Verify protein constraint: (corn * 8 + soy * 45) / 100 >= 20
        let protein = (corn * 8.0 + soy * 45.0) / 100.0;
        assert!(protein >= 20.0 - 1e-6, "Protein should be >= 20%, got {}", protein);
    }

    #[test]
    fn test_property_aliases() {
        let source = r#"
            nutrient protein {}

            ingredient corn {
                cost 100
                nuts { protein 8.0 }
            }

            formula test {
                batch 100
                desc "Test formula using short aliases"

                nuts {
                    protein min 5
                }

                ings {
                    corn
                }
            }
        "#;

        let program = Parser::parse(source).unwrap();
        let mut compiler = Compiler::new();
        compiler.load(&program).unwrap();

        let compiled = compiler.compile_formula("test").unwrap();

        // Verify that 'batch' maps to batch_size
        assert_eq!(compiled.batch_size, 100.0);
        // Verify that 'desc' maps to description
        assert_eq!(compiled.description, Some("Test formula using short aliases".to_string()));
    }
}
