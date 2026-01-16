import {
  TableCell,
  TableHead,
  TableRow,
} from "@/components/ui/table";
import {
  useTableConfig,
  COLUMN_LABELS,
  COLUMN_CONFIG_KEY,
  type ColumnId,
} from "@/hooks/use-table-config";
import { SortableHeader } from "./sortable-header";
import type { Ingredient } from "./types";

const INGREDIENT_COLUMNS: ColumnId[] = ["id", "name", "code", "amount", "percentage", "cost"];

type IngSortKey = "id" | "name" | "code" | "amount" | "percentage" | "cost";

interface IngredientsSectionProps {
  ingredients: Ingredient[];
  colSpan: number;
  className?: string;
}

export function IngredientsSection({ ingredients, colSpan, className = "" }: IngredientsSectionProps) {
  const { config, getOrderedColumns, setIngSort } = useTableConfig();

  const ingSortKey = config.ingSortColumn as IngSortKey | null;
  const ingSortDir = config.ingSortDirection;

  const toggleSort = (key: IngSortKey) => {
    if (ingSortKey !== key) {
      setIngSort(key, key === "id" ? "asc" : "desc");
    } else if (ingSortDir === "desc") {
      setIngSort(key, "asc");
    } else if (ingSortDir === "asc") {
      setIngSort(null, null);
    } else {
      setIngSort(key, key === "id" ? "asc" : "desc");
    }
  };

  const sortedIngredients = [...ingredients].sort((a, b) => {
    if (!ingSortKey || !ingSortDir) return 0;
    const mult = ingSortDir === "asc" ? 1 : -1;
    if (ingSortKey === "id") return mult * a.id.localeCompare(b.id);
    if (ingSortKey === "name") return mult * (a.name || "").localeCompare(b.name || "");
    if (ingSortKey === "code") return mult * (a.code || "").localeCompare(b.code || "");
    return mult * (a[ingSortKey] - b[ingSortKey]);
  });

  const visibleCols = getOrderedColumns((col) => {
    if (!INGREDIENT_COLUMNS.includes(col)) return false;
    const key = COLUMN_CONFIG_KEY[col];
    return config[key] as boolean;
  });

  const colsToShow = visibleCols.length > 0 ? visibleCols : ["id" as ColumnId];

  const renderCell = (ing: Ingredient, col: ColumnId) => {
    switch (col) {
      case "id":
        return (
          <TableCell key={col} className="font-medium">
            <span className="font-mono text-xs">{ing.id}</span>
          </TableCell>
        );
      case "name":
        return <TableCell key={col}>{ing.name || "-"}</TableCell>;
      case "code":
        return <TableCell key={col} className="font-mono text-xs">{ing.code || "-"}</TableCell>;
      case "amount":
        return (
          <TableCell key={col} className="text-right tabular-nums">
            {ing.amount.toFixed(2)}
          </TableCell>
        );
      case "percentage":
        return (
          <TableCell key={col} className="text-right tabular-nums">
            {ing.percentage.toFixed(2)}%
          </TableCell>
        );
      case "cost":
        return (
          <TableCell key={col} className="text-right tabular-nums">
            ${ing.cost.toFixed(2)}
          </TableCell>
        );
      default:
        return null;
    }
  };

  const extraCols = colSpan - colsToShow.length;

  return (
    <>
      {/* Section header */}
      <TableRow className={className}>
        <TableHead colSpan={colSpan} className="font-semibold text-xs uppercase tracking-wide text-muted-foreground bg-card border-t-2">
          Ingredients
        </TableHead>
      </TableRow>

      {/* Column headers */}
      <TableRow className={className}>
        <TableHead className="w-8 bg-card" /> {/* Spacer to align with chevron column */}
        {colsToShow.map((col) => {
          const isNumeric = ["amount", "percentage", "cost"].includes(col);
          const cellClassName = isNumeric ? "text-right" : "";
          return (
            <SortableHeader
              key={col}
              onClick={() => toggleSort(col as IngSortKey)}
              className={`${cellClassName} bg-card`}
              isActive={ingSortKey === col}
              direction={ingSortDir}
            >
              {COLUMN_LABELS[col]}
            </SortableHeader>
          );
        })}
        {extraCols > 1 && <TableHead colSpan={extraCols - 1} className="bg-card" />}
      </TableRow>

      {/* Data rows */}
      {sortedIngredients.map((ing) => (
        <TableRow key={ing.id} className={className}>
          <TableCell className="w-8" /> {/* Spacer to align with chevron column */}
          {colsToShow.map((col) => renderCell(ing, col))}
          {extraCols > 1 && <TableCell colSpan={extraCols - 1} />}
        </TableRow>
      ))}
    </>
  );
}
