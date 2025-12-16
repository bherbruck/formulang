/// The result of solving an LP problem
#[derive(Debug, Clone)]
pub struct Solution {
    /// Solution status
    pub status: SolutionStatus,
    /// Optimal values for each variable
    pub values: Vec<f64>,
    /// Optimal objective value
    pub objective_value: f64,
    /// Detailed analysis
    pub analysis: Analysis,
    /// Constraint violations (populated when infeasible)
    pub violations: Vec<ConstraintViolation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolutionStatus {
    /// An optimal solution was found
    Optimal,
    /// The problem is infeasible (no solution exists)
    Infeasible,
    /// The problem is unbounded
    Unbounded,
    /// Solver encountered an error
    Error,
}

/// Detailed analysis of the optimal solution
#[derive(Debug, Clone)]
pub struct Analysis {
    /// Shadow prices (dual values) for each constraint
    /// Indicates how much the objective would change per unit relaxation
    pub shadow_prices: Vec<ShadowPrice>,

    /// Reduced costs for each variable
    /// For non-basic variables, indicates how much cost must change to enter solution
    pub reduced_costs: Vec<ReducedCost>,

    /// Which constraints are binding (tight) at optimum
    pub binding_constraints: Vec<String>,

    /// Sensitivity ranges for objective coefficients
    pub objective_sensitivity: Vec<SensitivityRange>,

    /// Sensitivity ranges for constraint RHS values
    pub rhs_sensitivity: Vec<SensitivityRange>,
}

#[derive(Debug, Clone)]
pub struct ShadowPrice {
    /// Constraint name
    pub constraint: String,
    /// Shadow price value
    pub value: f64,
    /// Interpretation
    pub interpretation: String,
}

#[derive(Debug, Clone)]
pub struct ReducedCost {
    /// Variable name
    pub variable: String,
    /// Current value in solution
    pub value: f64,
    /// Reduced cost
    pub reduced_cost: f64,
    /// Is this variable in the basis?
    pub is_basic: bool,
}

#[derive(Debug, Clone)]
pub struct SensitivityRange {
    /// Variable or constraint name
    pub name: String,
    /// Current value
    pub current: f64,
    /// Lower bound of range where solution structure stays same
    pub lower_bound: f64,
    /// Upper bound of range where solution structure stays same
    pub upper_bound: f64,
}

/// Information about a violated constraint
#[derive(Debug, Clone)]
pub struct ConstraintViolation {
    /// Constraint name
    pub constraint: String,
    /// Required value (from constraint RHS)
    pub required: f64,
    /// Actual value achieved
    pub actual: f64,
    /// How much the constraint is violated by
    pub violation_amount: f64,
    /// Human-readable description of what's wrong
    pub description: String,
}

impl Solution {
    pub fn infeasible() -> Self {
        Self {
            status: SolutionStatus::Infeasible,
            values: Vec::new(),
            objective_value: f64::INFINITY,
            analysis: Analysis::empty(),
            violations: Vec::new(),
        }
    }

    pub fn infeasible_with_violations(violations: Vec<ConstraintViolation>) -> Self {
        Self {
            status: SolutionStatus::Infeasible,
            values: Vec::new(),
            objective_value: f64::INFINITY,
            analysis: Analysis::empty(),
            violations,
        }
    }

    pub fn infeasible_with_relaxed(
        values: Vec<f64>,
        objective_value: f64,
        violations: Vec<ConstraintViolation>,
    ) -> Self {
        Self {
            status: SolutionStatus::Infeasible,
            values,
            objective_value,
            analysis: Analysis::empty(),
            violations,
        }
    }

    pub fn unbounded() -> Self {
        Self {
            status: SolutionStatus::Unbounded,
            values: Vec::new(),
            objective_value: f64::NEG_INFINITY,
            analysis: Analysis::empty(),
            violations: Vec::new(),
        }
    }
}

impl Analysis {
    pub fn empty() -> Self {
        Self {
            shadow_prices: Vec::new(),
            reduced_costs: Vec::new(),
            binding_constraints: Vec::new(),
            objective_sensitivity: Vec::new(),
            rhs_sensitivity: Vec::new(),
        }
    }
}
