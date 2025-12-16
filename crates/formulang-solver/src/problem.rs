/// Represents a linear programming problem
#[derive(Debug, Clone)]
pub struct LpProblem {
    /// Variable names
    pub variables: Vec<String>,
    /// Objective function coefficients (costs)
    pub objective: Objective,
    /// Constraints
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone)]
pub struct Objective {
    /// Coefficients for each variable
    pub coefficients: Vec<f64>,
    /// Whether to minimize or maximize
    pub minimize: bool,
}

#[derive(Debug, Clone)]
pub struct Constraint {
    /// Name/label for the constraint (for diagnostics)
    pub name: String,
    /// Coefficients for each variable
    pub coefficients: Vec<f64>,
    /// Comparison operator
    pub op: ConstraintOp,
    /// Right-hand side value
    pub rhs: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintOp {
    /// Less than or equal (<=)
    Le,
    /// Greater than or equal (>=)
    Ge,
    /// Equal (=)
    Eq,
}

impl LpProblem {
    pub fn new(variables: Vec<String>) -> Self {
        let n = variables.len();
        Self {
            variables,
            objective: Objective {
                coefficients: vec![0.0; n],
                minimize: true,
            },
            constraints: Vec::new(),
        }
    }

    pub fn set_objective(&mut self, coefficients: Vec<f64>, minimize: bool) {
        self.objective = Objective { coefficients, minimize };
    }

    pub fn add_constraint(&mut self, name: impl Into<String>, coefficients: Vec<f64>, op: ConstraintOp, rhs: f64) {
        self.constraints.push(Constraint {
            name: name.into(),
            coefficients,
            op,
            rhs,
        });
    }

    pub fn num_variables(&self) -> usize {
        self.variables.len()
    }

    pub fn num_constraints(&self) -> usize {
        self.constraints.len()
    }
}
