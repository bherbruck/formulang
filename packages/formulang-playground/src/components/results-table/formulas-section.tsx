import {
  Play,
  Loader2,
  RefreshCw,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  TableHead,
  TableRow,
} from "@/components/ui/table";
import {
  useTableConfig,
  IDENTIFIER_COLUMNS,
  COLUMN_LABELS,
  COLUMN_CONFIG_KEY,
  type ColumnId,
} from "@/hooks/use-table-config";
import { SortableHeader } from "./sortable-header";
import { ColumnSettings } from "./column-settings";
import { FormulaRow } from "./formula-row";
import type { SolveResult } from "./types";

type FormulaSortKey = "id" | "name" | "code" | "status" | "batch" | "cost" | "costPerKg";
type SortDir = "asc" | "desc" | null;

interface FormulasSectionProps {
  formulas: string[];
  solveResults: Record<string, SolveResult>;
  loadingFormulas: Set<string>;
  expandedFormula: string | null;
  onToggleExpand: (name: string) => void;
  onSolve: (formulaName: string) => void;
  onSolveAll: () => void;
  onRefresh: () => void;
  wasmReady: boolean;
  sortKey: FormulaSortKey | null;
  sortDir: SortDir;
  onToggleSort: (key: FormulaSortKey) => void;
}

export function FormulasSection({
  formulas,
  solveResults,
  loadingFormulas,
  expandedFormula,
  onToggleExpand,
  onSolve,
  onSolveAll,
  onRefresh,
  wasmReady,
  sortKey,
  sortDir,
  onToggleSort,
}: FormulasSectionProps) {
  const { config, getOrderedColumns } = useTableConfig();

  // Get ordered identifier columns that are visible
  const visibleIdentifiers = getOrderedColumns((col) => {
    if (!IDENTIFIER_COLUMNS.includes(col)) return false;
    const key = COLUMN_CONFIG_KEY[col];
    return config[key] as boolean;
  });

  // Ensure at least one identifier column is shown (default to 'id')
  const identifierCols = visibleIdentifiers.length > 0 ? visibleIdentifiers : ["id" as ColumnId];

  // Count visible columns: chevron + identifiers + status + batch + cost + cost/kg + actions
  const colCount = 1 + identifierCols.length + 4 + 1;

  const handleSolve = (e: React.MouseEvent, formulaName: string) => {
    e.stopPropagation();
    onSolve(formulaName);
  };

  const getStatusOrder = (result: SolveResult | undefined): number => {
    if (!result) return 3;
    if (result.status === "optimal") return 0;
    if (result.status === "infeasible") return 1;
    return 2;
  };

  const sortedFormulas = [...formulas].sort((a, b) => {
    if (!sortKey || !sortDir) return 0;
    const mult = sortDir === "asc" ? 1 : -1;
    const resultA = solveResults[a];
    const resultB = solveResults[b];

    switch (sortKey) {
      case "id":
        return mult * a.localeCompare(b);
      case "name": {
        const nameA = resultA?.formulaName || a;
        const nameB = resultB?.formulaName || b;
        return mult * nameA.localeCompare(nameB);
      }
      case "code": {
        const codeA = resultA?.formulaCode || "";
        const codeB = resultB?.formulaCode || "";
        return mult * codeA.localeCompare(codeB);
      }
      case "status":
        return mult * (getStatusOrder(resultA) - getStatusOrder(resultB));
      case "batch": {
        const batchA = resultA?.batchSize ?? -1;
        const batchB = resultB?.batchSize ?? -1;
        return mult * (batchA - batchB);
      }
      case "cost": {
        const costA = resultA?.totalCost ?? -1;
        const costB = resultB?.totalCost ?? -1;
        return mult * (costA - costB);
      }
      case "costPerKg": {
        const cpkA = resultA && resultA.batchSize > 0 ? resultA.totalCost / resultA.batchSize : -1;
        const cpkB = resultB && resultB.batchSize > 0 ? resultB.totalCost / resultB.batchSize : -1;
        return mult * (cpkA - cpkB);
      }
      default:
        return 0;
    }
  });

  return (
    <>

      {/* Column headers - sticky */}
      <TableRow className="sticky top-0 z-10 bg-background border-0 shadow-[inset_0_-1px_0_hsl(var(--foreground)/0.15)]">
        <TableHead className="w-8 bg-background">
          <div className="flex items-center justify-center">
            <ColumnSettings />
          </div>
        </TableHead>
        {identifierCols.map((col) => (
          <SortableHeader
            key={col}
            onClick={() => onToggleSort(col as FormulaSortKey)}
            isActive={sortKey === col}
            direction={sortDir}
            className="bg-background"
          >
            {COLUMN_LABELS[col]}
          </SortableHeader>
        ))}
        <SortableHeader
          onClick={() => onToggleSort("status")}
          className="text-center bg-background"
          isActive={sortKey === "status"}
          direction={sortDir}
        >
          Status
        </SortableHeader>
        <SortableHeader
          onClick={() => onToggleSort("batch")}
          className="text-right bg-background"
          isActive={sortKey === "batch"}
          direction={sortDir}
        >
          Batch
        </SortableHeader>
        <SortableHeader
          onClick={() => onToggleSort("cost")}
          className="text-right bg-background"
          isActive={sortKey === "cost"}
          direction={sortDir}
        >
          Cost
        </SortableHeader>
        <SortableHeader
          onClick={() => onToggleSort("costPerKg")}
          className="text-right bg-background"
          isActive={sortKey === "costPerKg"}
          direction={sortDir}
        >
          Cost/kg
        </SortableHeader>
        <TableHead className="w-20 text-right bg-background">
          <div className="flex items-center justify-end gap-1">
            <Button
              size="sm"
              variant="ghost"
              className="h-6 w-6 p-0"
              onClick={onRefresh}
              disabled={loadingFormulas.size > 0}
              title="Clear results"
            >
              <RefreshCw className="h-3 w-3" />
            </Button>
            <Button
              size="sm"
              variant="ghost"
              className="h-6 w-6 p-0"
              onClick={onSolveAll}
              disabled={loadingFormulas.size > 0 || !wasmReady}
              title="Solve all formulas"
            >
              {loadingFormulas.size > 0 ? (
                <Loader2 className="h-3 w-3 animate-spin" />
              ) : (
                <Play className="h-3 w-3" />
              )}
            </Button>
          </div>
        </TableHead>
      </TableRow>

      {/* Formula rows */}
      {sortedFormulas.map((formulaId) => (
        <FormulaRow
          key={formulaId}
          formulaId={formulaId}
          result={solveResults[formulaId]}
          isExpanded={expandedFormula === formulaId}
          isLoading={loadingFormulas.has(formulaId)}
          identifierCols={identifierCols}
          colCount={colCount}
          wasmReady={wasmReady}
          onToggleExpand={() => onToggleExpand(formulaId)}
          onSolve={(e) => handleSolve(e, formulaId)}
        />
      ))}
    </>
  );
}

export type { FormulaSortKey };
