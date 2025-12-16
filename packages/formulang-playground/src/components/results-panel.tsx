import { FileCode, FlaskConical, Loader2, BarChart3 } from "lucide-react";

import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ScrollArea } from "@/components/ui/scroll-area";
import { FormulasTable } from "./formulas-table";

export interface SolveResult {
  status: "optimal" | "infeasible" | "error";
  formula: string;
  description?: string;
  batchSize: number;
  totalCost: number;
  ingredients: Array<{
    id: string;
    name?: string;
    code?: string;
    amount: number;
    percentage: number;
    unitCost: number;
    cost: number;
    costPercentage: number;
  }>;
  nutrients: Array<{
    id: string;
    name?: string;
    code?: string;
    value: number;
    unit?: string;
  }>;
  analysis?: {
    bindingConstraints: string[];
    shadowPrices: Array<{
      constraint: string;
      value: number;
      interpretation: string;
    }>;
  };
  violations?: Array<{
    constraint: string;
    required: number;
    actual: number;
    violationAmount: number;
    description: string;
  }>;
}

export interface ParseResult {
  nutrients: number;
  ingredients: number;
  formulas: string[];
}

interface ResultsPanelProps {
  parseResult: ParseResult | null;
  solveResults: Record<string, SolveResult>;
  loadingFormula: string | null;
  onSolve: (formulaName: string) => void;
  onSolveAll: () => void;
  wasmReady: boolean;
}

export function ResultsPanel({
  parseResult,
  solveResults,
  loadingFormula,
  onSolve,
  onSolveAll,
  wasmReady,
}: ResultsPanelProps) {
  const formulas = parseResult?.formulas || [];

  return (
    <div className="flex w-1/2 flex-col overflow-hidden">
      <Tabs
        defaultValue="results"
        className="flex h-full flex-col overflow-hidden"
      >
        <div className="flex h-10 items-center justify-between border-b bg-muted/50 px-4">
          <TabsList className="h-7">
            <TabsTrigger value="results" className="h-6 gap-1.5 px-2 text-xs">
              <BarChart3 className="h-3 w-3" />
              Formulas
            </TabsTrigger>
            <TabsTrigger value="ast" className="h-6 gap-1.5 px-2 text-xs">
              <FileCode className="h-3 w-3" />
              AST
            </TabsTrigger>
          </TabsList>

          {!wasmReady && (
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <Loader2 className="h-3 w-3 animate-spin" />
              Loading solver...
            </div>
          )}
        </div>

        <TabsContent
          value="results"
          className="mt-0 min-h-0 flex-1 data-[state=active]:flex data-[state=active]:flex-col"
        >
          <ScrollArea className="h-full">
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
              <FormulasTable
                formulas={formulas}
                solveResults={solveResults}
                loadingFormula={loadingFormula}
                onSolve={onSolve}
                onSolveAll={onSolveAll}
                wasmReady={wasmReady}
              />
            )}
          </ScrollArea>
        </TabsContent>

        <TabsContent value="ast" className="mt-0 flex-1 overflow-auto p-4">
          <pre className="rounded-lg bg-muted p-4 text-xs">
            {JSON.stringify(parseResult, null, 2)}
          </pre>
        </TabsContent>
      </Tabs>
    </div>
  );
}
