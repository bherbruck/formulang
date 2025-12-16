# 4. Expressions

Expressions are used in constraints to define the left-hand side value being constrained.

## 4.1 Grammar

```
expression := term (('+' | '-') term)*
term       := factor (('*' | '/') factor)*
factor     := number | reference | '(' expression ')'
reference  := identifier ('.' identifier)*
```

## 4.2 References

References identify nutrients, ingredients, or members of imported modules.

```
protein                     // Local nutrient
corn                        // Local ingredient
base_formula.nutrients      // All nutrients from a formula
amino_acids.lysine          // Nutrient from imported module
poultry_base.nutrients.protein.min  // Specific constraint from base
```

### Reference Resolution

1. Local scope (current file's declarations)
2. Imported modules (via `import` statements)
3. Base formula references (via dot notation)

## 4.3 Arithmetic Expressions

Expressions support standard arithmetic for defining ratios and sums.

### Operators

| Operator | Description    | Example                |
|----------|----------------|------------------------|
| `+`      | Addition       | `corn + wheat`         |
| `-`      | Subtraction    | `total - filler`       |
| `*`      | Multiplication | `protein * 0.8`        |
| `/`      | Division       | `calcium / phosphorus` |

### Precedence (highest to lowest)

1. Parentheses `()`
2. Multiplication, Division `*`, `/`
3. Addition, Subtraction `+`, `-`

### Examples

```
nutrients {
  // Ratio constraint
  calcium / phosphorus min 1.5 max 2.0

  // Adjusted value
  protein * 0.9 min 18
}

ingredients {
  // Grouped ingredients
  corn + wheat + barley max 60%

  // Parenthesized expression
  (corn + wheat) / batch_size max 0.5
}
```

## 4.4 Context-Sensitive Behavior

The meaning of identifiers depends on the block context:

- In `nutrients {}`: Identifiers resolve to nutrient values
- In `ingredients {}`: Identifiers resolve to ingredient amounts

```
formula example {
  batch_size: 1000

  nutrients {
    protein min 18        // protein = weighted sum of ingredient proteins
  }

  ingredients {
    corn max 500          // corn = amount of corn in batch
  }
}
```
