# 6. Imports and Modules

Formulang supports modular organization through imports.

## 6.1 Import Statement

```
import_stmt := 'import' path ('as' identifier)?
path        := './'? (identifier '/')* identifier ('.fm')?
```

### Examples

```
import ./nutrients.fm
import ./ingredients/grains.fm
import ./bases/poultry.fm as poultry
```

## 6.2 Import Resolution

Paths are resolved relative to the importing file:

```
project/
  nutrients.fm
  ingredients/
    grains.fm
    proteins.fm
  formulas/
    poultry/
      starter.fm     <- import ../ingredients/grains.fm
```

## 6.3 Namespacing

Imported declarations are accessed via the module name (filename without extension, or alias):

```
// nutrients.fm
nutrient protein {}
nutrient energy {}

// formula.fm
import ./nutrients.fm

ingredient corn {
  nutrients {
    nutrients.protein 8.5    // Qualified reference
    nutrients.energy 3350
  }
}
```

With alias:

```
import ./nutrients.fm as n

ingredient corn {
  nutrients {
    n.protein 8.5
    n.energy 3350
  }
}
```

## 6.4 Direct Imports

To import declarations directly into the current namespace:

```
import ./nutrients.fm { protein, energy }

ingredient corn {
  nutrients {
    protein 8.5    // Direct reference
    energy 3350
  }
}
```

Import all:

```
import ./nutrients.fm { * }
```

## 6.5 Re-exports

A module can re-export imported declarations:

```
// all_nutrients.fm
import ./amino_acids.fm { * }
import ./minerals.fm { * }
import ./vitamins.fm { * }

// Now other files can import all_nutrients.fm to get everything
```
