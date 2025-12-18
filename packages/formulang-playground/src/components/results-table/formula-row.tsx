import { Fragment } from "react";
import {
  Play,
  ChevronDown,
  ChevronRight,
  Loader2,
  AlertCircle,
  AlertTriangle,
  CheckCircle2,
  Minus,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { TableCell, TableRow } from "@/components/ui/table";
import { type ColumnId } from "@/hooks/use-table-config";
import { IngredientsSection } from "./ingredients-section";
import { NutrientsSection } from "./nutrients-section";
import { SensitivitySection } from "./sensitivity-section";
import { ViolationsSection } from "./violations-section";
import type { SolveResult } from "./types";

interface FormulaRowProps {
  formulaId: string;
  result: SolveResult | undefined;
  isExpanded: boolean;
  isLoading: boolean;
  identifierCols: ColumnId[];
  colCount: number;
  wasmReady: boolean;
  onToggleExpand: () => void;
  onSolve: (e: React.MouseEvent) => void;
}

export function FormulaRow({
  formulaId,
  result,
  isExpanded,
  isLoading,
  identifierCols,
  colCount,
  wasmReady,
  onToggleExpand,
  onSolve,
}: FormulaRowProps) {
  return (
    <Fragment>
      <TableRow
        className="cursor-pointer hover:bg-muted/50"
        onClick={onToggleExpand}
      >
        <TableCell className="py-2">
          {isExpanded ? (
            <ChevronDown className="h-4 w-4 text-muted-foreground" />
          ) : (
            <ChevronRight className="h-4 w-4 text-muted-foreground" />
          )}
        </TableCell>
        {identifierCols.map((col) => {
          switch (col) {
            case "id":
              return <TableCell key={col} className="py-2 font-medium font-mono text-xs">{formulaId}</TableCell>;
            case "name":
              return <TableCell key={col} className="py-2">{result?.formulaName || "-"}</TableCell>;
            case "code":
              return <TableCell key={col} className="py-2 font-mono text-xs">{result?.formulaCode || "-"}</TableCell>;
            default:
              return null;
          }
        })}
        <TableCell className="py-2 text-center">
          {result ? (
            result.status === "optimal" ? (
              <Badge className="gap-1 bg-green-500/10 text-green-600 hover:bg-green-500/20 dark:text-green-400">
                <CheckCircle2 className="h-3 w-3" />
                Optimal
              </Badge>
            ) : result.status === "infeasible" ? (
              <Badge
                variant="outline"
                className="gap-1 border-amber-500/50 text-amber-600 dark:text-amber-400"
              >
                <AlertTriangle className="h-3 w-3" />
                Suboptimal
              </Badge>
            ) : (
              <Badge variant="destructive" className="gap-1">
                <AlertCircle className="h-3 w-3" />
                Error
              </Badge>
            )
          ) : (
            <span className="text-muted-foreground">
              <Minus className="mx-auto h-4 w-4" />
            </span>
          )}
        </TableCell>
        <TableCell className="py-2 text-right tabular-nums">
          {result && result.status !== "error" ? (
            <>
              {result.batchSize.toLocaleString()}
              <span className="text-muted-foreground"> kg</span>
            </>
          ) : (
            <span className="text-muted-foreground">—</span>
          )}
        </TableCell>
        <TableCell className="py-2 text-right tabular-nums">
          {result && result.status !== "error" ? (
            <span className="text-primary font-medium">
              ${result.totalCost.toFixed(2)}
            </span>
          ) : (
            <span className="text-muted-foreground">—</span>
          )}
        </TableCell>
        <TableCell className="py-2 text-right tabular-nums">
          {result && result.status !== "error" && result.batchSize > 0 ? (
            `$${(result.totalCost / result.batchSize).toFixed(2)}`
          ) : (
            <span className="text-muted-foreground">—</span>
          )}
        </TableCell>
        <TableCell className="py-2 text-right">
          <Button
            size="sm"
            variant="ghost"
            className="h-6 w-6 p-0"
            onClick={onSolve}
            disabled={isLoading || !wasmReady}
            title={`Solve ${formulaId}`}
          >
            {isLoading ? (
              <Loader2 className="h-3 w-3 animate-spin" />
            ) : (
              <Play className="h-3 w-3" />
            )}
          </Button>
        </TableCell>
      </TableRow>

      {/* Expanded content - sub-sections as table rows */}
      {isExpanded && (
        <>
          {!result && (
            <TableRow>
              <TableCell colSpan={colCount} className="py-4 text-center text-sm text-muted-foreground bg-card">
                Click <strong>Solve</strong> to find the optimal formulation.
              </TableCell>
            </TableRow>
          )}

          {result && result.status === "error" && (
            <TableRow>
              <TableCell colSpan={colCount} className="bg-card">
                <div className="flex items-center gap-2 rounded-lg border border-red-500/20 bg-red-500/10 p-3 text-red-600 dark:text-red-400">
                  <AlertCircle className="h-5 w-5" />
                  <span className="font-medium">Error solving formula</span>
                </div>
              </TableCell>
            </TableRow>
          )}

          {result && result.status !== "error" && (
            <>
              {result.violations && result.violations.length > 0 && (
                <ViolationsSection violations={result.violations} colSpan={colCount} className="bg-card" />
              )}

              {result.ingredients.length > 0 && (
                <IngredientsSection ingredients={result.ingredients} colSpan={colCount} className="bg-card" />
              )}

              {result.nutrients.length > 0 && (
                <NutrientsSection nutrients={result.nutrients} colSpan={colCount} className="bg-card" />
              )}

              {result.analysis && result.analysis.shadowPrices.length > 0 && (
                <SensitivitySection shadowPrices={result.analysis.shadowPrices} colSpan={colCount} className="bg-card" />
              )}
            </>
          )}
        </>
      )}
    </Fragment>
  );
}
