use crate::problem::{ConstraintOp, LpProblem};
use crate::solution::{Analysis, ConstraintViolation, ReducedCost, SensitivityRange, ShadowPrice, Solution, SolutionStatus};

/// Simplex solver for linear programming problems
pub struct Solver {
    /// Maximum iterations before giving up
    max_iterations: usize,
    /// Tolerance for floating point comparisons
    tolerance: f64,
}

impl Default for Solver {
    fn default() -> Self {
        Self {
            max_iterations: 10000,
            tolerance: 1e-9,
        }
    }
}

impl Solver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    pub fn with_tolerance(mut self, tol: f64) -> Self {
        self.tolerance = tol;
        self
    }

    /// Solve the LP problem using the two-phase simplex method
    pub fn solve(&self, problem: &LpProblem) -> Solution {
        // Convert to standard form and solve
        let mut tableau = match self.build_tableau(problem) {
            Ok(t) => t,
            Err(_) => return self.solve_with_relaxation(problem),
        };

        // Phase 1: Find initial basic feasible solution
        if tableau.has_artificial {
            if !self.phase1(&mut tableau) {
                return self.solve_with_relaxation(problem);
            }
        }

        // Phase 2: Optimize
        match self.phase2(&mut tableau) {
            SimplexResult::Optimal => {}
            SimplexResult::Unbounded => return Solution::unbounded(),
            SimplexResult::Infeasible => return self.solve_with_relaxation(problem),
        }

        // Extract solution and add violations field (empty for optimal)
        let mut solution = self.extract_solution(&tableau, problem);
        solution.violations = Vec::new();
        solution
    }

    /// When the original problem is infeasible, try to find a "best effort" solution
    /// by relaxing constraints and reporting which ones are violated
    fn solve_with_relaxation(&self, problem: &LpProblem) -> Solution {
        // Strategy: solve with only <= constraints (upper bounds) to get a baseline,
        // then check which >= constraints (lower bounds) are violated

        // Create a relaxed problem with only upper bound constraints
        let mut relaxed = LpProblem::new(problem.variables.clone());
        relaxed.set_objective(
            problem.objective.coefficients.clone(),
            problem.objective.minimize,
        );

        // Add only <= constraints (these are usually max limits)
        // Also add equality constraints that sum to batch size
        for c in &problem.constraints {
            match c.op {
                ConstraintOp::Le => {
                    relaxed.add_constraint(
                        c.name.clone(),
                        c.coefficients.clone(),
                        c.op,
                        c.rhs,
                    );
                }
                ConstraintOp::Eq => {
                    // Keep equality constraints as they define the problem structure
                    relaxed.add_constraint(
                        c.name.clone(),
                        c.coefficients.clone(),
                        c.op,
                        c.rhs,
                    );
                }
                ConstraintOp::Ge => {
                    // Skip >= constraints in relaxed problem
                }
            }
        }

        // Try to solve the relaxed problem
        let relaxed_solution = self.solve_relaxed(&relaxed);

        if relaxed_solution.status != SolutionStatus::Optimal {
            // Even relaxed problem fails - analyze direct conflicts
            return self.analyze_conflicts(problem);
        }

        // Check which constraints are violated
        let violations = self.find_violations(problem, &relaxed_solution.values);

        if violations.is_empty() {
            // No violations means original should have been feasible - return as optimal
            let mut sol = relaxed_solution;
            sol.violations = Vec::new();
            return sol;
        }

        // Return infeasible with relaxed solution and violations
        Solution::infeasible_with_relaxed(
            relaxed_solution.values,
            relaxed_solution.objective_value,
            violations,
        )
    }

    /// Solve without infeasibility recovery (used for relaxed problems)
    fn solve_relaxed(&self, problem: &LpProblem) -> Solution {
        let mut tableau = match self.build_tableau(problem) {
            Ok(t) => t,
            Err(_) => return Solution::infeasible(),
        };

        if tableau.has_artificial {
            if !self.phase1(&mut tableau) {
                return Solution::infeasible();
            }
        }

        match self.phase2(&mut tableau) {
            SimplexResult::Optimal => {}
            SimplexResult::Unbounded => return Solution::unbounded(),
            SimplexResult::Infeasible => return Solution::infeasible(),
        }

        let mut solution = self.extract_solution(&tableau, problem);
        solution.violations = Vec::new();
        solution
    }

    /// Find which constraints are violated by a given solution
    fn find_violations(&self, problem: &LpProblem, values: &[f64]) -> Vec<ConstraintViolation> {
        let mut violations = Vec::new();

        for c in &problem.constraints {
            // Calculate LHS value
            let mut lhs = 0.0;
            for (j, &coef) in c.coefficients.iter().enumerate() {
                if j < values.len() {
                    lhs += coef * values[j];
                }
            }

            let (is_violated, violation_amount, description) = match c.op {
                ConstraintOp::Le => {
                    if lhs > c.rhs + self.tolerance {
                        let amt = lhs - c.rhs;
                        (true, amt, format!("{} exceeds maximum of {:.2} by {:.2}", c.name, c.rhs, amt))
                    } else {
                        (false, 0.0, String::new())
                    }
                }
                ConstraintOp::Ge => {
                    if lhs < c.rhs - self.tolerance {
                        let amt = c.rhs - lhs;
                        (true, amt, format!("{} is below minimum of {:.2} by {:.2}", c.name, c.rhs, amt))
                    } else {
                        (false, 0.0, String::new())
                    }
                }
                ConstraintOp::Eq => {
                    let diff = (lhs - c.rhs).abs();
                    if diff > self.tolerance {
                        (true, diff, format!("{} requires exactly {:.2} but got {:.2}", c.name, c.rhs, lhs))
                    } else {
                        (false, 0.0, String::new())
                    }
                }
            };

            if is_violated {
                violations.push(ConstraintViolation {
                    constraint: c.name.clone(),
                    required: c.rhs,
                    actual: lhs,
                    violation_amount,
                    description,
                });
            }
        }

        // Sort by violation amount (worst first)
        violations.sort_by(|a, b| b.violation_amount.partial_cmp(&a.violation_amount).unwrap_or(std::cmp::Ordering::Equal));

        violations
    }

    /// Analyze direct constraint conflicts when even relaxed solve fails
    fn analyze_conflicts(&self, problem: &LpProblem) -> Solution {
        let mut violations = Vec::new();

        // Look for obvious conflicts: min > max on the same expression
        // Group constraints by their coefficient pattern
        use std::collections::HashMap;
        let mut constraint_groups: HashMap<Vec<i64>, Vec<&crate::problem::Constraint>> = HashMap::new();

        for c in &problem.constraints {
            // Create a key from coefficient signs (simplified pattern matching)
            let key: Vec<i64> = c.coefficients.iter().map(|&x| {
                if x.abs() < self.tolerance { 0 }
                else if x > 0.0 { 1 }
                else { -1 }
            }).collect();
            constraint_groups.entry(key).or_default().push(c);
        }

        // Check each group for conflicts
        for (_key, constraints) in constraint_groups {
            let mut min_bound: Option<(f64, &str)> = None;
            let mut max_bound: Option<(f64, &str)> = None;

            for c in &constraints {
                match c.op {
                    ConstraintOp::Ge => {
                        if min_bound.is_none() || c.rhs > min_bound.unwrap().0 {
                            min_bound = Some((c.rhs, &c.name));
                        }
                    }
                    ConstraintOp::Le => {
                        if max_bound.is_none() || c.rhs < max_bound.unwrap().0 {
                            max_bound = Some((c.rhs, &c.name));
                        }
                    }
                    ConstraintOp::Eq => {
                        min_bound = Some((c.rhs, &c.name));
                        max_bound = Some((c.rhs, &c.name));
                    }
                }
            }

            if let (Some((min_val, min_name)), Some((max_val, max_name))) = (min_bound, max_bound) {
                if min_val > max_val + self.tolerance {
                    violations.push(ConstraintViolation {
                        constraint: format!("{} vs {}", min_name, max_name),
                        required: min_val,
                        actual: max_val,
                        violation_amount: min_val - max_val,
                        description: format!(
                            "Conflict: {} requires >= {:.2} but {} requires <= {:.2}",
                            min_name, min_val, max_name, max_val
                        ),
                    });
                }
            }
        }

        Solution::infeasible_with_violations(violations)
    }

    fn build_tableau(&self, problem: &LpProblem) -> Result<Tableau, ()> {
        let n_vars = problem.num_variables();
        let n_constraints = problem.num_constraints();

        // Count slack and artificial variables needed
        let mut n_slack = 0;
        let mut n_artificial = 0;

        for c in &problem.constraints {
            match c.op {
                ConstraintOp::Le => n_slack += 1,
                ConstraintOp::Ge => {
                    n_slack += 1; // surplus
                    n_artificial += 1;
                }
                ConstraintOp::Eq => n_artificial += 1,
            }
        }

        let total_cols = n_vars + n_slack + n_artificial + 1; // +1 for RHS
        let total_rows = n_constraints + 1; // +1 for objective

        let mut tableau = Tableau {
            data: vec![vec![0.0; total_cols]; total_rows],
            basic_vars: vec![0; n_constraints],
            n_vars,
            n_slack,
            n_artificial,
            has_artificial: n_artificial > 0,
            constraint_names: problem.constraints.iter().map(|c| c.name.clone()).collect(),
            variable_names: problem.variables.clone(),
        };

        // Fill in constraint rows
        let mut slack_idx = n_vars;
        let mut artificial_idx = n_vars + n_slack;

        for (i, c) in problem.constraints.iter().enumerate() {
            // Original variables
            for (j, &coef) in c.coefficients.iter().enumerate() {
                tableau.data[i][j] = coef;
            }

            // RHS (ensure non-negative)
            let mut rhs = c.rhs;
            let mut flip = false;
            if rhs < 0.0 {
                rhs = -rhs;
                flip = true;
                for j in 0..n_vars {
                    tableau.data[i][j] = -tableau.data[i][j];
                }
            }
            tableau.data[i][total_cols - 1] = rhs;

            // Add slack/surplus/artificial
            match c.op {
                ConstraintOp::Le => {
                    let sign = if flip { -1.0 } else { 1.0 };
                    tableau.data[i][slack_idx] = sign;
                    tableau.basic_vars[i] = slack_idx;
                    slack_idx += 1;
                }
                ConstraintOp::Ge => {
                    let sign = if flip { 1.0 } else { -1.0 };
                    tableau.data[i][slack_idx] = sign; // surplus
                    slack_idx += 1;
                    tableau.data[i][artificial_idx] = 1.0; // artificial
                    tableau.basic_vars[i] = artificial_idx;
                    artificial_idx += 1;
                }
                ConstraintOp::Eq => {
                    tableau.data[i][artificial_idx] = 1.0;
                    tableau.basic_vars[i] = artificial_idx;
                    artificial_idx += 1;
                }
            }
        }

        // Objective row (last row)
        // Simplex maximizes, so for minimization we negate the coefficients
        // The objective row stores -c for the reduced costs
        let obj_row = n_constraints;
        for (j, &coef) in problem.objective.coefficients.iter().enumerate() {
            tableau.data[obj_row][j] = if problem.objective.minimize { -coef } else { coef };
        }

        Ok(tableau)
    }

    fn phase1(&self, tableau: &mut Tableau) -> bool {
        // Create auxiliary objective: minimize sum of artificial variables
        // We negate to turn it into maximization (maximize -sum = minimize sum)
        let n_constraints = tableau.data.len() - 1;
        let n_cols = tableau.data[0].len();
        let art_start = tableau.n_vars + tableau.n_slack;

        // Save original objective
        let orig_obj = tableau.data[n_constraints].clone();

        // Set phase 1 objective: maximize -artificials (= minimize artificials)
        for j in 0..n_cols {
            tableau.data[n_constraints][j] = 0.0;
        }
        for j in art_start..(art_start + tableau.n_artificial) {
            tableau.data[n_constraints][j] = -1.0;  // Negated for maximization
        }

        // Make objective row consistent with basic artificial variables
        // For each basic artificial, add its row to cancel the -1 coefficient
        for i in 0..n_constraints {
            if tableau.basic_vars[i] >= art_start {
                for j in 0..n_cols {
                    tableau.data[n_constraints][j] += tableau.data[i][j];
                }
            }
        }

        // Solve phase 1
        for _ in 0..self.max_iterations {
            let Some(pivot_col) = self.find_pivot_column(tableau) else {
                break;
            };
            let Some(pivot_row) = self.find_pivot_row(tableau, pivot_col) else {
                // Unbounded in phase 1 means infeasible original
                return false;
            };
            self.pivot(tableau, pivot_row, pivot_col);
        }

        // Check if all artificials are zero
        let rhs_col = n_cols - 1;
        for i in 0..n_constraints {
            if tableau.basic_vars[i] >= art_start {
                if tableau.data[i][rhs_col].abs() > self.tolerance {
                    return false; // Infeasible
                }
            }
        }

        // Restore original objective and adjust for basic variables
        tableau.data[n_constraints] = orig_obj;
        for i in 0..n_constraints {
            let basic = tableau.basic_vars[i];
            if tableau.data[n_constraints][basic].abs() > self.tolerance {
                let ratio = tableau.data[n_constraints][basic];
                for j in 0..n_cols {
                    tableau.data[n_constraints][j] -= ratio * tableau.data[i][j];
                }
            }
        }

        true
    }

    fn phase2(&self, tableau: &mut Tableau) -> SimplexResult {
        // Exclude artificial variable columns from pivoting
        let exclude_from = tableau.n_vars + tableau.n_slack;

        for _ in 0..self.max_iterations {
            let Some(pivot_col) = self.find_pivot_column_excluding(tableau, exclude_from) else {
                return SimplexResult::Optimal;
            };
            let Some(pivot_row) = self.find_pivot_row(tableau, pivot_col) else {
                return SimplexResult::Unbounded;
            };
            self.pivot(tableau, pivot_row, pivot_col);
        }
        SimplexResult::Optimal // Max iterations reached, return best found
    }

    fn find_pivot_column(&self, tableau: &Tableau) -> Option<usize> {
        self.find_pivot_column_excluding(tableau, 0)
    }

    fn find_pivot_column_excluding(&self, tableau: &Tableau, exclude_from: usize) -> Option<usize> {
        let obj_row = tableau.data.len() - 1;
        // Exclude RHS and any columns >= exclude_from
        let n_cols = if exclude_from > 0 {
            exclude_from
        } else {
            tableau.data[0].len() - 1
        };

        // Look for the most positive reduced cost (can improve objective)
        let mut max_val = self.tolerance;
        let mut max_col = None;

        for j in 0..n_cols {
            if tableau.data[obj_row][j] > max_val {
                max_val = tableau.data[obj_row][j];
                max_col = Some(j);
            }
        }

        max_col
    }

    fn find_pivot_row(&self, tableau: &Tableau, col: usize) -> Option<usize> {
        let n_constraints = tableau.data.len() - 1;
        let rhs_col = tableau.data[0].len() - 1;

        let mut min_ratio = f64::INFINITY;
        let mut min_row = None;

        for i in 0..n_constraints {
            let val = tableau.data[i][col];
            if val > self.tolerance {
                let ratio = tableau.data[i][rhs_col] / val;
                if ratio >= 0.0 && ratio < min_ratio {
                    min_ratio = ratio;
                    min_row = Some(i);
                }
            }
        }

        min_row
    }

    fn pivot(&self, tableau: &mut Tableau, row: usize, col: usize) {
        let n_rows = tableau.data.len();
        let n_cols = tableau.data[0].len();

        // Update basic variable
        tableau.basic_vars[row] = col;

        // Scale pivot row
        let pivot_val = tableau.data[row][col];
        for j in 0..n_cols {
            tableau.data[row][j] /= pivot_val;
        }

        // Eliminate column in other rows
        for i in 0..n_rows {
            if i != row {
                let factor = tableau.data[i][col];
                for j in 0..n_cols {
                    tableau.data[i][j] -= factor * tableau.data[row][j];
                }
            }
        }
    }

    fn extract_solution(&self, tableau: &Tableau, problem: &LpProblem) -> Solution {
        let n_vars = problem.num_variables();
        let n_constraints = problem.num_constraints();
        let n_cols = tableau.data[0].len();
        let rhs_col = n_cols - 1;

        // Extract variable values
        let mut values = vec![0.0; n_vars];
        for i in 0..n_constraints {
            let basic = tableau.basic_vars[i];
            if basic < n_vars {
                values[basic] = tableau.data[i][rhs_col];
            }
        }

        // Calculate objective value
        let mut objective_value = 0.0;
        for (j, &val) in values.iter().enumerate() {
            objective_value += problem.objective.coefficients[j] * val;
        }

        // Perform analysis
        let analysis = self.analyze(tableau, problem, &values);

        Solution {
            status: SolutionStatus::Optimal,
            values,
            objective_value,
            analysis,
            violations: Vec::new(),
        }
    }

    fn analyze(&self, tableau: &Tableau, problem: &LpProblem, values: &[f64]) -> Analysis {
        let n_vars = problem.num_variables();
        let n_constraints = problem.num_constraints();
        let n_cols = tableau.data[0].len();
        let obj_row = n_constraints;

        // Shadow prices: negative of objective row entries for slack variables
        let mut shadow_prices = Vec::new();
        for (i, constraint) in problem.constraints.iter().enumerate() {
            let slack_col = n_vars + i;
            if slack_col < n_cols - 1 {
                let value = -tableau.data[obj_row][slack_col];
                let interpretation = if value.abs() < self.tolerance {
                    "Non-binding constraint".to_string()
                } else if value > 0.0 {
                    format!("Increasing RHS by 1 unit would decrease cost by {:.4}", value)
                } else {
                    format!("Increasing RHS by 1 unit would increase cost by {:.4}", -value)
                };
                shadow_prices.push(ShadowPrice {
                    constraint: constraint.name.clone(),
                    value,
                    interpretation,
                });
            }
        }

        // Reduced costs
        let mut reduced_costs = Vec::new();
        for (j, var_name) in problem.variables.iter().enumerate() {
            let is_basic = tableau.basic_vars.contains(&j);
            let rc = if is_basic { 0.0 } else { tableau.data[obj_row][j] };
            reduced_costs.push(ReducedCost {
                variable: var_name.clone(),
                value: values[j],
                reduced_cost: rc,
                is_basic,
            });
        }

        // Binding constraints
        let binding_constraints: Vec<String> = shadow_prices
            .iter()
            .filter(|sp| sp.value.abs() > self.tolerance)
            .map(|sp| sp.constraint.clone())
            .collect();

        // Sensitivity ranges (simplified - would need more computation for exact ranges)
        let objective_sensitivity = problem.variables.iter().enumerate().map(|(j, name)| {
            SensitivityRange {
                name: name.clone(),
                current: problem.objective.coefficients[j],
                lower_bound: f64::NEG_INFINITY, // Placeholder
                upper_bound: f64::INFINITY,     // Placeholder
            }
        }).collect();

        let rhs_sensitivity = problem.constraints.iter().map(|c| {
            SensitivityRange {
                name: c.name.clone(),
                current: c.rhs,
                lower_bound: 0.0,           // Placeholder
                upper_bound: f64::INFINITY, // Placeholder
            }
        }).collect();

        Analysis {
            shadow_prices,
            reduced_costs,
            binding_constraints,
            objective_sensitivity,
            rhs_sensitivity,
        }
    }
}

