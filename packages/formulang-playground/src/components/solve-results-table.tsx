import { useState } from "react";
import { ArrowUpDown, ArrowUp, ArrowDown } from "lucide-react";

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Card, CardContent } from "@/components/ui/card";

interface Ingredient {
  id: string;
  name?: string;
  code?: string;
  amount: number;
  percentage: number;
  unitCost: number;
  cost: number;
  costPercentage: number;
}

interface Nutrient {
  id: string;
  name?: string;
  code?: string;
  value: number;
  unit?: string;
}

interface SolveResultsTableProps {
  ingredients: Ingredient[];
  nutrients: Nutrient[];
}

type IngSortKey = "id" | "amount" | "percentage" | "cost" | "costPercentage";
type NutSortKey = "id" | "value";
type SortDir = "asc" | "desc";

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
      <div className={`flex items-center gap-1 ${className.includes("text-right") ? "justify-end" : ""}`}>
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

export function SolveResultsTable({
  ingredients,
  nutrients,
}: SolveResultsTableProps) {
  const [ingSortKey, setIngSortKey] = useState<IngSortKey>("id");
  const [ingSortDir, setIngSortDir] = useState<SortDir>("asc");
  const [nutSortKey, setNutSortKey] = useState<NutSortKey>("id");
  const [nutSortDir, setNutSortDir] = useState<SortDir>("asc");

  const toggleIngSort = (key: IngSortKey) => {
    if (ingSortKey === key) {
      setIngSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setIngSortKey(key);
      setIngSortDir(key === "id" ? "asc" : "desc");
    }
  };

  const toggleNutSort = (key: NutSortKey) => {
    if (nutSortKey === key) {
      setNutSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setNutSortKey(key);
      setNutSortDir(key === "id" ? "asc" : "desc");
    }
  };

  const sortedIngredients = [...ingredients].sort((a, b) => {
    const mult = ingSortDir === "asc" ? 1 : -1;
    if (ingSortKey === "id") return mult * a.id.localeCompare(b.id);
    return mult * (a[ingSortKey] - b[ingSortKey]);
  });

  const sortedNutrients = [...nutrients].sort((a, b) => {
    const mult = nutSortDir === "asc" ? 1 : -1;
    if (nutSortKey === "id") return mult * a.id.localeCompare(b.id);
    return mult * (a.value - b.value);
  });

  return (
    <Card>
      <CardContent className="p-0">
        <Table>
          <TableHeader>
            <TableRow className="bg-muted/50">
              <TableHead colSpan={5} className="font-semibold">
                Ingredients
              </TableHead>
            </TableRow>
            <TableRow>
              <SortableHeader onClick={() => toggleIngSort("id")} isActive={ingSortKey === "id"} direction={ingSortDir}>
                ID
              </SortableHeader>
              <SortableHeader
                onClick={() => toggleIngSort("amount")}
                className="text-right"
                isActive={ingSortKey === "amount"}
                direction={ingSortDir}
              >
                Amount (kg)
              </SortableHeader>
              <SortableHeader
                onClick={() => toggleIngSort("percentage")}
                className="text-right"
                isActive={ingSortKey === "percentage"}
                direction={ingSortDir}
              >
                %
              </SortableHeader>
              <SortableHeader
                onClick={() => toggleIngSort("cost")}
                className="text-right"
                isActive={ingSortKey === "cost"}
                direction={ingSortDir}
              >
                Cost
              </SortableHeader>
              <SortableHeader
                onClick={() => toggleIngSort("costPercentage")}
                className="text-right"
                isActive={ingSortKey === "costPercentage"}
                direction={ingSortDir}
              >
                Cost %
              </SortableHeader>
            </TableRow>
          </TableHeader>
          <TableBody>
            {sortedIngredients.map((ing, i) => (
              <TableRow key={i}>
                <TableCell className="font-medium">{ing.name ?? ing.id}</TableCell>
                <TableCell className="text-right tabular-nums">
                  {ing.amount.toFixed(2)}
                </TableCell>
                <TableCell className="text-right tabular-nums">
                  {ing.percentage.toFixed(2)}%
                </TableCell>
                <TableCell className="text-right tabular-nums">
                  ${ing.cost.toFixed(2)}
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex items-center justify-end gap-2">
                    <span className="tabular-nums">{ing.costPercentage.toFixed(1)}%</span>
                    <div className="h-2 w-16 overflow-hidden rounded-full bg-muted">
                      <div
                        className="h-full bg-primary transition-all"
                        style={{ width: `${ing.costPercentage}%` }}
                      />
                    </div>
                  </div>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
          {nutrients.length > 0 && (
            <>
              <TableHeader>
                <TableRow className="bg-muted/50">
                  <TableHead colSpan={5} className="font-semibold">
                    Nutrients
                  </TableHead>
                </TableRow>
                <TableRow>
                  <SortableHeader onClick={() => toggleNutSort("id")} isActive={nutSortKey === "id"} direction={nutSortDir}>
                    ID
                  </SortableHeader>
                  <SortableHeader
                    onClick={() => toggleNutSort("value")}
                    className="text-right"
                    isActive={nutSortKey === "value"}
                    direction={nutSortDir}
                  >
                    Value
                  </SortableHeader>
                  <TableHead className="text-right">Unit</TableHead>
                  <TableHead colSpan={2}></TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {sortedNutrients.map((nut, i) => (
                  <TableRow key={i}>
                    <TableCell className="font-medium">{nut.name ?? nut.id}</TableCell>
                    <TableCell className="text-right tabular-nums">
                      {nut.value.toFixed(2)}
                    </TableCell>
                    <TableCell className="text-right text-muted-foreground">
                      {nut.unit || "%"}
                    </TableCell>
                    <TableCell colSpan={2}></TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </>
          )}
        </Table>
      </CardContent>
    </Card>
  );
}
