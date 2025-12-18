import { useState, useCallback, useEffect, useRef } from "react";
import { FlaskConical, Github, Moon, Sun } from "lucide-react";
import init, { solve as wasmSolve, get_formulas as wasmGetFormulas } from "formulang-lang";

import { Button } from "@/components/ui/button";
import { EditorPanel } from "@/components/editor-panel";
import {
  ResultsPanel,
  type SolveResult,
  type ParseResult,
} from "@/components/results-panel";
import { TableConfigProvider } from "@/hooks/use-table-config";

const STORAGE_KEY = "formulang-playground-code";
const THEME_STORAGE_KEY = "formulang-playground-theme";

const EXAMPLE_CODE = `// Formulang - Least Cost Feed Formulation DSL

nutrient protein {
  name "Crude Protein"
  unit "%"
}

nutrient energy {
  name "Metabolizable Energy"
  unit "kcal/kg"
}

nutrient fiber {
  name "Crude Fiber"
  unit "%"
}

nutrient calcium {
  name "Calcium"
  unit "%"
}

nutrient phosphorus {
  name "Available Phosphorus"
  unit "%"
}

ingredient corn {
  name "Yellow Corn"
  cost 150
  nuts {
    protein 8.5
    energy 3350
    fiber 2.2
    calcium 0.02
    phosphorus 0.08
  }
}

ingredient soybean_meal {
  name "Soybean Meal 48%"
  cost 450
  nuts {
    protein 48.0
    energy 2230
    fiber 3.5
    calcium 0.30
    phosphorus 0.27
  }
}

ingredient wheat_midds {
  name "Wheat Middlings"
  cost 180
  nuts {
    protein 15.5
    energy 2600
    fiber 7.0
    calcium 0.12
    phosphorus 0.35
  }
}

ingredient limestone {
  name "Limestone"
  cost 50
  nuts {
    calcium 38.0
    phosphorus 0.0
  }
}

ingredient dicalcium_phosphate {
  name "Dicalcium Phosphate"
  cost 600
  nuts {
    calcium 22.0
    phosphorus 18.5
  }
}

formula starter {
  name "Starter Feed"
  desc "For chicks 0-3 weeks"
  batch 1000

  nuts {
    protein min 20 max 24
    energy min 2900
    fiber max 5
    calcium min 0.9 max 1.2
    phosphorus min 0.4 max 0.7
  }

  ings {
    corn max 70%
    soybean_meal min 15% max 45%
    wheat_midds max 15%
    limestone max 3%
    dicalcium_phosphate max 3%
  }
}
`;

interface WasmSolveResult {
  status: string;
  formula: string;
  formula_name?: string;
  formula_code?: string;
  description?: string;
  batch_size: number;
  total_cost: number;
  ingredients: Array<{
    id: string;
    name?: string;
    code?: string;
    amount: number;
    percentage: number;
    unit_cost: number;
    cost: number;
    cost_percentage: number;
  }>;
  nutrients: Array<{
    id: string;
    name?: string;
    code?: string;
    value: number;
    unit?: string;
  }>;
  analysis?: {
    binding_constraints: string[];
    shadow_prices: Array<{
      constraint: string;
      value: number;
      interpretation: string;
    }>;
  };
  violations: Array<{
    constraint: string;
    required: number;
    actual: number;
    violation_amount: number;
    description: string;
  }>;
}

