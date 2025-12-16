# 2. Lexical Structure

## 2.1 Source Encoding

Formulang source files use UTF-8 encoding with the `.fm` file extension.

## 2.2 Comments

```
// Single-line comment

/*
   Multi-line
   comment
*/
```

## 2.3 Whitespace

Whitespace (spaces, tabs, newlines) is used to separate tokens and is otherwise ignored. Newlines are significant within blocks to separate statements.

## 2.4 Identifiers

Identifiers name nutrients, ingredients, formulas, and properties.

```
identifier := [a-zA-Z_][a-zA-Z0-9_]*
```

Examples: `protein`, `corn`, `soybean_meal`, `starter_v2`

## 2.5 Keywords

Reserved keywords:

```
nutrient    ingredient    formula
nutrients   ingredients   batch_size
cost        name          min
max         import
```

## 2.6 Literals

### 2.6.1 Numbers

Numbers can be integers or decimals, optionally negative:

```
number := '-'? [0-9]+ ('.' [0-9]+)?
```

Examples: `100`, `8.5`, `-20`, `0.005`

### 2.6.2 Percentages

A number followed by `%` represents a percentage of `batch_size`:

```
percentage := number '%'
```

Examples: `50%`, `10.5%`

Percentages are only valid in `ingredients` blocks.

### 2.6.3 Strings

Strings are enclosed in double quotes:

```
string := '"' [^"]* '"'
```

Examples: `"Crude Protein"`, `"Starter Feed"`

## 2.7 Operators

```
/    Division (for ratio expressions)
+    Addition (for grouping ingredients)
-    Subtraction
*    Multiplication
.    Member access
```

## 2.8 Punctuation

```
{    }    Block delimiters
[    ]    List delimiters
:         Property assignment
,         List separator (optional in multi-line lists)
```
