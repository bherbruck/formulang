# 3. Declarations

Formulang programs consist of top-level declarations: nutrients, ingredients, and formulas.

## 3.0 Common Properties & Aliases

All declaration types support these standard optional properties:

| Property | Alias  | Type   | Description                    |
|----------|--------|--------|--------------------------------|
| `name`   | -      | string | Display name                   |
| `code`   | -      | string | Identifier/SKU code            |
| `description` | `desc` | string | Description text          |

### Shorthand Aliases

For brevity, these aliases are supported:

| Full Name     | Alias  |
|---------------|--------|
| `description` | `desc` |
| `batch_size`  | `batch`|
| `nutrients`   | `nuts` |
| `ingredients` | `ings` |

## 3.1 Nutrient Declaration

Nutrients define the nutritional parameters used in formulation.

```
nutrient_decl := 'nutrient' identifier '{' nutrient_body '}'
nutrient_body := (property)*
property      := identifier (string | number | identifier)
```

### Properties

| Property | Type   | Required | Description                    |
|----------|--------|----------|--------------------------------|
| `name`   | string | No       | Display name for the nutrient  |
| `code`   | string | No       | Identifier/SKU code            |
| `desc`   | string | No       | Description                    |
| `unit`   | string | No       | Unit of measurement (for docs) |

### Examples

```
nutrient protein {
  name "Crude Protein"
  code "CP"
  unit "%"
}

nutrient energy {
  name "Metabolizable Energy"
  code "ME"
  unit "kcal/kg"
}

// Minimal declaration
nutrient fiber {}
```

## 3.2 Ingredient Declaration

Ingredients define raw materials with costs and nutrient compositions.

```
ingredient_decl := 'ingredient' identifier '{' ingredient_body '}'
ingredient_body := (property | nutrients_block)*
nutrients_block := 'nutrients' | 'nuts' '{' (nutrient_value)* '}'
nutrient_value  := reference number
```

### Properties

| Property | Type   | Required | Description                    |
|----------|--------|----------|--------------------------------|
| `name`   | string | No       | Display name for the ingredient|
| `code`   | string | No       | Identifier/SKU code            |
| `desc`   | string | No       | Description                    |
| `cost`   | number | Yes      | Cost per unit                  |

### Examples

```
ingredient corn {
  name "Yellow Corn"
  code "CORN-001"
  cost 150
  nuts {
    protein 8.5
    energy 3350
    fiber 2.2
  }
}

ingredient soybean_meal {
  name "Soybean Meal 48%"
  code "SBM-48"
  desc "High protein soybean meal"
  cost 450
  nuts {
    protein 44.0
    energy 2230
  }
}
```

### Nutrient References

Nutrients can be referenced by their identifier directly or via import path:

```
nuts {
  protein 8.5              // Local or imported nutrient
  amino_acids.lysine 0.26  // Namespaced nutrient from import
}
```

## 3.3 Formula Declaration

Formulas define a complete formulation problem with constraints.

```
formula_decl      := 'formula' identifier '{' formula_body '}'
formula_body      := (property | nutrients_block | ingredients_block)*
nutrients_block   := 'nutrients' | 'nuts' '{' (constraint)* '}'
ingredients_block := 'ingredients' | 'ings' '{' (ingredient_constraint)* '}'
```

### Properties

| Property     | Alias   | Type   | Required | Description                     |
|--------------|---------|--------|----------|---------------------------------|
| `name`       | -       | string | No       | Display name for the formula    |
| `code`       | -       | string | No       | Identifier/SKU code             |
| `description`| `desc`  | string | No       | Description of the formula      |
| `batch_size` | `batch` | number | Yes      | Total batch size                |

### Examples

```
formula starter {
  name "Starter Feed"
  code "BRO-START-01"
  desc "For chicks 0-3 weeks"
  batch 1000

  nuts {
    protein min 22 max 24
    energy min 3000
    fiber max 5
    calcium min 0.9 max 1.1
    calcium / phosphorus min 1.5 max 2.0
  }

  ings {
    corn max 50%
    soybean_meal min 20% max 35%
    wheat_midds max 10%
    limestone
    dicalcium_phosphate
    vitamin_premix min 0.25% max 0.25%
  }
}
```
