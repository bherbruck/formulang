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
import type { Nutrient } from "./types";

const NUTRIENT_COLUMNS: ColumnId[] = ["id", "name", "code", "value", "unit"];

type NutSortKey = "id" | "name" | "code" | "value";

interface NutrientsSectionProps {
  nutrients: Nutrient[];
  colSpan: number;
  className?: string;
}

export function NutrientsSection({ nutrients, colSpan, className = "" }: NutrientsSectionProps) {
  const { config, getOrderedColumns, setNutSort } = useTableConfig();

  const nutSortKey = config.nutSortColumn as NutSortKey | null;
  const nutSortDir = config.nutSortDirection;

  const toggleSort = (key: NutSortKey) => {
    if (nutSortKey !== key) {
      setNutSort(key, key === "id" ? "asc" : "desc");
    } else if (nutSortDir === "desc") {
      setNutSort(key, "asc");
    } else if (nutSortDir === "asc") {
      setNutSort(null, null);
    } else {
      setNutSort(key, key === "id" ? "asc" : "desc");
    }
  };

  const sortedNutrients = [...nutrients].sort((a, b) => {
    if (!nutSortKey || !nutSortDir) return 0;
    const mult = nutSortDir === "asc" ? 1 : -1;
    if (nutSortKey === "id") return mult * a.id.localeCompare(b.id);
    if (nutSortKey === "name") return mult * (a.name || "").localeCompare(b.name || "");
    if (nutSortKey === "code") return mult * (a.code || "").localeCompare(b.code || "");
    return mult * (a.value - b.value);
  });

  const visibleCols = getOrderedColumns((col) => {
    if (!NUTRIENT_COLUMNS.includes(col)) return false;
    const key = COLUMN_CONFIG_KEY[col];
    return config[key] as boolean;
  });

  const colsToShow = visibleCols.length > 0 ? visibleCols : ["id" as ColumnId];

  const renderCell = (nut: Nutrient, col: ColumnId) => {
    switch (col) {
      case "id":
        return (
          <TableCell key={col} className="font-medium">
            <span className="font-mono text-xs">{nut.id}</span>
          </TableCell>
        );
      case "name":
        return <TableCell key={col}>{nut.name || "-"}</TableCell>;
      case "code":
        return <TableCell key={col} className="font-mono text-xs">{nut.code || "-"}</TableCell>;
      case "value":
        return (
          <TableCell key={col} className="text-right tabular-nums">
            {nut.value.toFixed(2)}
          </TableCell>
        );
      case "unit":
        return (
          <TableCell key={col} className="text-right text-muted-foreground">
            {nut.unit || "%"}
          </TableCell>
        );
      default:
        return null;
    }
  };

  if (nutrients.length === 0) return null;

  const extraCols = colSpan - colsToShow.length;

  return (
    <>
      {/* Section header */}
      <TableRow className={className}>
        <TableHead colSpan={colSpan} className="font-semibold text-xs uppercase tracking-wide text-muted-foreground bg-card border-t-2">
          Nutrients
        </TableHead>
      </TableRow>

      {/* Column headers */}
      <TableRow className={className}>
        <TableHead className="w-8 bg-card" /> {/* Spacer to align with chevron column */}
        {colsToShow.map((col) => {
          const isNumeric = col === "value";
          const cellClassName = isNumeric || col === "unit" ? "text-right" : "";

          if (col === "unit") {
            return <TableHead key={col} className={`${cellClassName} bg-card`}>{COLUMN_LABELS[col]}</TableHead>;
          }

          return (
            <SortableHeader
              key={col}
              onClick={() => toggleSort(col as NutSortKey)}
              className={`${cellClassName} bg-card`}
              isActive={nutSortKey === col}
              direction={nutSortDir}
            >
              {COLUMN_LABELS[col]}
            </SortableHeader>
          );
        })}
        {extraCols > 1 && <TableHead colSpan={extraCols - 1} className="bg-card" />}
      </TableRow>

      {/* Data rows */}
      {sortedNutrients.map((nut) => (
        <TableRow key={nut.id} className={className}>
          <TableCell className="w-8" /> {/* Spacer to align with chevron column */}
          {colsToShow.map((col) => renderCell(nut, col))}
          {extraCols > 1 && <TableCell colSpan={extraCols - 1} />}
        </TableRow>
      ))}
    </>
  );
}
