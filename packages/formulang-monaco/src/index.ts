import type * as Monaco from 'monaco-editor';

export const LANGUAGE_ID = 'formulang';

/**
 * Language configuration for Formulang
 */
export const languageConfiguration: Monaco.languages.LanguageConfiguration = {
  comments: {
    lineComment: '//',
    blockComment: ['/*', '*/'],
  },
  brackets: [
    ['{', '}'],
    ['[', ']'],
    ['(', ')'],
  ],
  autoClosingPairs: [
    { open: '{', close: '}' },
    { open: '[', close: ']' },
    { open: '(', close: ')' },
    { open: '"', close: '"' },
  ],
  surroundingPairs: [
    { open: '{', close: '}' },
    { open: '[', close: ']' },
    { open: '(', close: ')' },
    { open: '"', close: '"' },
  ],
  folding: {
    markers: {
      start: /^\s*\/\/\s*#?region\b/,
      end: /^\s*\/\/\s*#?endregion\b/,
    },
  },
};

/**
 * Monarch tokenizer for Formulang syntax highlighting
 */
export const monarchTokensProvider: Monaco.languages.IMonarchLanguage = {
  defaultToken: '',
  tokenPostfix: '.fm',

  // Declaration keywords - each gets a unique token
  declarations: ['nutrient', 'ingredient', 'formula'],

  // Modifiers
  modifiers: ['template', 'import'],

  // Constraint keywords
  constraints: ['min', 'max', 'as'],

  // Block keywords
  blocks: ['nutrients', 'nuts', 'ingredients', 'ings'],

  // Property names
  properties: ['name', 'code', 'desc', 'description', 'cost', 'batch', 'batch_size', 'unit'],

  operators: ['+', '-', '*', '/'],

  symbols: /[+\-*\/]/,

  escapes: /\\(?:[abfnrtv\\"']|x[0-9A-Fa-f]{1,4}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,

  tokenizer: {
    root: [
      // Comments
      [/\/\/.*$/, 'comment'],
      [/\/\*/, 'comment', '@comment'],

      // Declaration keywords - transition to state that captures the declaration name
      [/\b(template)\b/, 'keyword.modifier', '@templateDecl'],
      [/\b(import)\b/, 'keyword.import'],
      [/\b(nutrient)\b/, 'keyword.nutrient', '@declarationName'],
      [/\b(ingredient)\b/, 'keyword.ingredient', '@declarationName'],
      [/\b(formula)\b/, 'keyword.formula', '@declarationName'],

      // 'as' keyword - transition to alias state to capture the alias name
      [/\b(as)\b/, 'keyword.constraint', '@alias'],

      // Constraint keywords
      [/\b(min|max)\b/, 'keyword.constraint'],

      // Block keywords
      [/\b(nutrients|nuts|ingredients|ings)\b/, 'keyword.block'],

      // Property names (when followed by value)
      [/\b(name|code|desc|description|cost|batch|batch_size|unit)\b/, 'variable.property'],

      // Percentage symbol after number
      [/(\d+\.?\d*)(%?)/, ['number', 'number.percent']],

      // Base identifier followed by dot (e.g., `formula.` in `formula.nutrients.protein`)
      [/([a-zA-Z_]\w*)(\.)/,  ['class', 'delimiter.dot']],

      // Regular identifiers
      [/[a-zA-Z_]\w*/, 'variable'],

      // Whitespace
      { include: '@whitespace' },

      // Delimiters and operators
      [/[{}]/, 'delimiter.bracket'],
      [/[()]/, 'delimiter.parenthesis'],
      [/[\[\]]/, 'delimiter.square'],
      [/@symbols/, 'operator'],

      // Strings
      [/"([^"\\]|\\.)*$/, 'string.invalid'],
      [/"/, 'string', '@string'],
    ],

    // State for capturing the type keyword after 'template' (e.g., 'formula' or 'ingredient')
    templateDecl: [
      [/[ \t]+/, ''], // Skip whitespace
      [/\b(formula)\b/, 'keyword.formula', '@declarationName'],
      [/\b(ingredient)\b/, 'keyword.ingredient', '@declarationName'],
      ['', '', '@pop'], // Fallback - return to root
    ],

    // State for capturing declaration name after nutrient/ingredient/formula keywords
    declarationName: [
      [/[ \t]+/, ''], // Skip whitespace
      [/[a-zA-Z_]\w*/, 'class', '@pop'], // Declaration name gets 'class' token (gold)
      ['', '', '@pop'], // Fallback - return to root if no identifier
    ],

    // State for capturing alias name after 'as' keyword
    alias: [
      [/[ \t]+/, ''], // Skip whitespace
      [/[a-zA-Z_]\w*/, 'type', '@pop'], // Alias name gets 'type' token (teal)
      ['', '', '@pop'], // Fallback - return to root if no identifier
    ],

    comment: [
      [/[^\/*]+/, 'comment'],
      [/\*\//, 'comment', '@pop'],
      [/[\/*]/, 'comment'],
    ],

    string: [
      [/[^\\"]+/, 'string'],
      [/@escapes/, 'string.escape'],
      [/\\./, 'string.escape.invalid'],
      [/"/, 'string', '@pop'],
    ],

    whitespace: [[/[ \t\r\n]+/, '']],
  },
};

/**
 * Token types used by the Formulang tokenizer.
 * These are the semantic token names that themes should define rules for.
 */
export const TOKEN_TYPES = {
  // Declaration keywords
  KEYWORD_NUTRIENT: 'keyword.nutrient',
  KEYWORD_INGREDIENT: 'keyword.ingredient',
  KEYWORD_FORMULA: 'keyword.formula',
  KEYWORD_MODIFIER: 'keyword.modifier',
  KEYWORD_IMPORT: 'keyword.import',
  KEYWORD_BLOCK: 'keyword.block',
  KEYWORD_CONSTRAINT: 'keyword.constraint',

  // Variables and properties
  VARIABLE: 'variable',
  VARIABLE_PROPERTY: 'variable.property',

  // Types (alias names after 'as')
  TYPE: 'type',

  // Classes (base identifiers before dots, e.g., 'formula' in 'formula.nutrients')
  CLASS: 'class',

  // Literals
  NUMBER: 'number',
  NUMBER_PERCENT: 'number.percent',
  STRING: 'string',

  // Delimiters
  DELIMITER_BRACKET: 'delimiter.bracket',
  DELIMITER_DOT: 'delimiter.dot',
  DELIMITER_PARENTHESIS: 'delimiter.parenthesis',
  DELIMITER_SQUARE: 'delimiter.square',
  OPERATOR: 'operator',

  // Comments
  COMMENT: 'comment',
} as const;

/**
 * Create a Formulang theme for Monaco editor.
 *
 * @param isDark - Whether to create a dark theme
 * @param colors - Color configuration matching your app's theme
 */
export function createFormulangTheme(
  isDark: boolean,
  colors: {
    background: string;
    foreground: string;
    primary: string;
    primaryDimmed: string;
    secondary: string;
    muted: string;
    mutedForeground: string;
    accent: string;
    string: string;
  }
): Monaco.editor.IStandaloneThemeData {
  return {
    base: isDark ? 'vs-dark' : 'vs',
    inherit: true,
    rules: [
      // Declarations - primary color
      { token: TOKEN_TYPES.KEYWORD_NUTRIENT, foreground: colors.primary, fontStyle: 'bold' },
      { token: TOKEN_TYPES.KEYWORD_INGREDIENT, foreground: colors.primary, fontStyle: 'bold' },
      { token: TOKEN_TYPES.KEYWORD_FORMULA, foreground: colors.primary, fontStyle: 'bold' },
      { token: TOKEN_TYPES.KEYWORD_MODIFIER, foreground: colors.primary, fontStyle: 'bold' },
      { token: TOKEN_TYPES.KEYWORD_IMPORT, foreground: colors.primary, fontStyle: 'italic' },

      // Block sections - primary dimmed
      { token: TOKEN_TYPES.KEYWORD_BLOCK, foreground: colors.primaryDimmed },

      // Logic - secondary
      { token: TOKEN_TYPES.KEYWORD_CONSTRAINT, foreground: colors.secondary },
      { token: TOKEN_TYPES.OPERATOR, foreground: colors.secondary },

      // Identifiers - foreground
      { token: TOKEN_TYPES.VARIABLE, foreground: colors.foreground },
      { token: TOKEN_TYPES.VARIABLE_PROPERTY, foreground: colors.foreground },

      // Types (alias names) - use accent color
      { token: TOKEN_TYPES.TYPE, foreground: colors.accent },

      // Classes (base identifiers before dots) - use primary dimmed
      { token: TOKEN_TYPES.CLASS, foreground: colors.primaryDimmed },

      // Numbers - accent/muted
      { token: TOKEN_TYPES.NUMBER, foreground: colors.accent },
      { token: TOKEN_TYPES.NUMBER_PERCENT, foreground: colors.accent },

      // Strings - muted green
      { token: TOKEN_TYPES.STRING, foreground: colors.string },

      // Braces - muted foreground
      { token: TOKEN_TYPES.DELIMITER_BRACKET, foreground: colors.mutedForeground },
      { token: TOKEN_TYPES.DELIMITER_DOT, foreground: colors.mutedForeground },
      { token: TOKEN_TYPES.DELIMITER_PARENTHESIS, foreground: colors.mutedForeground },
      { token: TOKEN_TYPES.DELIMITER_SQUARE, foreground: colors.mutedForeground },

      // Comments
      { token: TOKEN_TYPES.COMMENT, foreground: colors.mutedForeground, fontStyle: 'italic' },
    ],
    colors: {
      'editor.background': colors.background,
      'editor.foreground': colors.foreground,
      'editor.lineHighlightBackground': isDark ? '#ffffff08' : '#00000008',
      'editorLineNumber.foreground': colors.mutedForeground,
      'editorLineNumber.activeForeground': colors.foreground,
    },
  };
}

/**
 * Static completion items for Formulang keywords
 */
const completionItems: Monaco.languages.CompletionItem[] = [
  {
    label: 'nutrient',
    kind: 14, // Keyword
    insertText: 'nutrient ${1:name} {\n\tname "${2:Display Name}"\n\tcode "${3}"\n\tunit "${4:%}"\n}',
    insertTextRules: 4, // InsertAsSnippet
    documentation: 'Define a nutrient that can be tracked in ingredients',
    detail: 'Nutrient definition',
  },
  {
    label: 'ingredient',
    kind: 14,
    insertText:
      'ingredient ${1:name} {\n\tname "${2:Display Name}"\n\tcode "${3}"\n\tcost ${4:0}\n\tnutrients {\n\t\t${5:nutrient} ${6:0}\n\t}\n}',
    insertTextRules: 4,
    documentation: 'Define an ingredient with cost and nutrient composition',
    detail: 'Ingredient definition',
  },
  {
    label: 'formula',
    kind: 14,
    insertText:
      'formula ${1:name} {\n\tname "${2:Display Name}"\n\tcode "${3}"\n\tdesc "${4}"\n\tbatch ${5:1000}\n\n\tnutrients {\n\t\t${6:nutrient} min ${7:0}\n\t}\n\n\tingredients {\n\t\t${8:ingredient}\n\t}\n}',
    insertTextRules: 4,
    documentation: 'Define a formula with nutrient requirements and ingredient constraints',
    detail: 'Formula definition',
  },
  {
    label: 'import',
    kind: 14,
    insertText: 'import "${1:./file.fm}"',
    insertTextRules: 4,
    documentation: 'Import definitions from another file',
    detail: 'Import statement',
  },
  {
    label: 'template formula',
    kind: 14,
    insertText: 'template formula ${1:name} {\n\tnutrients {\n\t\t${2}\n\t}\n\tingredients {\n\t\t${3}\n\t}\n}',
    insertTextRules: 4,
    documentation: 'Define a template formula for composition (not solvable)',
    detail: 'Template formula',
  },
  {
    label: 'template ingredient',
    kind: 14,
    insertText: 'template ingredient ${1:name} {\n\tnutrients {\n\t\t${2}\n\t}\n}',
    insertTextRules: 4,
    documentation: 'Define a template ingredient for composition (no cost required)',
    detail: 'Template ingredient',
  },
  {
    label: 'min',
    kind: 14,
    insertText: 'min ${1:0}',
    insertTextRules: 4,
    documentation: 'Set minimum constraint value',
    detail: 'Minimum bound',
  },
  {
    label: 'max',
    kind: 14,
    insertText: 'max ${1:0}',
    insertTextRules: 4,
    documentation: 'Set maximum constraint value',
    detail: 'Maximum bound',
  },
  {
    label: 'as',
    kind: 14,
    insertText: 'as ${1:alias_name}',
    insertTextRules: 4,
    documentation: 'Name a constraint expression for referencing',
    detail: 'Constraint alias',
  },
  {
    label: 'nutrients',
    kind: 14,
    insertText: 'nutrients {\n\t${1}\n}',
    insertTextRules: 4,
    documentation: 'Nutrient constraints block',
    detail: 'Nutrients block',
    sortText: '0nutrients', // Sort before 'nuts'
  },
  {
    label: 'ingredients',
    kind: 14,
    insertText: 'ingredients {\n\t${1}\n}',
    insertTextRules: 4,
    documentation: 'Ingredient constraints block',
    detail: 'Ingredients block',
    sortText: '0ingredients', // Sort before 'ings'
  },
  {
    label: 'nuts',
    kind: 14,
    insertText: 'nuts {\n\t${1}\n}',
    insertTextRules: 4,
    documentation: 'Nutrient constraints block (short form)',
    detail: 'Nutrients block (alias)',
    sortText: '1nuts',
  },
  {
    label: 'ings',
    kind: 14,
    insertText: 'ings {\n\t${1}\n}',
    insertTextRules: 4,
    documentation: 'Ingredient constraints block (short form)',
    detail: 'Ingredients block (alias)',
    sortText: '1ings',
  },
] as Monaco.languages.CompletionItem[];

// WASM functions interface
interface WasmFunctions {
  get_completions: (source: string, position: number) => WasmCompletion[];
  get_hover: (source: string, position: number) => WasmHover | null;
  validate: (source: string) => WasmDiagnostic[];
}

interface WasmCompletion {
  label: string;
  kind: string;
  detail?: string;
  insert_text: string;
}

interface WasmHover {
  contents: string;
  start: number;
  end: number;
}

interface WasmDiagnostic {
  start: number;
  end: number;
  severity: string;
  message: string;
}

// Store WASM functions when initialized
let wasmFunctions: WasmFunctions | null = null;

/**
 * Set the WASM functions for enhanced language features
 */
export function setWasmFunctions(fns: WasmFunctions): void {
  wasmFunctions = fns;
}

/**
 * Completion provider for Formulang
 *
 * Uses WASM-powered completions as the source of truth when available.
 * Only falls back to static completions when WASM isn't initialized.
 */
export function createCompletionProvider(): Monaco.languages.CompletionItemProvider {
  return {
    triggerCharacters: ['.', ' '],
    provideCompletionItems(model, position) {
      const word = model.getWordUntilPosition(position);
      const range = {
        startLineNumber: position.lineNumber,
        endLineNumber: position.lineNumber,
        startColumn: word.startColumn,
        endColumn: word.endColumn,
      };

      // Use WASM completions as source of truth when available
      if (wasmFunctions) {
        try {
          const source = model.getValue();
          const offset = model.getOffsetAt(position);
          const wasmCompletions = wasmFunctions.get_completions(source, offset);

          if (Array.isArray(wasmCompletions)) {
            const suggestions = wasmCompletions.map((c) => ({
              label: c.label,
              kind: c.kind === 'keyword' ? 14 : c.kind === 'property' ? 10 : 5,
              insertText: c.insert_text,
              insertTextRules: c.insert_text.includes('$') ? 4 : 0,
              documentation: c.detail || undefined,
              detail: c.kind,
              range,
            }));
            return { suggestions };
          }
        } catch (e) {
          console.error('WASM completion error:', e);
        }
      }

      // Fallback to static completions only when WASM unavailable
      const suggestions = completionItems.map((item) => ({
        ...item,
        range,
      }));
      return { suggestions };
    },
  };
}

/**
 * Hover provider for Formulang
 */
export function createHoverProvider(): Monaco.languages.HoverProvider {
  const hoverInfo: Record<string, string> = {
    nutrient: 'Defines a nutrient that can be tracked in ingredients and constrained in formulas.',
    ingredient: 'Defines a feed ingredient with cost and nutrient composition.',
    formula:
      'Defines a feed formula with nutrient requirements and ingredient constraints for least-cost formulation.',
    import: 'Imports definitions from another .fm file.',
    min: 'Sets a minimum bound for a constraint.',
    max: 'Sets a maximum bound for a constraint.',
    nutrients: 'Block containing nutrient constraints (alias: nuts).',
    nuts: 'Block containing nutrient constraints (short for nutrients).',
    ingredients: 'Block containing ingredient constraints (alias: ings).',
    ings: 'Block containing ingredient constraints (short for ingredients).',
    as: 'Names a constraint expression for referencing. Example: `protein min 18 as min_protein`',
    batch_size: 'The total weight of the formula batch (alias: batch).',
    batch: 'The total weight of the formula batch (short for batch_size).',
    description: 'Description text (alias: desc).',
    desc: 'Description text (short for description).',
    cost: 'The cost per unit of the ingredient.',
    name: 'Display name for the entity.',
    code: 'Identifier/SKU code for the entity.',
    unit: 'Unit of measurement for the nutrient.',
  };

  return {
    provideHover(model, position) {
      const word = model.getWordAtPosition(position);
      if (!word) return null;

      // Try static hover info first
      const info = hoverInfo[word.word];
      if (info) {
        return {
          contents: [{ value: `**${word.word}**\n\n${info}` }],
          range: {
            startLineNumber: position.lineNumber,
            endLineNumber: position.lineNumber,
            startColumn: word.startColumn,
            endColumn: word.endColumn,
          },
        };
      }

      // Try WASM hover if available
      if (wasmFunctions) {
        try {
          const source = model.getValue();
          const offset = model.getOffsetAt(position);
          const hover = wasmFunctions.get_hover(source, offset);

          if (hover) {
            const startPos = model.getPositionAt(hover.start);
            const endPos = model.getPositionAt(hover.end);
            return {
              contents: [{ value: hover.contents }],
              range: {
                startLineNumber: startPos.lineNumber,
                endLineNumber: endPos.lineNumber,
                startColumn: startPos.column,
                endColumn: endPos.column,
              },
            };
          }
        } catch (e) {
          console.error('WASM hover error:', e);
        }
      }

      return null;
    },
  };
}

/**
 * Create a diagnostics updater that validates code and sets markers
 */
export function createDiagnosticsUpdater(monaco: typeof Monaco) {
  return (model: Monaco.editor.ITextModel) => {
    if (!wasmFunctions) return;
    if (model.getLanguageId() !== LANGUAGE_ID) return;

    try {
      const source = model.getValue();
      const diagnostics = wasmFunctions.validate(source);

      const markers: Monaco.editor.IMarkerData[] = [];

      if (Array.isArray(diagnostics)) {
        for (const d of diagnostics) {
          const startPos = model.getPositionAt(d.start);
          const endPos = model.getPositionAt(d.end);

          markers.push({
            severity: d.severity === 'error' ? 8 : 4, // Error or Warning
            message: d.message,
            startLineNumber: startPos.lineNumber,
            startColumn: startPos.column,
            endLineNumber: endPos.lineNumber,
            endColumn: endPos.column,
          });
        }
      }

      monaco.editor.setModelMarkers(model, LANGUAGE_ID, markers);
    } catch (e) {
      console.error('WASM validation error:', e);
    }
  };
}

/**
 * Register the Formulang language with Monaco
 */
export const THEME_DARK = 'formulang-dark';
export const THEME_LIGHT = 'formulang-light';

export function registerFormulang(monaco: typeof Monaco): void {
  // Register the language
  monaco.languages.register({
    id: LANGUAGE_ID,
    extensions: ['.fm'],
    aliases: ['Formulang', 'formulang'],
    mimetypes: ['text/x-formulang'],
  });

  // Set language configuration
  monaco.languages.setLanguageConfiguration(LANGUAGE_ID, languageConfiguration);

  // Set monarch tokens provider for syntax highlighting
  monaco.languages.setMonarchTokensProvider(LANGUAGE_ID, monarchTokensProvider);

  // Register completion provider
  monaco.languages.registerCompletionItemProvider(LANGUAGE_ID, createCompletionProvider());

  // Register hover provider
  monaco.languages.registerHoverProvider(LANGUAGE_ID, createHoverProvider());
}

/**
 * Register Formulang with WASM-powered features
 */
export function registerFormulangWithWasm(
  monaco: typeof Monaco,
  wasm: WasmFunctions
): { updateDiagnostics: (model: Monaco.editor.ITextModel) => void } {
  // Set WASM functions
  setWasmFunctions(wasm);

  // Register basic language features
  registerFormulang(monaco);

  // Return diagnostics updater for the caller to use
  return {
    updateDiagnostics: createDiagnosticsUpdater(monaco),
  };
}
