# 7. Composition and Inheritance

Formulas can compose constraints from other formulas using reference inclusion.

## 7.1 Including Constraints

Within a `nutrients` or `ingredients` block, reference another formula's block to include all its constraints:

```
formula poultry_base {
  nutrients {
    protein min 16
    energy min 2800
    calcium min 0.8 max 1.2
    phosphorus min 0.4
    calcium / phosphorus min 1.5 max 2.0
  }
}

formula starter {
  batch_size 1000

  nutrients {
    poultry_base.nutrients    // Include all nutrient constraints
    protein min 22            // Override: tighter minimum
  }

  ingredients {
    corn max 50%
    soybean_meal min 20%
  }
}
```

## 7.2 Selective Inclusion

Include specific constraints using bracket notation:

```
nutrients {
  poultry_base.nutrients.[protein, energy]    // Only these two
  calcium min 1.0 max 1.1                     // Define others locally
}
```

## 7.3 Partial Constraint Inclusion

Include only the `min` or `max` portion of a constraint:

```
nutrients {
  poultry_base.nutrients.protein.min    // Only the >= constraint
  protein max 20                         // Define a different max
}
```

## 7.4 Multiple Inheritance

Compose from multiple base formulas:

```
formula organic_starter {
  batch_size 1000

  nutrients {
    poultry_base.nutrients
    organic_standards.nutrients     // Additional organic requirements
    protein min 23                  // Override
  }

  ingredients {
    organic_ingredients.ingredients
    corn max 45%
  }
}
```

## 7.5 Override Semantics

When the same constraint appears multiple times, the **last definition wins**:

```
nutrients {
  poultry_base.nutrients      // protein min 16
  starter_base.nutrients      // protein min 20
  protein min 22              // Final: protein min 22
}
```

This allows base formulas to provide defaults that can be overridden.

## 7.6 Ingredient List Composition

Include ingredient lists from other formulas:

```
formula grower {
  batch_size 1000

  ingredients {
    starter.ingredients           // Include all ingredients from starter
    corn max 55%                  // Override corn limit
    meat_meal min 2% max 5%       // Add new ingredient
  }
}
```

## 7.7 Ingredient Groups

Define reusable ingredient groups for constraints:

```
// groups.fm
ingredient corn {}
ingredient wheat {}
ingredient barley {}
ingredient oats {}

group grains [corn, wheat, barley, oats]

// formula.fm
import ./groups.fm

formula starter {
  batch_size 1000

  ingredients {
    grains.[corn, wheat, barley]   // Select from group
    grains max 60%                 // Constrain entire group
    corn max 50%                   // Individual constraint
  }
}
```

## 7.8 Complete Composition Example

```
// bases/poultry.fm
formula poultry_base {
  nutrients {
    protein min 16
    energy min 2800
    fiber max 7
    calcium min 0.8 max 1.5
    phosphorus min 0.4 max 0.8
    calcium / phosphorus min 1.5 max 2.0
  }
}

// bases/organic.fm
formula organic_base {
  nutrients {
    // Organic may have different requirements
  }

  ingredients {
    // Only organic-approved ingredients
    synthetic_amino_acids max 0%
  }
}

// formulas/organic_starter.fm
import ../bases/poultry.fm
import ../bases/organic.fm

formula organic_starter {
  name "Organic Starter"
  batch_size 1000

  nutrients {
    poultry_base.nutrients
    protein min 22 max 24       // Starters need more protein
    energy min 3000             // Higher energy for growth
  }

  ingredients {
    organic_base.ingredients
    organic_corn max 50%
    organic_soybean_meal min 25% max 35%
    limestone min 1% max 2%
    organic_vitamin_premix min 0.25% max 0.5%
  }
}
```
