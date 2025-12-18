import { useState } from "react";
import { ChevronDown, ChevronRight, ArrowUpDown, ArrowUp, ArrowDown } from "lucide-react";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { ShadowPrice } from "./types";

type SortKey = "constraint" | "value";
type SortDir = "asc" | "desc";

interface SensitivitySectionProps {
  shadowPrices: ShadowPrice[];
  colSpan: number;
  className?: string;
}

export function SensitivitySection({ shadowPrices, colSpan, className = "" }: SensitivitySectionProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [sortKey, setSortKey] = useState<SortKey>("value");
  const [sortDir, setSortDir] = useState<SortDir>("desc");

  // Filter to only show non-zero shadow prices
  const activePrices = shadowPrices.filter((sp) => sp.value > 0);

  if (activePrices.length === 0) return null;

  const toggleSort = (key: SortKey) => {
    if (sortKey === key) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortKey(key);
      setSortDir("desc");
    }
  };

  const sortedPrices = [...activePrices].sort((a, b) => {
    const mult = sortDir === "asc" ? 1 : -1;
    if (sortKey === "constraint") {
      return mult * a.constraint.localeCompare(b.constraint);
    }
    return mult * (a.value - b.value);
  });

  return (
    <>
      {/* Section header - clickable to expand/collapse */}
      <TableRow
        className={`cursor-pointer hover:bg-muted/50 ${className}`}
        onClick={() => setIsOpen(!isOpen)}
      >
        <TableCell colSpan={colSpan} className="font-semibold text-xs uppercase tracking-wide text-muted-foreground py-2 bg-card border-t-2">
          <div className="flex items-center gap-2">
            {isOpen ? (
              <ChevronDown className="h-4 w-4" />
            ) : (
              <ChevronRight className="h-4 w-4" />
            )}
            Sensitivity Analysis
            <span className="font-normal normal-case">
              ({activePrices.length} binding)
            </span>
          </div>
        </TableCell>
      </TableRow>

      {isOpen && (
        <TableRow className={className}>
          <TableCell colSpan={colSpan} className="p-0">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead
                    className="cursor-pointer hover:bg-muted/50 group"
                    onClick={() => toggleSort("constraint")}
                  >
                    <div className="flex items-center gap-1">
                      Constraint
                      {sortKey === "constraint" ? (
                        sortDir === "asc" ? (
                          <ArrowUp className="h-3 w-3" />
                        ) : (
                          <ArrowDown className="h-3 w-3" />
                        )
                      ) : (
                        <ArrowUpDown className="h-3 w-3 text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity" />
                      )}
                    </div>
                  </TableHead>
                  <TableHead
                    className="cursor-pointer text-right hover:bg-muted/50 group w-24"
                    onClick={() => toggleSort("value")}
                  >
                    <div className="flex items-center justify-end gap-1">
                      Value
                      {sortKey === "value" ? (
                        sortDir === "asc" ? (
                          <ArrowUp className="h-3 w-3" />
                        ) : (
                          <ArrowDown className="h-3 w-3" />
                        )
                      ) : (
                        <ArrowUpDown className="h-3 w-3 text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity" />
                      )}
                    </div>
                  </TableHead>
                  <TableHead>Interpretation</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {sortedPrices.map((sp) => (
                  <TableRow key={sp.constraint}>
                    <TableCell className="font-medium">{sp.constraint}</TableCell>
                    <TableCell className="text-right tabular-nums">
                      ${sp.value.toFixed(2)}
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      {sp.interpretation}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableCell>
        </TableRow>
      )}
    </>
  );
}