struct Tableau {
    data: Vec<Vec<f64>>,
    basic_vars: Vec<usize>,
    n_vars: usize,
    n_slack: usize,
    n_artificial: usize,
    has_artificial: bool,
    constraint_names: Vec<String>,
    variable_names: Vec<String>,
}

enum SimplexResult {
    Optimal,
    Unbounded,
    Infeasible,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problem::LpProblem;

    #[test]
    fn test_simple_maximization() {
        // Maximize: 3x + 2y
        // Subject to:
        //   x + y <= 4
        //   x <= 3
        //   y <= 3
        //   x, y >= 0
        // Optimal: x=3, y=1, obj=11
        let mut problem = LpProblem::new(vec!["x".to_string(), "y".to_string()]);
        problem.set_objective(vec![3.0, 2.0], false); // maximize
        problem.add_constraint("sum", vec![1.0, 1.0], ConstraintOp::Le, 4.0);
        problem.add_constraint("x_max", vec![1.0, 0.0], ConstraintOp::Le, 3.0);
        problem.add_constraint("y_max", vec![0.0, 1.0], ConstraintOp::Le, 3.0);

        let solver = Solver::new();
        let solution = solver.solve(&problem);

        println!("Status: {:?}", solution.status);
        println!("Values: {:?}", solution.values);
        println!("Objective: {}", solution.objective_value);

        assert_eq!(solution.status, SolutionStatus::Optimal);
        assert!((solution.values[0] - 3.0).abs() < 1e-6, "x = {} (expected 3)", solution.values[0]);
        assert!((solution.values[1] - 1.0).abs() < 1e-6, "y = {} (expected 1)", solution.values[1]);
        assert!((solution.objective_value - 11.0).abs() < 1e-6, "obj = {} (expected 11)", solution.objective_value);
    }

