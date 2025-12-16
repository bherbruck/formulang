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
  defaultToken: 'invalid',
  tokenPostfix: '.fm',

  keywords: ['nutrient', 'ingredient', 'formula', 'import', 'min', 'max'],

  typeKeywords: ['nutrients', 'nuts', 'ingredients', 'ings'],

  operators: ['+', '-', '*', '/', '%', '.'],

  symbols: /[=><!~?:&|+\-*\/\^%]+/,

  escapes: /\\(?:[abfnrtv\\"']|x[0-9A-Fa-f]{1,4}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,

  tokenizer: {
    root: [
      // Comments
      [/\/\/.*$/, 'comment'],
      [/\/\*/, 'comment', '@comment'],

      // Keywords
      [
        /[a-zA-Z_]\w*/,
        {
          cases: {
            '@keywords': 'keyword',
            '@typeKeywords': 'type',
            '@default': 'identifier',
          },
        },
      ],

      // Whitespace
      { include: '@whitespace' },

      // Delimiters and operators
      [/[{}()\[\]]/, '@brackets'],
      [/,/, 'delimiter'],
      [/@symbols/, 'operator'],

      // Numbers
      [/\d+\.?\d*%?/, 'number'],

      // Strings
      [/"([^"\\]|\\.)*$/, 'string.invalid'],
      [/"/, 'string', '@string'],
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

    whitespace: [[/[ \t\r\n]+/, 'white']],
  },
};

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
      'ingredient ${1:name} {\n\tname "${2:Display Name}"\n\tcode "${3}"\n\tcost ${4:0}\n\tnuts {\n\t\t${5:nutrient} ${6:0}\n\t}\n}',
    insertTextRules: 4,
    documentation: 'Define an ingredient with cost and nutrient composition',
    detail: 'Ingredient definition',
  },
  {
    label: 'formula',
    kind: 14,
    insertText:
      'formula ${1:name} {\n\tname "${2:Display Name}"\n\tcode "${3}"\n\tdesc "${4}"\n\tbatch ${5:1000}\n\n\tnuts {\n\t\t${6:nutrient} min ${7:0}\n\t}\n\n\tings {\n\t\t${8:ingredient}\n\t}\n}',
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
    label: 'nutrients',
    kind: 14,
    insertText: 'nuts {\n\t${1}\n}',
    insertTextRules: 4,
    documentation: 'Nutrient constraints block (alias: nuts)',
    detail: 'Nutrients block',
  },
  {
    label: 'nuts',
    kind: 14,
    insertText: 'nuts {\n\t${1}\n}',
    insertTextRules: 4,
    documentation: 'Nutrient constraints block (short for nutrients)',
    detail: 'Nutrients block',
  },
  {
    label: 'ingredients',
    kind: 14,
    insertText: 'ings {\n\t${1}\n}',
    insertTextRules: 4,
    documentation: 'Ingredient constraints block (alias: ings)',
    detail: 'Ingredients block',
  },
  {
    label: 'ings',
    kind: 14,
    insertText: 'ings {\n\t${1}\n}',
    insertTextRules: 4,
    documentation: 'Ingredient constraints block (short for ingredients)',
    detail: 'Ingredients block',
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

      // Start with static completions
      const suggestions = completionItems.map((item) => ({
        ...item,
        range,
      }));

      // Add dynamic completions from WASM if available
      if (wasmFunctions) {
        try {
          const source = model.getValue();
          const offset = model.getOffsetAt(position);
          const wasmCompletions = wasmFunctions.get_completions(source, offset);

          if (Array.isArray(wasmCompletions)) {
            for (const c of wasmCompletions) {
              // Skip if already in static completions
              if (suggestions.some(s => s.label === c.label)) continue;

              suggestions.push({
                label: c.label,
                kind: c.kind === 'keyword' ? 14 : 5, // Keyword or Variable
                insertText: c.insert_text,
                insertTextRules: c.insert_text.includes('$') ? 4 : 0,
                documentation: c.detail || undefined,
                detail: c.kind,
                range: {
                  startLineNumber: range.startLineNumber,
                  endLineNumber: range.endLineNumber,
                  startColumn: range.startColumn,
                  endColumn: range.endColumn,
                },
              });
            }
          }
        } catch (e) {
          console.error('WASM completion error:', e);
        }
      }

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
