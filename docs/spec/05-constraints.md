# 5. Constraints

Constraints define the bounds for the optimization problem using `min` and `max` keywords.

## 5.1 Grammar

```
constraint      := expression constraint_bounds?
constraint_bounds := min_bound max_bound? | max_bound
min_bound       := 'min' number
max_bound       := 'max' number
```

## 5.2 Constraint Forms

### Minimum Only

```
protein min 18
energy min 2800
```

Translates to LP constraint: `expression >= value`

### Maximum Only

```
fiber max 5
corn max 500
```

Translates to LP constraint: `expression <= value`

### Range (Min and Max)

```
protein min 18 max 22
calcium min 0.9 max 1.1
```

Translates to two LP constraints:
- `expression >= min_value`
- `expression <= max_value`

### No Bounds (Inclusion Only)

In `ingredients` blocks, an identifier without bounds indicates the ingredient is available for use with no specific limits (implicitly 0 to batch_size):

```
ingredients {
  limestone        // Available, no specific bounds
  corn max 50%     // Has upper bound
}
```

## 5.3 Percentages in Ingredient Constraints

The `%` suffix is valid only in `ingredients` blocks and represents a percentage of `batch_size`:

```
formula example {
  batch_size 1000

  ingredients {
    corn max 50%          // Equivalent to: corn max 500
    soybean_meal min 10%  // Equivalent to: soybean_meal min 100
  }
}
```

Percentages are **not valid** in `nutrients` blocks since nutrient values are already in their natural units (%, kcal/kg, ppm, etc.).

## 5.4 Complex Constraints

Constraints can use expressions on the left-hand side:

### Ratio Constraints

```
nutrients {
  calcium / phosphorus min 1.5 max 2.0
  lysine / methionine min 2.5
}
```

### Grouped Ingredient Constraints

```
ingredients {
  corn + wheat + barley max 60%
  animal_proteins min 5% max 15%
}
```

### Arithmetic Constraints

```
nutrients {
  digestible_protein * 0.9 min 16
}
```

## 5.5 Constraint Semantics

### In `nutrients` Block

The expression computes the weighted sum of nutrient contributions from all ingredients:

```
nutrients {
  protein min 18
}
```

Meaning: The sum of `(ingredient_amount * ingredient_protein)` for all ingredients must be >= 18% of batch.

### In `ingredients` Block

The expression computes amounts of ingredients:

```
ingredients {
  corn + wheat max 60%
}
```

Meaning: The sum of corn and wheat amounts must be <= 60% of batch_size.
