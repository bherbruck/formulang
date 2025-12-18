import { useState, useEffect, useCallback, createContext, useContext, type ReactNode } from "react";

const STORAGE_KEY = "formulang-table-config";

export type SortDirection = "asc" | "desc" | null;

export interface TableColumnConfig {
  // Identifier columns - which to show for the name/identifier
  showId: boolean;
  showName: boolean;
  showCode: boolean;
  // Ingredient columns
  showAmount: boolean;
  showPercentage: boolean;
  showCost: boolean;
  showCostPercentage: boolean;
  showUnitCost: boolean;
  // Nutrient columns
  showValue: boolean;
  showUnit: boolean;
  // Column order
  columnOrder: string[];
  // Sort state
  ingSortColumn: string | null;
  ingSortDirection: SortDirection;
  nutSortColumn: string | null;
  nutSortDirection: SortDirection;
}

// All available columns in default order
export const ALL_COLUMNS = [
  "id",
  "name",
  "code",
  "amount",
  "percentage",
  "unitCost",
  "cost",
  "costPercentage",
  "value",
  "unit",
] as const;

export type ColumnId = typeof ALL_COLUMNS[number];

// Column groupings
export const IDENTIFIER_COLUMNS: ColumnId[] = ["id", "name", "code"];
export const INGREDIENT_COLUMNS: ColumnId[] = ["amount", "percentage", "unitCost", "cost", "costPercentage"];
export const NUTRIENT_COLUMNS: ColumnId[] = ["value", "unit"];

// Column display names
export const COLUMN_LABELS: Record<ColumnId, string> = {
  id: "ID",
  name: "Name",
  code: "Code",
  amount: "Amount (kg)",
  percentage: "Percentage",
  unitCost: "Unit Cost",
  cost: "Cost",
  costPercentage: "Cost %",
  value: "Value",
  unit: "Unit",
};

// Map column id to config key
export const COLUMN_CONFIG_KEY: Record<ColumnId, keyof TableColumnConfig> = {
  id: "showId",
  name: "showName",
  code: "showCode",
  amount: "showAmount",
  percentage: "showPercentage",
  unitCost: "showUnitCost",
  cost: "showCost",
  costPercentage: "showCostPercentage",
  value: "showValue",
  unit: "showUnit",
};

const defaultConfig: TableColumnConfig = {
  showId: true,
  showName: true,
  showCode: false,
  showAmount: true,
  showPercentage: true,
  showCost: true,
  showCostPercentage: true,
  showUnitCost: false,
  showValue: true,
  showUnit: true,
  columnOrder: [...ALL_COLUMNS],
  ingSortColumn: "id",
  ingSortDirection: "asc",
  nutSortColumn: "id",
  nutSortDirection: "asc",
};

interface TableConfigContextValue {
  config: TableColumnConfig;
  setConfig: React.Dispatch<React.SetStateAction<TableColumnConfig>>;
  toggleColumn: (key: keyof TableColumnConfig) => void;
  resetToDefaults: () => void;
  reorderColumns: (newOrder: string[]) => void;
  getOrderedColumns: (filterFn?: (col: ColumnId) => boolean) => ColumnId[];
  setIngSort: (column: string | null, direction: SortDirection) => void;
  setNutSort: (column: string | null, direction: SortDirection) => void;
}

const TableConfigContext = createContext<TableConfigContextValue | null>(null);

export function TableConfigProvider({ children }: { children: ReactNode }) {
  const [config, setConfig] = useState<TableColumnConfig>(() => {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        const parsed = JSON.parse(saved);
        // Ensure columnOrder has all columns (in case new ones were added)
        const savedOrder = parsed.columnOrder || [];
        const allCols = [...ALL_COLUMNS];
        const order = [
          ...savedOrder.filter((c: string) => allCols.includes(c as ColumnId)),
          ...allCols.filter(c => !savedOrder.includes(c)),
        ];
        return { ...defaultConfig, ...parsed, columnOrder: order };
      }
    } catch {
      // Ignore parse errors
    }
    return defaultConfig;
  });

  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(config));
    } catch {
      // Ignore storage errors
    }
  }, [config]);

  const toggleColumn = useCallback((key: keyof TableColumnConfig) => {
    if (key === "columnOrder") return; // Don't toggle array
    setConfig((prev) => ({ ...prev, [key]: !prev[key] }));
  }, []);

  const resetToDefaults = useCallback(() => {
    setConfig(defaultConfig);
  }, []);

  const reorderColumns = useCallback((newOrder: string[]) => {
    setConfig((prev) => ({ ...prev, columnOrder: newOrder }));
  }, []);

  const getOrderedColumns = useCallback((filterFn?: (col: ColumnId) => boolean): ColumnId[] => {
    const ordered = config.columnOrder.filter((col): col is ColumnId =>
      ALL_COLUMNS.includes(col as ColumnId)
    );
    if (filterFn) {
      return ordered.filter(filterFn);
    }
    return ordered;
  }, [config.columnOrder]);

  const setIngSort = useCallback((column: string | null, direction: SortDirection) => {
    setConfig((prev) => ({ ...prev, ingSortColumn: column, ingSortDirection: direction }));
  }, []);

  const setNutSort = useCallback((column: string | null, direction: SortDirection) => {
    setConfig((prev) => ({ ...prev, nutSortColumn: column, nutSortDirection: direction }));
  }, []);

  return (
    <TableConfigContext.Provider
      value={{ config, setConfig, toggleColumn, resetToDefaults, reorderColumns, getOrderedColumns, setIngSort, setNutSort }}
    >
      {children}
    </TableConfigContext.Provider>
  );
}

export function useTableConfig() {
  const context = useContext(TableConfigContext);
  if (!context) {
    throw new Error("useTableConfig must be used within a TableConfigProvider");
  }
  return context;
}

export type { TableColumnConfig as TableConfig };
