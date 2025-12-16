# 8. Complete Examples

## 8.1 Basic Poultry Feed Formulation

### nutrients.fm

```
nutrient protein {
  name "Crude Protein"
  unit "%"
}

nutrient energy {
  name "Metabolizable Energy"
  unit "kcal/kg"
}

nutrient fiber {
  name "Crude Fiber"
  unit "%"
}

nutrient fat {
  name "Crude Fat"
  unit "%"
}

nutrient calcium {
  name "Calcium"
  unit "%"
}

nutrient phosphorus {
  name "Available Phosphorus"
  unit "%"
}

nutrient sodium {
  name "Sodium"
  unit "%"
}

nutrient lysine {
  name "Lysine"
  unit "%"
}

nutrient methionine {
  name "Methionine"
  unit "%"
}
```

### ingredients.fm

```
import ./nutrients.fm { * }

ingredient corn {
  name "Yellow Corn"
  cost 150
  nutrients {
    protein 8.5
    energy 3350
    fiber 2.2
    fat 3.8
    calcium 0.02
    phosphorus 0.08
    lysine 0.26
    methionine 0.18
  }
}

ingredient soybean_meal {
  name "Soybean Meal 48%"
  cost 450
  nutrients {
    protein 48.0
    energy 2230
    fiber 3.5
    fat 1.0
    calcium 0.30
    phosphorus 0.27
    lysine 2.96
    methionine 0.67
  }
}

ingredient wheat_midds {
  name "Wheat Middlings"
  cost 180
  nutrients {
    protein 15.5
    energy 2600
    fiber 7.0
    fat 4.0
    calcium 0.12
    phosphorus 0.35
    lysine 0.58
    methionine 0.23
  }
}

ingredient meat_meal {
  name "Meat and Bone Meal"
  cost 380
  nutrients {
    protein 50.0
    energy 2100
    fiber 2.5
    fat 10.0
    calcium 10.0
    phosphorus 5.0
    lysine 2.50
    methionine 0.70
  }
}

ingredient limestone {
  name "Limestone"
  cost 50
  nutrients {
    calcium 38.0
    phosphorus 0.0
  }
}

ingredient dicalcium_phosphate {
  name "Dicalcium Phosphate"
  cost 600
  nutrients {
    calcium 22.0
    phosphorus 18.5
  }
}

ingredient salt {
  name "Salt"
  cost 100
  nutrients {
    sodium 39.0
  }
}

ingredient soy_oil {
  name "Soybean Oil"
  cost 800
  nutrients {
    energy 8800
    fat 99.0
  }
}

ingredient vitamin_premix {
  name "Vitamin/Mineral Premix"
  cost 2500
  nutrients {}
}

ingredient dl_methionine {
  name "DL-Methionine"
  cost 3500
  nutrients {
    methionine 99.0
  }
}

ingredient lysine_hcl {
  name "L-Lysine HCl"
  cost 2000
  nutrients {
    lysine 78.0
  }
}
```

### bases/poultry.fm

```
formula poultry_base {
  nutrients {
    fiber max 7
    calcium min 0.8 max 1.5
    phosphorus min 0.35 max 0.8
    calcium / phosphorus min 1.5 max 2.5
    sodium min 0.15 max 0.25
  }
}
```

### formulas/broiler_starter.fm

```
import ../nutrients.fm { * }
import ../ingredients.fm { * }
import ../bases/poultry.fm

formula broiler_starter {
  name "Broiler Starter"
  description "High protein starter for broiler chicks 0-14 days"
  batch_size 1000

  nutrients {
    poultry_base.nutrients
    protein min 22 max 24
    energy min 3000 max 3100
    lysine min 1.30
    methionine min 0.50
    lysine / methionine min 2.4 max 2.8
  }

  ingredients {
    corn max 55%
    soybean_meal min 25% max 40%
    wheat_midds max 5%
    meat_meal max 5%
    soy_oil max 3%
    limestone min 0.5% max 2%
    dicalcium_phosphate max 2%
    salt min 0.2% max 0.5%
    vitamin_premix min 0.25% max 0.5%
    dl_methionine max 0.3%
    lysine_hcl max 0.2%
  }
}
```

### formulas/broiler_grower.fm

