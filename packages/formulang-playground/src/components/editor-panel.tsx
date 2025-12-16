import { useCallback, useEffect, useRef } from "react";
import Editor, { type OnMount } from "@monaco-editor/react";
import { FileCode } from "lucide-react";
import { registerFormulangWithWasm } from "formulang-monaco";
import { get_completions, get_hover, validate } from "formulang-lang";

import { Badge } from "@/components/ui/badge";

interface ParseResult {
  nutrients: number;
  ingredients: number;
  formulas: string[];
}

interface EditorPanelProps {
  code: string;
  onCodeChange: (code: string) => void;
  parseResult: ParseResult | null;
  isDark: boolean;
}

export function EditorPanel({
  code,
  onCodeChange,
  parseResult,
  isDark,
}: EditorPanelProps) {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const modelRef = useRef<any>(null);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const updateDiagnosticsRef = useRef<((model: any) => void) | null>(null);

  const handleEditorMount: OnMount = useCallback((editor, monacoInstance) => {
    modelRef.current = editor.getModel();

    const { updateDiagnostics } = registerFormulangWithWasm(monacoInstance, {
      get_completions,
      get_hover,
      validate,
    });
    updateDiagnosticsRef.current = updateDiagnostics;

    if (modelRef.current) {
      updateDiagnostics(modelRef.current);
    }
  }, []);

  useEffect(() => {
    if (modelRef.current && updateDiagnosticsRef.current) {
      updateDiagnosticsRef.current(modelRef.current);
    }
  }, [code]);

  return (
    <div className="flex w-1/2 flex-col border-r">
      <div className="flex h-10 items-center justify-between border-b bg-muted/50 px-4">
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <FileCode className="h-4 w-4" />
          <span>formula.fm</span>
        </div>
        {parseResult && (
          <div className="flex items-center gap-1.5">
            <Badge variant="secondary" className="text-xs">
              {parseResult.nutrients} nutrients
            </Badge>
            <Badge variant="secondary" className="text-xs">
              {parseResult.ingredients} ingredients
            </Badge>
            <Badge variant="secondary" className="text-xs">
              {parseResult.formulas.length} formulas
            </Badge>
          </div>
        )}
      </div>
      <div className="flex-1">
        <Editor
          height="100%"
          defaultLanguage="formulang"
          value={code}
          onChange={(value) => onCodeChange(value || "")}
          onMount={handleEditorMount}
          theme={isDark ? "vs-dark" : "light"}
          options={{
            fontSize: 13,
            fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
            minimap: { enabled: false },
            scrollBeyondLastLine: false,
            padding: { top: 16 },
            lineNumbers: "on",
            renderLineHighlight: "line",
            cursorBlinking: "smooth",
            smoothScrolling: true,
            tabSize: 2,
          }}
        />
      </div>
    </div>
  );
}
