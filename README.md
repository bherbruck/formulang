# Formulang

A domain-specific language for least-cost feed formulation. Define nutrients, ingredients, and formula constraints in a readable syntax, then solve for the optimal mix using linear programming.

![Formulang Playground](docs/media/screenshot.png)

**[Try the Live Demo](https://bherbruck.github.io/formulang/)**

## Features

- **Least-cost optimization** - Solves for the minimum cost formulation meeting all constraints
- **Templates** - Reuse definitions across ingredients and formulas
- **Reference expressions** - Access values from other definitions with dot notation
- **Computed values** - Use expressions like `corn.cost * 2`
- **Nutrient ratios** - Constrain ratios like `lysine / methionine min 0.1`
- **Ingredient groups** - Constrain combinations like `corn + soybean_meal max 75%`
- **Inheritance** - Spread nutrients and ingredients from other formulas

## Examples

### Nutrients

```
nutrient protein {
  code "02"
  name "Crude Protein"
  unit "%"
}

nutrient energy {
  code "01"
  name "Metabolizable Energy"
  unit "kcal/kg"
}

// Minimal definitions work too
nutrient lysine {}
nutrient methionine {}
```

### Ingredients with Templates

Define reusable templates and reference their values:

```
template ingredient corn_base {
  cost 0.15
  nutrients {
    protein 7.5
    energy 3350
    fiber 2.2
  }
}

ingredient corn {
  name "Yellow Corn"
  cost corn_base.cost
  nutrients {
    corn_base.nutrients
  }
}

// Computed values and overrides
ingredient org_corn {
  name "ORG Corn"
  cost corn.cost * 2
  nutrients {
    corn_base.nutrients
    protein 8  // Override specific values
  }
}
```

### Formulas with Inheritance

Build formula families that share constraints:

```
template formula base {
  batch 1000
}

formula con_1001 {
  code "1001"
  name "CON Starter"
  desc "For chicks 0-3 weeks"
  batch base.batch

  nutrients {
    protein     min   20    max   24
    energy      min 2900    max 2900
    fiber                   max    5
    calcium     min    0.9  max    1.2
    phosphorus  min    0.4  max    0.7
  }

  ingredients {
    corn                        max 50%
    soybean_meal        min 15% max 45%
    corn + soybean_meal         max 75%
    wheat_midds                 max 20%
    limestone                   max  3%
    premix              min  1% max  1%
  }
}

// Inherit and extend
formula con_1002 {
  code "1002"
  name "CON Grower"
  batch base.batch

  nutrients {
    con_1001.nutrients
  }

  ingredients {
    con_1001.ingredients
    corn_oil max 30%  // Add new constraint
  }
}
```

### Nutrient Ratios

Constrain nutrient ratios with named aliases:

```
template formula ratios {
  nutrients {
    lysine min 12
    lysine / arginine   min 0.1 as lysine_arginine
    lysine / methionine min 0.1 as lysine_methionine
  }
}

formula starter {
  nutrients {
    protein min 20 max 24
    ratios.nutrients.lysine_arginine
  }
  // ...
}
```

### Scaling Formulas

Create product families from base definitions:

```
formula org_7001 {
  code "7001"
  name "ORG Starter"
  batch 1000
  nutrients { con_1001.nutrients }
  ingredients {
    org_corn                max 50%
    soybean_meal    min 15% max 45%
    org_corn + soybean_meal max 75%
    // ...
  }
}

// Rapidly define variants
formula org_7002 {
  code "7002"
  name "ORG Grower"
  batch 1000
  nutrients { org_7001.nutrients }
  ingredients { org_7001.ingredients }
}

formula org_7003 {
  code "7003"
  name "ORG Grower Plus"
  batch 1000
  nutrients { org_7001.nutrients }
  ingredients {
    org_7001.ingredients
    corn_oil max 20%  // Customize
  }
}
```

## License

MIT
