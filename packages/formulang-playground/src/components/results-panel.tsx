import { useCallback } from "react";
import { Download, FlaskConical, Loader2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { ResultsTable, type SolveResult } from "./results-table";

export type { SolveResult };

export interface ParseResult {
  nutrients: number;
  ingredients: number;
  formulas: string[];
}

interface ResultsPanelProps {
  parseResult: ParseResult | null;
  solveResults: Record<string, SolveResult>;
  loadingFormulas: Set<string>;
  onSolve: (formulaName: string) => void;
  onSolveAll: () => void;
  onRefresh: () => void;
  wasmReady: boolean;
}

function generateCsv(
  formulas: string[],
  solveResults: Record<string, SolveResult>
): string {
  const rows: string[][] = [];

  rows.push([
    "formula_id",
    "formula_code",
    "formula_name",
    "status",
    "batch",
    "total_cost",
    "item_type",
    "item_id",
    "item_code",
    "item_name",
    "value",
    "unit",
    "cost",
  ]);

  for (const formula of formulas) {
    const result = solveResults[formula];
    if (!result) continue;

    // Add ingredient rows
    for (const ing of result.ingredients) {
      rows.push([
        formula,
        result.formulaCode || "",
        result.formulaName || formula,
        result.status,
        result.batchSize.toString(),
        result.totalCost.toFixed(2),
        "Ingredient",
        ing.id,
        ing.code || "",
        ing.name || "",
        ing.amount.toFixed(4),
        "",
        ing.cost.toFixed(2),
      ]);
    }

    // Add nutrient rows
    for (const nut of result.nutrients) {
      rows.push([
        formula,
        result.formulaCode || "",
        result.formulaName || formula,
        result.status,
        result.batchSize.toString(),
        result.totalCost.toFixed(2),
        "Nutrient",
        nut.id,
        nut.code || "",
        nut.name || "",
        nut.value.toFixed(4),
        nut.unit || "",
        "",
      ]);
    }
  }

  return rows
    .map((row) =>
      row.map((cell) => `"${cell.replace(/"/g, '""')}"`).join(",")
    )
    .join("\n");
}

export function ResultsPanel({
  parseResult,
  solveResults,
  loadingFormulas,
  onSolve,
  onSolveAll,
  onRefresh,
  wasmReady,
}: ResultsPanelProps) {
  const formulas = parseResult?.formulas || [];

  const handleDownload = useCallback(() => {
    const csv = generateCsv(formulas, solveResults);
    const blob = new Blob([csv], { type: "text/csv;charset=utf-8;" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = "formulas.csv";
    link.click();
    URL.revokeObjectURL(url);
  }, [formulas, solveResults]);

  const hasResults = Object.keys(solveResults).length > 0;

  return (
    <div className="flex w-1/2 flex-col overflow-hidden">
      <div className="flex h-10 shrink-0 items-center justify-between border-b bg-muted/50 px-4">
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <FlaskConical className="h-4 w-4" />
          <span>Results</span>
        </div>
        <div className="flex items-center gap-2">
          {!wasmReady && (
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <Loader2 className="h-3 w-3 animate-spin" />
              Loading solver...
            </div>
          )}
          {hasResults && (
            <Button
              variant="ghost"
              size="sm"
              className="h-7 gap-1.5 px-2 text-xs"
              onClick={handleDownload}
            >
              <Download className="h-3 w-3" />
              CSV
            </Button>
          )}
        </div>
      </div>
      <ScrollArea className="grid h-full flex-1">
        {formulas.length === 0 ? (
          <div className="flex h-64 flex-col items-center justify-center text-center text-muted-foreground">
            <FlaskConical className="mb-4 h-12 w-12 opacity-20" />
            <p className="text-sm">
              No formulas defined yet.
              <br />
              Add a <code className="text-xs">formula</code> block in the
              editor.
            </p>
          </div>
        ) : (
          <ResultsTable
            formulas={formulas}
            solveResults={solveResults}
            loadingFormulas={loadingFormulas}
            onSolve={onSolve}
            onSolveAll={onSolveAll}
            onRefresh={onRefresh}
            wasmReady={wasmReady}
          />
        )}
      </ScrollArea>
    </div>
  );
}
