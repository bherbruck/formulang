import { AlertTriangle } from "lucide-react";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { Violation } from "./types";

interface ViolationsSectionProps {
  violations: Violation[];
  colSpan: number;
  className?: string;
}

export function ViolationsSection({ violations, colSpan, className = "" }: ViolationsSectionProps) {
  if (violations.length === 0) return null;

  return (
    <>
      {/* Section header */}
      <TableRow className={`bg-amber-500/10 ${className}`}>
        <TableCell colSpan={colSpan} className="font-semibold text-xs uppercase tracking-wide text-amber-600 dark:text-amber-400 py-2 bg-amber-500/10 border-t-2">
          <div className="flex items-center gap-2">
            <AlertTriangle className="h-4 w-4" />
            Constraint Violations
          </div>
        </TableCell>
      </TableRow>

      {/* Nested table */}
      <TableRow className={className}>
        <TableCell colSpan={colSpan} className="p-0">
          <p className="text-xs text-muted-foreground py-2 px-4">
            The following constraints could not be satisfied. The solution shown is a best-effort result.
          </p>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Constraint</TableHead>
                <TableHead className="text-right w-24">Required</TableHead>
                <TableHead className="text-right w-24">Actual</TableHead>
                <TableHead className="text-right w-24">Gap</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {violations.map((v) => (
                <TableRow key={v.constraint}>
                  <TableCell className="font-medium">{v.constraint}</TableCell>
                  <TableCell className="text-right tabular-nums">{v.required.toFixed(2)}</TableCell>
                  <TableCell className="text-right tabular-nums">{v.actual.toFixed(2)}</TableCell>
                  <TableCell className="text-right tabular-nums text-amber-600 dark:text-amber-400">
                    {v.violationAmount.toFixed(2)}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableCell>
      </TableRow>
    </>
  );
}