function App() {
  const [code, setCode] = useState(() => {
    const saved = localStorage.getItem(STORAGE_KEY);
    return saved ?? EXAMPLE_CODE;
  });
  const [parseResult, setParseResult] = useState<ParseResult | null>(null);
  const [solveResults, setSolveResults] = useState<Record<string, SolveResult>>(
    {}
  );
  const [loadingFormulas, setLoadingFormulas] = useState<Set<string>>(new Set());
  const [isDark, setIsDark] = useState(() => {
    if (typeof window !== "undefined") {
      // Check localStorage first, then system preference
      const stored = localStorage.getItem(THEME_STORAGE_KEY);
      const prefersDark = stored !== null
        ? stored === "dark"
        : window.matchMedia("(prefers-color-scheme: dark)").matches;
      // Set class immediately to avoid flash
      document.documentElement.classList.toggle("dark", prefersDark);
      return prefersDark;
    }
    return true;
  });
  const [wasmReady, setWasmReady] = useState(false);
  const wasmInitialized = useRef(false);

  useEffect(() => {
    if (wasmInitialized.current) return;
    wasmInitialized.current = true;

    init()
      .then(() => setWasmReady(true))
      .catch((err) => console.error("Failed to initialize WASM:", err));
  }, []);

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, code);
  }, [code]);

  useEffect(() => {
    document.documentElement.classList.toggle("dark", isDark);
    localStorage.setItem(THEME_STORAGE_KEY, isDark ? "dark" : "light");
  }, [isDark]);

  useEffect(() => {
    // Only update parseResult for the editor panel badges (nutrients/ingredients/formulas counts)
    // This is fast and doesn't affect solve results
    if (!wasmReady) return;

    let formulas: string[] = [];
    try {
      const formulaInfos = wasmGetFormulas(code) as Array<{ name: string }>;
      formulas = formulaInfos.map((f) => f.name);
    } catch {
      // Fallback to regex if WASM fails (but include template filter)
      const formulaRegex = /(?<!template\s+)formula\s+(\w+)\s*\{/g;
      let match;
      while ((match = formulaRegex.exec(code)) !== null) {
        formulas.push(match[1]);
      }
    }

    const nutrientCount = (code.match(/nutrient\s+\w+\s*\{/g) || []).length;
    const ingredientCount = (code.match(/(?<!template\s+)ingredient\s+\w+\s*\{/g) || []).length;

    setParseResult({
      nutrients: nutrientCount,
      ingredients: ingredientCount,
      formulas,
    });

    // DON'T clear solve results on code change - only clear when formula is deleted
    // Results stay until user re-solves or formula no longer exists
  }, [code, wasmReady]);

  const handleSolve = useCallback(
    async (formulaName: string) => {
      if (!wasmReady) return;

      setLoadingFormulas((prev) => new Set(prev).add(formulaName));

      // Minimum loading duration for visual feedback
      const minLoadingTime = 150;
      const startTime = Date.now();

      try {
        const result = wasmSolve(code, formulaName) as WasmSolveResult;

        // Ensure minimum loading time for visual feedback
        const elapsed = Date.now() - startTime;
        if (elapsed < minLoadingTime) {
          await new Promise((resolve) => setTimeout(resolve, minLoadingTime - elapsed));
        }

        setSolveResults((prev) => ({
          ...prev,
          [formulaName]: {
            status: result.status as "optimal" | "infeasible" | "error",
            formula: result.formula,
            formulaName: result.formula_name,
            formulaCode: result.formula_code,
            description: result.description,
            batchSize: result.batch_size,
            totalCost: result.total_cost,
            ingredients: result.ingredients.map((i) => ({
              id: i.id,
              name: i.name,
              code: i.code,
              amount: i.amount,
              percentage: i.percentage,
              unitCost: i.unit_cost,
              cost: i.cost,
              costPercentage: i.cost_percentage,
            })),
            nutrients: result.nutrients.map((n) => ({
              id: n.id,
              name: n.name,
              code: n.code,
              value: n.value,
              unit: n.unit,
            })),
            analysis: result.analysis
              ? {
                  bindingConstraints: result.analysis.binding_constraints,
                  shadowPrices: result.analysis.shadow_prices,
                }
              : undefined,
            violations: result.violations.map((v) => ({
              constraint: v.constraint,
              required: v.required,
              actual: v.actual,
              violationAmount: v.violation_amount,
              description: v.description,
            })),
          },
        }));
      } catch (err) {
        console.error("Solve error:", err);

        // Ensure minimum loading time even on error
        const elapsed = Date.now() - startTime;
        if (elapsed < minLoadingTime) {
          await new Promise((resolve) => setTimeout(resolve, minLoadingTime - elapsed));
        }

        setSolveResults((prev) => ({
          ...prev,
          [formulaName]: {
            status: "error",
            formula: formulaName,
            batchSize: 0,
            totalCost: 0,
            ingredients: [],
            nutrients: [],
            violations: [],
          },
        }));
      } finally {
        setLoadingFormulas((prev) => {
          const next = new Set(prev);
          next.delete(formulaName);
          return next;
        });
      }
    },
    [code, wasmReady]
  );

  const handleSolveAll = useCallback(() => {
    if (!wasmReady || !parseResult) return;

    for (const formulaName of parseResult.formulas) {
      handleSolve(formulaName);
    }
  }, [wasmReady, parseResult, handleSolve]);

  const handleRefresh = useCallback(() => {
    // Only remove results for formulas that no longer exist
    // Keep results for formulas that still exist in the code
    if (!parseResult) return;

    setSolveResults((prev) => {
      const newResults: Record<string, SolveResult> = {};
      for (const name of parseResult.formulas) {
        if (prev[name]) {
          newResults[name] = prev[name];
        }
      }
      return newResults;
    });
  }, [parseResult]);

  return (
    <TableConfigProvider>
      <div className="flex h-screen flex-col bg-background">
        <header className="flex h-14 items-center justify-between border-b px-4">
          <div className="flex items-center gap-2">
            <FlaskConical className="h-6 w-6 text-primary" />
            <h1 className="text-lg font-semibold">Formulang Playground</h1>
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setIsDark(!isDark)}
            >
              {isDark ? (
                <Sun className="h-4 w-4" />
              ) : (
                <Moon className="h-4 w-4" />
              )}
            </Button>
            <Button variant="ghost" size="icon" asChild>
              <a
                href="https://github.com/bherbruck/formulang"
                target="_blank"
                rel="noopener noreferrer"
              >
                <Github className="h-4 w-4" />
              </a>
            </Button>
          </div>
        </header>

        <div className="flex flex-1 overflow-hidden">
          <EditorPanel
            code={code}
            onCodeChange={setCode}
            parseResult={parseResult}
            isDark={isDark}
            onSolveAll={handleSolveAll}
          />
          <ResultsPanel
            parseResult={parseResult}
            solveResults={solveResults}
            loadingFormulas={loadingFormulas}
            onSolve={handleSolve}
            onSolveAll={handleSolveAll}
            onRefresh={handleRefresh}
            wasmReady={wasmReady}
          />
        </div>
      </div>
    </TableConfigProvider>
  );
}

export default App;