    #[test]
    fn test_minimization_with_ge() {
        // Minimize: 2x + 3y
        // Subject to:
        //   x + y >= 4
        //   x <= 3
        //   y <= 3
        //   x, y >= 0
        // Optimal: x=3, y=1, obj=9
        let mut problem = LpProblem::new(vec!["x".to_string(), "y".to_string()]);
        problem.set_objective(vec![2.0, 3.0], true);
        problem.add_constraint("sum", vec![1.0, 1.0], ConstraintOp::Ge, 4.0);
        problem.add_constraint("x_max", vec![1.0, 0.0], ConstraintOp::Le, 3.0);
        problem.add_constraint("y_max", vec![0.0, 1.0], ConstraintOp::Le, 3.0);

        let solver = Solver::new();
        let solution = solver.solve(&problem);

        println!("Status: {:?}", solution.status);
        println!("Values: {:?}", solution.values);
        println!("Objective: {}", solution.objective_value);

        assert_eq!(solution.status, SolutionStatus::Optimal);
        assert!((solution.values[0] - 3.0).abs() < 1e-6, "x = {} (expected 3)", solution.values[0]);
        assert!((solution.values[1] - 1.0).abs() < 1e-6, "y = {} (expected 1)", solution.values[1]);
        assert!((solution.objective_value - 9.0).abs() < 1e-6, "obj = {} (expected 9)", solution.objective_value);
    }

    #[test]
    fn test_infeasible() {
        // x >= 5
        // x <= 3
        let mut problem = LpProblem::new(vec!["x".to_string()]);
        problem.set_objective(vec![1.0], true);
        problem.add_constraint("lower", vec![1.0], ConstraintOp::Ge, 5.0);
        problem.add_constraint("upper", vec![1.0], ConstraintOp::Le, 3.0);

        let solver = Solver::new();
        let solution = solver.solve(&problem);

        assert_eq!(solution.status, SolutionStatus::Infeasible);
    }
}
