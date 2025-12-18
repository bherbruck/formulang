import type * as Monaco from "monaco-editor";
import { THEME_DARK, THEME_LIGHT } from "formulang-monaco";

/**
 * Register Formulang themes with Monaco using default VS Code colors
 */
export function registerFormulangThemes(monaco: typeof Monaco): void {
  // Dark theme - VS Code dark defaults
  monaco.editor.defineTheme(THEME_DARK, {
    base: "vs-dark",
    inherit: true,
    rules: [
      // Keywords - VS Code blue
      { token: "keyword", foreground: "569CD6" },
      { token: "keyword.nutrient", foreground: "569CD6" },
      { token: "keyword.ingredient", foreground: "569CD6" },
      { token: "keyword.formula", foreground: "569CD6" },
      { token: "keyword.modifier", foreground: "569CD6" },
      { token: "keyword.block", foreground: "569CD6" },
      { token: "keyword.constraint", foreground: "569CD6" },
      { token: "keyword.import", foreground: "C586C0" },

      // Variables - VS Code light blue
      { token: "variable", foreground: "9CDCFE" },
      { token: "variable.property", foreground: "9CDCFE" },

      // Types/Aliases - VS Code teal (used for `as` alias names)
      { token: "type", foreground: "4EC9B0" },

      // Classes - VS Code gold (used for base identifiers like `formula.`)
      { token: "class", foreground: "DCDCAA" },

      // Numbers - VS Code light green
      { token: "number", foreground: "B5CEA8" },
      { token: "number.percent", foreground: "B5CEA8" },

      // Strings - VS Code orange
      { token: "string", foreground: "CE9178" },

      // Operators
      { token: "operator", foreground: "D4D4D4" },

      // Comments - VS Code green
      { token: "comment", foreground: "6A9955", fontStyle: "italic" },
    ],
    colors: {},
  });

  // Light theme - VS Code light defaults
  monaco.editor.defineTheme(THEME_LIGHT, {
    base: "vs",
    inherit: true,
    rules: [
      // Keywords - VS Code blue
      { token: "keyword", foreground: "0000FF" },
      { token: "keyword.nutrient", foreground: "0000FF" },
      { token: "keyword.ingredient", foreground: "0000FF" },
      { token: "keyword.formula", foreground: "0000FF" },
      { token: "keyword.modifier", foreground: "0000FF" },
      { token: "keyword.block", foreground: "0000FF" },
      { token: "keyword.constraint", foreground: "0000FF" },
      { token: "keyword.import", foreground: "AF00DB" },

      // Variables - dark color
      { token: "variable", foreground: "001080" },
      { token: "variable.property", foreground: "001080" },

      // Types/Aliases - VS Code teal (used for `as` alias names)
      { token: "type", foreground: "267F99" },

      // Classes - VS Code dark gold (used for base identifiers like `formula.`)
      { token: "class", foreground: "795E26" },

      // Numbers - VS Code green
      { token: "number", foreground: "098658" },
      { token: "number.percent", foreground: "098658" },

      // Strings - VS Code red/brown
      { token: "string", foreground: "A31515" },

      // Operators
      { token: "operator", foreground: "000000" },

      // Comments - VS Code green
      { token: "comment", foreground: "008000", fontStyle: "italic" },
    ],
    colors: {},
  });
}