```
import ../nutrients.fm { * }
import ../ingredients.fm { * }
import ../bases/poultry.fm
import ./broiler_starter.fm

formula broiler_grower {
  name "Broiler Grower"
  description "Growth phase feed for broilers 15-28 days"
  batch_size 1000

  nutrients {
    poultry_base.nutrients
    protein min 20 max 22
    energy min 3100 max 3200
    lysine min 1.15
    methionine min 0.45
  }

  ingredients {
    broiler_starter.ingredients
    corn max 60%                    // Allow more corn
    soybean_meal min 20% max 35%    // Less soy needed
  }
}
```

### formulas/broiler_finisher.fm

```
import ../nutrients.fm { * }
import ../ingredients.fm { * }
import ../bases/poultry.fm

formula broiler_finisher {
  name "Broiler Finisher"
  description "Finishing feed for broilers 29+ days"
  batch_size 1000

  nutrients {
    poultry_base.nutrients
    protein min 18 max 20
    energy min 3200 max 3300
    lysine min 1.00
    methionine min 0.40
  }

  ingredients {
    corn max 65%
    soybean_meal min 15% max 30%
    wheat_midds max 10%
    meat_meal max 7%
    soy_oil max 5%
    limestone min 0.5% max 2%
    dicalcium_phosphate max 2%
    salt min 0.2% max 0.5%
    vitamin_premix min 0.25% max 0.5%
    dl_methionine max 0.25%
    lysine_hcl max 0.15%
  }
}
```

## 8.2 Swine Feed with Phases

### formulas/swine_base.fm

```
import ../nutrients.fm { * }

formula swine_base {
  nutrients {
    fiber max 5
    calcium min 0.5 max 1.0
    phosphorus min 0.4 max 0.7
    sodium min 0.15 max 0.30
  }
}
```

### formulas/swine_nursery.fm

```
import ../nutrients.fm { * }
import ../ingredients.fm { * }
import ./swine_base.fm

formula swine_nursery {
  name "Swine Nursery Phase 1"
  description "For pigs 12-25 lbs"
  batch_size 1000

  nutrients {
    swine_base.nutrients
    protein min 22 max 24
    energy min 3300
    lysine min 1.50
    methionine min 0.40
  }

  ingredients {
    corn max 45%
    soybean_meal min 20% max 35%
    wheat_midds max 5%
    soy_oil max 3%
    limestone
    dicalcium_phosphate
    salt
    vitamin_premix min 0.5%
    dl_methionine max 0.3%
    lysine_hcl max 0.3%
  }
}
```

## 8.3 Mapping to Linear Programming

A Formulang formula translates directly to an LP problem:

### Objective Function

Minimize: `sum(ingredient_amount[i] * ingredient_cost[i])` for all ingredients

### Decision Variables

- `ingredient_amount[i]` for each ingredient in the formula

### Constraints

From `nutrients` block:
- For each `nutrient min V`: `sum(amount[i] * nutrient_content[i]) >= V * batch_size / 100`
- For each `nutrient max V`: `sum(amount[i] * nutrient_content[i]) <= V * batch_size / 100`

From `ingredients` block:
- For each `ingredient min V`: `amount[ingredient] >= V`
- For each `ingredient max V`: `amount[ingredient] <= V`
- Sum constraint: `sum(all amounts) == batch_size`

### Example LP Output

For `broiler_starter` with batch_size 1000:

```
Minimize:
  150*corn + 450*soybean_meal + 180*wheat_midds + 380*meat_meal +
  800*soy_oil + 50*limestone + 600*dicalcium_phosphate + 100*salt +
  2500*vitamin_premix + 3500*dl_methionine + 2000*lysine_hcl

Subject to:
  // Nutrient constraints
  8.5*corn + 48*soybean_meal + ... >= 220   // protein min 22%
  8.5*corn + 48*soybean_meal + ... <= 240   // protein max 24%
  3350*corn + 2230*soybean_meal + ... >= 3000000  // energy min 3000

  // Ingredient constraints
  corn <= 550                               // max 55%
  soybean_meal >= 250                       // min 25%
  soybean_meal <= 400                       // max 40%

  // Sum constraint
  corn + soybean_meal + ... = 1000          // batch_size

  // Non-negativity
  all variables >= 0
```
