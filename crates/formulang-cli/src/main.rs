use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "formulang")]
#[command(about = "A DSL for least-cost feed formulation", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a .fm file and output the AST
    Parse {
        /// The file to parse
        file: PathBuf,
        /// Output format (json, pretty)
        #[arg(short, long, default_value = "pretty")]
        format: String,
    },
    /// Solve a formula and output the optimal solution
    Solve {
        /// The file containing the formula
        file: PathBuf,
        /// The formula name to solve
        formula: String,
        /// Show detailed analysis
        #[arg(short, long)]
        analysis: bool,
    },
    /// Check a .fm file for errors
    Check {
        /// The file to check
        file: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { file, format } => {
            let source = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    std::process::exit(1);
                }
            };

            match formulang_lang::Parser::parse(&source) {
                Ok(program) => {
                    if format == "json" {
                        println!("{}", serde_json::to_string_pretty(&program).unwrap_or_else(|_| {
                            "Error: serde feature not enabled".to_string()
                        }));
                    } else {
                        println!("{:#?}", program);
                    }
                }
                Err(e) => {
                    eprintln!("Parse error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Solve { file, formula, analysis } => {
            let source = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    std::process::exit(1);
                }
            };

            let program = match formulang_lang::Parser::parse(&source) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Parse error: {}", e);
                    std::process::exit(1);
                }
            };

            // Compile
            let mut compiler = formulang_lang::Compiler::new();
            if let Err(e) = compiler.load(&program) {
                eprintln!("Compile error: {}", e);
                std::process::exit(1);
            }

            let compiled = match compiler.compile_formula(&formula) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Compile error: {}", e);
                    std::process::exit(1);
                }
            };

            // Solve
            let solver = formulang_solver::Solver::new();
            let solution = solver.solve(&compiled.lp_problem);

            // Output results
            println!("Formula: {}", compiled.name);
            if let Some(ref desc) = compiled.description {
                println!("Description: {}", desc);
            }
            println!("Batch size: {}", compiled.batch_size);
            println!();

            match solution.status {
                formulang_solver::SolutionStatus::Optimal => {
                    println!("Status: OPTIMAL");
                    println!("Total cost: {:.2}", solution.objective_value);
                    println!();
                    println!("Ingredients:");
                    for (i, name) in compiled.ingredients.iter().enumerate() {
                        let amount = solution.values[i];
                        let pct = amount / compiled.batch_size * 100.0;
                        if amount > 0.001 {
                            println!("  {:20} {:10.2} ({:5.2}%)", name, amount, pct);
                        }
                    }

                    if analysis {
                        println!();
                        println!("Analysis:");
                        println!();

                        if !solution.analysis.binding_constraints.is_empty() {
                            println!("Binding constraints (pinch points):");
                            for name in &solution.analysis.binding_constraints {
                                println!("  - {}", name);
                            }
                            println!();
                        }

                        println!("Shadow prices:");
                        for sp in &solution.analysis.shadow_prices {
                            if sp.value.abs() > 0.001 {
                                println!("  {:30} {:10.4}", sp.constraint, sp.value);
                                println!("    {}", sp.interpretation);
                            }
                        }
                        println!();

                        println!("Reduced costs (ingredients not in solution):");
                        for rc in &solution.analysis.reduced_costs {
                            if !rc.is_basic && rc.reduced_cost.abs() > 0.001 {
                                println!(
                                    "  {:20} cost must decrease by {:.2} to enter solution",
                                    rc.variable, rc.reduced_cost
                                );
                            }
                        }
                    }
                }
                formulang_solver::SolutionStatus::Infeasible => {
                    println!("Status: INFEASIBLE");
                    println!("No solution exists that satisfies all constraints.");
                    std::process::exit(1);
                }
                formulang_solver::SolutionStatus::Unbounded => {
                    println!("Status: UNBOUNDED");
                    println!("The problem has no finite optimal solution.");
                    std::process::exit(1);
                }
                formulang_solver::SolutionStatus::Error => {
                    println!("Status: ERROR");
                    println!("Solver encountered an error.");
                    std::process::exit(1);
                }
            }
        }
        Commands::Check { file } => {
            let source = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    std::process::exit(1);
                }
            };

            match formulang_lang::Parser::parse(&source) {
                Ok(program) => {
                    let mut nutrients = 0;
                    let mut ingredients = 0;
                    let mut formulas = 0;
                    let mut imports = 0;

                    for item in &program.items {
                        match item {
                            formulang_lang::Item::Nutrient(_) => nutrients += 1,
                            formulang_lang::Item::Ingredient(_) => ingredients += 1,
                            formulang_lang::Item::Formula(_) => formulas += 1,
                            formulang_lang::Item::Import(_) => imports += 1,
                        }
                    }

                    println!("✓ {} is valid", file.display());
                    println!("  {} imports", imports);
                    println!("  {} nutrients", nutrients);
                    println!("  {} ingredients", ingredients);
                    println!("  {} formulas", formulas);
                }
                Err(e) => {
                    eprintln!("✗ {} has errors:", file.display());
                    eprintln!("  {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
