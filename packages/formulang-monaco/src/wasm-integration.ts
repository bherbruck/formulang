import type * as Monaco from 'monaco-editor';
import { LANGUAGE_ID } from './index';

// Type definitions for the WASM module
interface FormulangWasm {
  parse(source: string): unknown;
  tokenize(source: string): TokenInfo[];
  validate(source: string): Diagnostic[];
  get_semantic_tokens(source: string): SemanticToken[];
  get_completions(source: string, position: number): Completion[];
  get_hover(source: string, position: number): HoverInfo | null;
}

interface TokenInfo {
  kind: string;
  text: string;
  start: number;
  end: number;
}

interface SemanticToken {
  start: number;
  end: number;
  token_type: string;
}

interface Diagnostic {
  start: number;
  end: number;
  severity: string;
  message: string;
}

interface Completion {
  label: string;
  kind: string;
  detail?: string;
  insert_text: string;
}

interface HoverInfo {
  contents: string;
  start: number;
  end: number;
}

let wasmModule: FormulangWasm | null = null;

/**
 * Initialize the WASM module
 * Call this before using WASM-based features
 */
export async function initWasm(): Promise<void> {
  try {
    // Dynamic import of the WASM module
    const wasm = await import('formulang-lang');
    await wasm.default(); // Initialize WASM
    wasmModule = wasm as unknown as FormulangWasm;
  } catch (error) {
    console.warn('Failed to load Formulang WASM module:', error);
    console.warn('Falling back to basic language features');
  }
}

/**
 * Check if WASM module is available
 */
export function isWasmAvailable(): boolean {
  return wasmModule !== null;
}

/**
 * Convert byte offset to Monaco Position
 */
function offsetToPosition(model: Monaco.editor.ITextModel, offset: number): Monaco.Position {
  return model.getPositionAt(offset);
}

/**
 * Convert Monaco Position to byte offset
 */
function positionToOffset(model: Monaco.editor.ITextModel, position: Monaco.IPosition): number {
  return model.getOffsetAt(position);
}

/**
 * Create a diagnostics provider that uses the WASM parser
 */
export function createDiagnosticsProvider(
  monaco: typeof Monaco
): {
  validate: (model: Monaco.editor.ITextModel) => void;
  dispose: () => void;
} {
  const markers = new Map<string, Monaco.editor.IMarkerData[]>();
  let disposable: Monaco.IDisposable | null = null;

  function validate(model: Monaco.editor.ITextModel): void {
    if (!wasmModule || model.getLanguageId() !== LANGUAGE_ID) return;

    try {
      const source = model.getValue();
      const diagnostics = wasmModule.validate(source);

      const monacoMarkers: Monaco.editor.IMarkerData[] = diagnostics.map((d) => {
        const startPos = offsetToPosition(model, d.start);
        const endPos = offsetToPosition(model, d.end);

        return {
          severity:
            d.severity === 'error'
              ? monaco.MarkerSeverity.Error
              : d.severity === 'warning'
                ? monaco.MarkerSeverity.Warning
                : monaco.MarkerSeverity.Info,
          message: d.message,
          startLineNumber: startPos.lineNumber,
          startColumn: startPos.column,
          endLineNumber: endPos.lineNumber,
          endColumn: endPos.column,
        };
      });

      monaco.editor.setModelMarkers(model, LANGUAGE_ID, monacoMarkers);
      markers.set(model.uri.toString(), monacoMarkers);
    } catch (error) {
      console.error('Error validating Formulang:', error);
    }
  }

  // Set up model change listener for all editors
  disposable = monaco.editor.onDidCreateModel((model: Monaco.editor.ITextModel) => {
    if (model.getLanguageId() === LANGUAGE_ID) {
      // Initial validation
      validate(model);

      // Validate on content change
      model.onDidChangeContent(() => {
        // Debounce validation
        setTimeout(() => validate(model), 500);
      });
    }
  });

  return {
    validate,
    dispose: () => {
      disposable?.dispose();
      markers.clear();
    },
  };
}

/**
 * Create a WASM-powered completion provider
 */
export function createWasmCompletionProvider(): Monaco.languages.CompletionItemProvider {
  return {
    provideCompletionItems(model, position) {
      if (!wasmModule) return { suggestions: [] };

      try {
        const source = model.getValue();
        const offset = positionToOffset(model, position);
        const completions = wasmModule.get_completions(source, offset);

        const word = model.getWordUntilPosition(position);
        const range = {
          startLineNumber: position.lineNumber,
          endLineNumber: position.lineNumber,
          startColumn: word.startColumn,
          endColumn: word.endColumn,
        };

        return {
          suggestions: completions.map((c) => ({
            label: c.label,
            kind: c.kind === 'keyword' ? 14 : 5, // Keyword or Field
            insertText: c.insert_text,
            insertTextRules: c.insert_text.includes('$') ? 4 : 0,
            detail: c.detail,
            range,
          })),
        };
      } catch (error) {
        console.error('Error getting completions:', error);
        return { suggestions: [] };
      }
    },
  };
}

/**
 * Create a WASM-powered hover provider
 */
export function createWasmHoverProvider(): Monaco.languages.HoverProvider {
  return {
    provideHover(model, position) {
      if (!wasmModule) return null;

      try {
        const source = model.getValue();
        const offset = positionToOffset(model, position);
        const hover = wasmModule.get_hover(source, offset);

        if (!hover) return null;

        const startPos = offsetToPosition(model, hover.start);
        const endPos = offsetToPosition(model, hover.end);

        return {
          contents: [{ value: hover.contents }],
          range: {
            startLineNumber: startPos.lineNumber,
            startColumn: startPos.column,
            endLineNumber: endPos.lineNumber,
            endColumn: endPos.column,
          },
        };
      } catch (error) {
        console.error('Error getting hover info:', error);
        return null;
      }
    },
  };
}

/**
 * Register WASM-powered language features
 * This should be called after initWasm() and registerFormulang()
 */
export function registerWasmFeatures(monaco: typeof Monaco): Monaco.IDisposable[] {
  const disposables: Monaco.IDisposable[] = [];

  if (!wasmModule) {
    console.warn('WASM module not loaded, skipping WASM features');
    return disposables;
  }

  // Register WASM-powered completion provider (higher priority)
  disposables.push(
    monaco.languages.registerCompletionItemProvider(LANGUAGE_ID, {
      ...createWasmCompletionProvider(),
      triggerCharacters: ['.', ' '],
    })
  );

  // Register WASM-powered hover provider
  disposables.push(monaco.languages.registerHoverProvider(LANGUAGE_ID, createWasmHoverProvider()));

  // Set up diagnostics
  const diagnostics = createDiagnosticsProvider(monaco);
  disposables.push({ dispose: diagnostics.dispose });

  // Validate all existing Formulang models
  monaco.editor.getModels().forEach((model) => {
    if (model.getLanguageId() === LANGUAGE_ID) {
      diagnostics.validate(model);
    }
  });

  return disposables;
}

export default {
  initWasm,
  isWasmAvailable,
  createDiagnosticsProvider,
  createWasmCompletionProvider,
  createWasmHoverProvider,
  registerWasmFeatures,
};
