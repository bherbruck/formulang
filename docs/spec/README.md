# Formulang Language Specification

Formulang is a domain-specific language for least-cost feed formulation.

## Specification Documents

1. [Introduction](./01-introduction.md) - Overview and design goals
2. [Lexical Structure](./02-lexical-structure.md) - Tokens, comments, literals
3. [Declarations](./03-declarations.md) - Nutrients, ingredients, formulas
4. [Expressions](./04-expressions.md) - Arithmetic and references
5. [Constraints](./05-constraints.md) - Min/max syntax
6. [Imports](./06-imports.md) - Modules and namespacing
7. [Composition](./07-composition.md) - Inheritance and overrides
8. [Examples](./08-examples.md) - Complete working examples

## Quick Reference

```
// Nutrients
nutrient protein {
  name: "Crude Protein"
  unit: "%"
}

// Ingredients
ingredient corn {
  cost: 150
  nutrients {
    protein: 8.5
    energy: 3350
  }
}

// Formulas
formula starter {
  batch_size: 1000

  nutrients {
    protein min 20 max 24
    energy min 2800
    calcium / phosphorus min 1.5
  }

  ingredients {
    corn max 50%
    soybean_meal min 20%
    limestone
  }
}

// Composition
formula grower {
  batch_size: 1000

  nutrients {
    starter.nutrients           // Include all
    protein min 18 max 22       // Override
  }

  ingredients {
    starter.ingredients
    corn max 60%
  }
}
```

## Examples

See the [examples](./examples/) directory for complete working examples.
