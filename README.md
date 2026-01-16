# Formulang

A domain-specific language (DSL) for least-cost feed formulation. Formulang lets you define nutrients, ingredients, and formula constraints in a readable syntax, then solves for the optimal (lowest cost) mix using linear programming.

![Formulang Playground](docs/media/screenshot.png)

**[Try the Live Demo](https://bherbruck.github.io/formulang/)**

## Features

- **Declarative DSL** - Define nutrients, ingredients, and formulas in a clean, readable syntax
- **Least-cost optimization** - Solves for the minimum cost formulation that meets all constraints
- **Constraint flexibility** - Set min/max bounds on nutrients and ingredients (absolute or percentage)
- **Ingredient groups** - Constrain combinations of ingredients together
- **Sensitivity analysis** - Understand shadow prices and binding constraints
- **Multiple targets** - Rust CLI, WebAssembly, and browser playground

## Installation

### CLI (Rust)

```bash
cargo install --path crates/formulang-cli
```

### WebAssembly (npm)

```bash
# Build the WASM package
cd crates/formulang-lang
wasm-pack build --target web --features wasm
```

## Language Syntax

### Nutrients

Define the nutritional parameters you want to track:

```
nutrient protein {
  name "Crude Protein"
  unit "%"
}

nutrient energy {
  name "Metabolizable Energy"
  unit "kcal/kg"
}
```

### Ingredients

Define available ingredients with their costs and nutrient compositions:

```
ingredient corn {
  name "Yellow Corn"
  cost 150
  nutrients {
    protein 8.5
    energy 3350
  }
}

ingredient soybean_meal {
  name "Soybean Meal 48%"
  cost 450
  nutrients {
    protein 48.0
    energy 2230
  }
}
```

### Formulas

Define formulas with nutrient requirements and ingredient constraints:

```
formula starter {
  name "Starter Feed"
  description "For chicks 0-3 weeks"
  batch_size 1000

  nutrients {
    protein min 20 max 24
    energy min 2900
    fiber max 5
  }

  ingredients {
    corn max 70%
    soybean_meal min 15% max 45%
    limestone max 3%
  }
}
```

### Ingredient Groups

Constrain multiple ingredients together:

```
ingredients {
  corn + soybean_meal max 75%
}
```

## CLI Usage

```bash
# Check a file for errors
formulang check feed.fm

# Parse and show the AST
formulang parse feed.fm

# Solve a formula
formulang solve feed.fm starter

# Solve with sensitivity analysis
formulang solve feed.fm starter --analysis
```

## Project Structure

```
formulang/
├── crates/
│   ├── formulang-lang/      # Lexer, parser, compiler, WASM bindings
│   ├── formulang-solver/    # Linear programming solver
│   └── formulang-cli/       # Command-line interface
├── packages/
│   ├── formulang-monaco/    # Monaco editor language support
│   └── formulang-playground/ # Web playground (React + Vite)
└── docs/
    └── spec/                # Language specification and examples
```

## License

MIT
