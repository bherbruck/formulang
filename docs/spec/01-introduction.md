# Formulang Specification

## 1. Introduction

Formulang is a domain-specific language for defining feed formulas and expressing least-cost formulation problems. It provides a declarative syntax for specifying nutrients, ingredients, and formulas with constraints that can be used by linear programming solvers to find optimal ingredient combinations.

### 1.1 Design Goals

- **Domain-friendly**: Syntax mirrors how nutritionists think about formulation (tables with min/max values)
- **Composable**: Formulas can inherit and override constraints from base formulas
- **Minimal**: Simple, consistent syntax with few special cases
- **Unit-agnostic**: Values are relative to batch size; users work in their preferred units

### 1.2 Overview

A Formulang program consists of:

- **Nutrients**: Named nutritional parameters (protein, energy, etc.)
- **Ingredients**: Raw materials with costs and nutrient compositions
- **Formulas**: Specifications combining ingredients with nutrient requirements and ingredient limits

```
nutrient protein {
  name: "Crude Protein"
}

ingredient corn {
  cost: 150
  nutrients {
    protein: 8.5
  }
}

formula starter {
  batch_size: 1000

  nutrients {
    protein min 18 max 22
  }

  ingredients {
    corn max 50%
  }
}
```
