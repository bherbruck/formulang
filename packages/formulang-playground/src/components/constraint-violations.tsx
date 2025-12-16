import { AlertTriangle } from "lucide-react";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

interface Violation {
  constraint: string;
  required: number;
  actual: number;
  violationAmount: number;
  description: string;
}

interface ConstraintViolationsProps {
  violations: Violation[];
}

export function ConstraintViolations({ violations }: ConstraintViolationsProps) {
  if (violations.length === 0) return null;

  return (
    <Card className="border-amber-500/50">
      <CardHeader className="pb-2">
        <div className="flex items-center gap-2">
          <AlertTriangle className="h-4 w-4 text-amber-500" />
          <CardTitle className="text-sm text-amber-600 dark:text-amber-400">
            Constraint Violations
          </CardTitle>
        </div>
      </CardHeader>
      <CardContent className="pt-0">
        <p className="mb-3 text-xs text-muted-foreground">
          The following constraints could not be satisfied. The solution shown is
          a best-effort result that minimizes cost while respecting upper limits.
        </p>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Constraint</TableHead>
              <TableHead className="text-right">Required</TableHead>
              <TableHead className="text-right">Actual</TableHead>
              <TableHead className="text-right">Gap</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {violations.map((v, i) => (
              <TableRow key={i}>
                <TableCell className="font-medium">{v.constraint}</TableCell>
                <TableCell className="text-right">{v.required.toFixed(2)}</TableCell>
                <TableCell className="text-right">{v.actual.toFixed(2)}</TableCell>
                <TableCell className="text-right text-amber-600 dark:text-amber-400">
                  {v.violationAmount.toFixed(2)}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
}
