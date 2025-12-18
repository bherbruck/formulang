import { Settings2, GripVertical } from "lucide-react";
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core";
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { Checkbox } from "@/components/ui/checkbox";

import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuSeparator,
  DropdownMenuLabel,
} from "@/components/ui/dropdown-menu";
import {
  useTableConfig,
  IDENTIFIER_COLUMNS,
  INGREDIENT_COLUMNS,
  NUTRIENT_COLUMNS,
  COLUMN_LABELS,
  COLUMN_CONFIG_KEY,
  type ColumnId,
  type TableColumnConfig,
} from "@/hooks/use-table-config";

function SortableColumnItem({
  id,
  label,
  checked,
  onToggle,
}: {
  id: string;
  label: string;
  checked: boolean;
  onToggle: () => void;
}) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className="flex items-center gap-2 px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-default"
    >
      <button
        type="button"
        className="cursor-grab touch-none text-muted-foreground hover:text-foreground"
        {...attributes}
        {...listeners}
      >
        <GripVertical className="h-4 w-4" />
      </button>
      <label className="flex items-center gap-2 flex-1 cursor-pointer">
        <Checkbox
          checked={checked}
          onCheckedChange={onToggle}
        />
        <span>{label}</span>
      </label>
    </div>
  );
}

interface ColumnSectionProps {
  title: string;
  columns: ColumnId[];
  orderedColumns: string[];
  config: TableColumnConfig;
  onToggle: (key: keyof TableColumnConfig) => void;
  onReorder: (newOrder: string[]) => void;
}

function ColumnSection({
  title,
  columns,
  orderedColumns,
  config,
  onToggle,
  onReorder,
}: ColumnSectionProps) {
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8,
      },
    }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  // Get columns in this section, respecting order
  const sectionColumns = orderedColumns.filter((col) =>
    columns.includes(col as ColumnId)
  ) as ColumnId[];

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (over && active.id !== over.id) {
      const oldIndex = sectionColumns.indexOf(active.id as ColumnId);
      const newIndex = sectionColumns.indexOf(over.id as ColumnId);
      const newSectionOrder = arrayMove(sectionColumns, oldIndex, newIndex);

      // Rebuild full order maintaining other sections
      const newFullOrder = orderedColumns.map((col) => {
        if (columns.includes(col as ColumnId)) {
          return newSectionOrder.shift()!;
        }
        return col;
      });
      onReorder(newFullOrder);
    }
  };

  return (
    <div className="py-1">
      <div className="px-2 py-1 text-xs font-medium text-muted-foreground">
        {title}
      </div>
      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        onDragEnd={handleDragEnd}
      >
        <SortableContext
          items={sectionColumns}
          strategy={verticalListSortingStrategy}
        >
          {sectionColumns.map((col) => {
            const configKey = COLUMN_CONFIG_KEY[col];
            return (
              <SortableColumnItem
                key={col}
                id={col}
                label={COLUMN_LABELS[col]}
                checked={config[configKey] as boolean}
                onToggle={() => onToggle(configKey)}
              />
            );
          })}
        </SortableContext>
      </DndContext>
    </div>
  );
}

export function ColumnSettings() {
  const { config, toggleColumn, reorderColumns } = useTableConfig();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="sm" className="h-6 w-6 p-0">
          <Settings2 className="h-3.5 w-3.5" />
          <span className="sr-only">Column settings</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-52">
        <DropdownMenuLabel className="text-xs text-muted-foreground">
          Drag to reorder, click to toggle
        </DropdownMenuLabel>
        <DropdownMenuSeparator />

        <ColumnSection
          title="Identifiers"
          columns={IDENTIFIER_COLUMNS}
          orderedColumns={config.columnOrder}
          config={config}
          onToggle={toggleColumn}
          onReorder={reorderColumns}
        />

        <DropdownMenuSeparator />

        <ColumnSection
          title="Ingredients"
          columns={INGREDIENT_COLUMNS}
          orderedColumns={config.columnOrder}
          config={config}
          onToggle={toggleColumn}
          onReorder={reorderColumns}
        />

        <DropdownMenuSeparator />

        <ColumnSection
          title="Nutrients"
          columns={NUTRIENT_COLUMNS}
          orderedColumns={config.columnOrder}
          config={config}
          onToggle={toggleColumn}
          onReorder={reorderColumns}
        />
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
