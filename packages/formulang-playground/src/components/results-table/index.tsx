import { useState } from "react";
import {
  Table,
  TableBody,
  TableHeader,
} from "@/components/ui/table";
import { FormulasSection, type FormulaSortKey } from "./formulas-section";
import type { SolveResult } from "./types";

type SortDir = "asc" | "desc" | null;

interface ResultsTableProps {
  formulas: string[];
  solveResults: Record<string, SolveResult>;
  loadingFormulas: Set<string>;
  onSolve: (formulaName: string) => void;
  onSolveAll: () => void;
  onRefresh: () => void;
  wasmReady: boolean;
}

export function ResultsTable({
  formulas,
  solveResults,
  loadingFormulas,
  onSolve,
  onSolveAll,
  onRefresh,
  wasmReady,
}: ResultsTableProps) {
  const [expandedFormula, setExpandedFormula] = useState<string | null>(null);
  const [sortKey, setSortKey] = useState<FormulaSortKey | null>("id");
  const [sortDir, setSortDir] = useState<SortDir>("asc");

  const toggleExpanded = (name: string) => {
    setExpandedFormula((prev) => (prev === name ? null : name));
  };

  const toggleSort = (key: FormulaSortKey) => {
    if (sortKey !== key) {
      setSortKey(key);
      setSortDir(key === "name" ? "asc" : "desc");
    } else if (sortDir === "desc") {
      setSortDir("asc");
    } else if (sortDir === "asc") {
      setSortKey(null);
      setSortDir(null);
    } else {
      setSortKey(key);
      setSortDir(key === "name" ? "asc" : "desc");
    }
  };

  return (
    <Table>
      <TableHeader>
        <FormulasSection
          formulas={formulas}
          solveResults={solveResults}
          loadingFormulas={loadingFormulas}
          expandedFormula={expandedFormula}
          onToggleExpand={toggleExpanded}
          onSolve={onSolve}
          onSolveAll={onSolveAll}
          onRefresh={onRefresh}
          wasmReady={wasmReady}
          sortKey={sortKey}
          sortDir={sortDir}
          onToggleSort={toggleSort}
        />
      </TableHeader>
      <TableBody />
    </Table>
  );
}

export type { SolveResult } from "./types";
