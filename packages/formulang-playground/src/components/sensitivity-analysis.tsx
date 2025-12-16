import { useState } from "react";
import { ChevronDown, ChevronRight, ArrowUpDown, ArrowUp, ArrowDown } from "lucide-react";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

interface ShadowPrice {
  constraint: string;
  value: number;
  interpretation: string;
}

interface SensitivityAnalysisProps {
  bindingConstraints: string[];
  shadowPrices: ShadowPrice[];
}

type SortKey = "constraint" | "value";
type SortDir = "asc" | "desc";

export function SensitivityAnalysis({
  shadowPrices,
}: SensitivityAnalysisProps) {
  const [open, setOpen] = useState(false);
  const [sortKey, setSortKey] = useState<SortKey>("value");
  const [sortDir, setSortDir] = useState<SortDir>("desc");

  // Filter shadow prices to only show those with non-zero values
  const activeShadowPrices = shadowPrices.filter((sp) => sp.value > 0);

  // Don't show if no active shadow prices
  if (activeShadowPrices.length === 0) {
    return null;
  }

  const toggleSort = (key: SortKey) => {
    if (sortKey === key) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortKey(key);
      setSortDir("desc");
    }
  };

  const sortedPrices = [...activeShadowPrices].sort((a, b) => {
    const mult = sortDir === "asc" ? 1 : -1;
    if (sortKey === "constraint") {
      return mult * a.constraint.localeCompare(b.constraint);
    }
    return mult * (a.value - b.value);
  });

  return (
    <Collapsible open={open} onOpenChange={setOpen}>
      <Card>
        <CollapsibleTrigger asChild>
          <CardHeader className="cursor-pointer hover:bg-muted/50 py-3">
            <div className="flex items-center gap-2">
              {open ? (
                <ChevronDown className="h-4 w-4" />
              ) : (
                <ChevronRight className="h-4 w-4" />
              )}
              <CardTitle className="text-sm">Sensitivity Analysis</CardTitle>
            </div>
          </CardHeader>
        </CollapsibleTrigger>
        <CollapsibleContent>
          <CardContent className="pt-0">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead
                    className="cursor-pointer hover:bg-muted/50"
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
                        <ArrowUpDown className="h-3 w-3 text-muted-foreground" />
                      )}
                    </div>
                  </TableHead>
                  <TableHead
                    className="cursor-pointer text-right hover:bg-muted/50"
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
                        <ArrowUpDown className="h-3 w-3 text-muted-foreground" />
                      )}
                    </div>
                  </TableHead>
                  <TableHead>Interpretation</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {sortedPrices.map((sp, i) => (
                  <TableRow key={i}>
                    <TableCell className="font-medium">
                      {sp.constraint}
                    </TableCell>
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
          </CardContent>
        </CollapsibleContent>
      </Card>
    </Collapsible>
  );
}
