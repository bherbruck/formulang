import { useState } from "react";
import {
  Play,
  ChevronDown,
  ChevronRight,
  Loader2,
  AlertCircle,
  AlertTriangle,
  CheckCircle2,
  Minus,
  ArrowUpDown,
  ArrowUp,
  ArrowDown,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { SolveResultsTable } from "./solve-results-table";
import { SensitivityAnalysis } from "./sensitivity-analysis";
import { ConstraintViolations } from "./constraint-violations";
import type { SolveResult } from "./results-panel";

type SortKey = "name" | "status" | "batch" | "cost" | "costPerKg";
type SortDir = "asc" | "desc";

interface FormulasTableProps {
  formulas: string[];
  solveResults: Record<string, SolveResult>;
  loadingFormula: string | null;
  onSolve: (formulaName: string) => void;
  onSolveAll: () => void;
  wasmReady: boolean;
}

function SortableHeader({
  children,
  onClick,
  className = "",
  isActive,
  direction,
}: {
  children: React.ReactNode;
  onClick: () => void;
  className?: string;
  isActive?: boolean;
  direction?: "asc" | "desc";
}) {
  return (
    <TableHead
      className={`cursor-pointer hover:bg-muted/50 ${className}`}
      onClick={onClick}
    >
      <div className={`flex items-center gap-1 ${className.includes("text-right") ? "justify-end" : className.includes("text-center") ? "justify-center" : ""}`}>
        {children}
        {isActive ? (
          direction === "asc" ? (
            <ArrowUp className="h-3 w-3" />
          ) : (
            <ArrowDown className="h-3 w-3" />
          )
        ) : (
          <ArrowUpDown className="h-3 w-3 text-muted-foreground" />
        )}
      </div>
    </TableHead>
  );
}

export function FormulasTable({
  formulas,
  solveResults,
  loadingFormula,
  onSolve,
  onSolveAll,
  wasmReady,
}: FormulasTableProps) {
  const [expandedFormula, setExpandedFormula] = useState<string | null>(null);
  const [sortKey, setSortKey] = useState<SortKey>("name");
  const [sortDir, setSortDir] = useState<SortDir>("asc");

  const toggleExpanded = (name: string) => {
    setExpandedFormula((prev) => (prev === name ? null : name));
  };

  const handleSolve = (e: React.MouseEvent, formulaName: string) => {
    e.stopPropagation();
    onSolve(formulaName);
  };

  const toggleSort = (key: SortKey) => {
    if (sortKey === key) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortKey(key);
      setSortDir(key === "name" ? "asc" : "desc");
    }
  };

  const getStatusOrder = (result: SolveResult | undefined): number => {
    if (!result) return 3;
    if (result.status === "optimal") return 0;
    if (result.status === "infeasible") return 1;
    return 2;
  };

  const sortedFormulas = [...formulas].sort((a, b) => {
    const mult = sortDir === "asc" ? 1 : -1;
    const resultA = solveResults[a];
    const resultB = solveResults[b];

    switch (sortKey) {
      case "name":
        return mult * a.localeCompare(b);
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
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead className="w-8"></TableHead>
          <SortableHeader onClick={() => toggleSort("name")} isActive={sortKey === "name"} direction={sortDir}>
            Formula
          </SortableHeader>
          <SortableHeader onClick={() => toggleSort("status")} className="text-center" isActive={sortKey === "status"} direction={sortDir}>
            Status
          </SortableHeader>
          <SortableHeader onClick={() => toggleSort("batch")} className="text-right" isActive={sortKey === "batch"} direction={sortDir}>
            Batch
          </SortableHeader>
          <SortableHeader onClick={() => toggleSort("cost")} className="text-right" isActive={sortKey === "cost"} direction={sortDir}>
            Cost
          </SortableHeader>
          <SortableHeader onClick={() => toggleSort("costPerKg")} className="text-right" isActive={sortKey === "costPerKg"} direction={sortDir}>
            Cost/kg
          </SortableHeader>
          <TableHead className="w-28 text-right">
            <Button
              size="sm"
              variant="outline"
              className="h-6 gap-1 px-2 text-xs"
              onClick={onSolveAll}
              disabled={!!loadingFormula || !wasmReady}
            >
              {loadingFormula ? (
                <Loader2 className="h-3 w-3 animate-spin" />
              ) : (
                <Play className="h-3 w-3" />
              )}
              Solve All
            </Button>
          </TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {sortedFormulas.map((name) => {
          const result = solveResults[name];
          const isExpanded = expandedFormula === name;
          const isLoading = loadingFormula === name;

          return (
            <>
              <TableRow
                key={name}
                className="cursor-pointer hover:bg-muted/50"
                onClick={() => toggleExpanded(name)}
              >
                <TableCell className="py-2">
                  {isExpanded ? (
                    <ChevronDown className="h-4 w-4 text-muted-foreground" />
                  ) : (
                    <ChevronRight className="h-4 w-4 text-muted-foreground" />
                  )}
                </TableCell>
                <TableCell className="py-2 font-medium">{name}</TableCell>
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
                    variant={result ? "outline" : "default"}
                    className="h-7 gap-1.5 px-2"
                    onClick={(e) => handleSolve(e, name)}
                    disabled={isLoading || !wasmReady}
                  >
                    {isLoading ? (
                      <Loader2 className="h-3 w-3 animate-spin" />
                    ) : (
                      <Play className="h-3 w-3" />
                    )}
                    <span className="sr-only md:not-sr-only">Solve</span>
                  </Button>
                </TableCell>
              </TableRow>

              {isExpanded && (
                <TableRow key={`${name}-expanded`}>
                  <TableCell colSpan={7} className="bg-muted/30 p-0">
                    <div className="space-y-4 p-4">
                      {!result && (
                        <p className="py-4 text-center text-sm text-muted-foreground">
                          Click <strong>Solve</strong> to find the optimal
                          formulation.
                        </p>
                      )}

                      {result && result.status === "error" && (
                        <div className="flex items-center gap-2 rounded-lg border border-red-500/20 bg-red-500/10 p-3 text-red-600 dark:text-red-400">
                          <AlertCircle className="h-5 w-5" />
                          <span className="font-medium">
                            Error solving formula
                          </span>
                        </div>
                      )}

                      {result && result.status !== "error" && (
                        <>
                          {result.description && (
                            <p className="text-sm text-muted-foreground">
                              {result.description}
                            </p>
                          )}

                          {result.violations &&
                            result.violations.length > 0 && (
                              <ConstraintViolations
                                violations={result.violations}
                              />
                            )}

                          {result.ingredients.length > 0 && (
                            <SolveResultsTable
                              ingredients={result.ingredients}
                              nutrients={result.nutrients}
                            />
                          )}

                          {result.analysis && (
                            <SensitivityAnalysis
                              bindingConstraints={
                                result.analysis.bindingConstraints
                              }
                              shadowPrices={result.analysis.shadowPrices}
                            />
                          )}
                        </>
                      )}
                    </div>
                  </TableCell>
                </TableRow>
              )}
            </>
          );
        })}
      </TableBody>
    </Table>
  );
}
