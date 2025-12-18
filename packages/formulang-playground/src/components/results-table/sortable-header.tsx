import { ArrowUpDown, ArrowUp, ArrowDown } from "lucide-react";
import { TableHead } from "@/components/ui/table";

interface SortableHeaderProps {
  children: React.ReactNode;
  onClick: () => void;
  className?: string;
  isActive?: boolean;
  direction?: "asc" | "desc" | null;
}

export function SortableHeader({
  children,
  onClick,
  className = "",
  isActive,
  direction,
}: SortableHeaderProps) {
  return (
    <TableHead
      className={`cursor-pointer hover:bg-muted/50 group ${className}`}
      onClick={onClick}
    >
      <div className={`flex items-center gap-1 ${className.includes("text-right") ? "justify-end" : className.includes("text-center") ? "justify-center" : ""}`}>
        {children}
        {isActive ? (
          direction === "asc" ? (
            <ArrowUp className="h-3 w-3" />
          ) : (
            <ArrowDown className="h-3 w-3" />
          )
        ) : (
          <ArrowUpDown className="h-3 w-3 text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity" />
        )}
      </div>
    </TableHead>
  );
}
