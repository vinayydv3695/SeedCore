import { LayoutList, LayoutGrid } from "lucide-react";
import { cn } from "../lib/utils";

interface ViewToggleProps {
  view: "table" | "cards";
  onChange: (view: "table" | "cards") => void;
}

export function ViewToggle({ view, onChange }: ViewToggleProps) {
  return (
    <div className="inline-flex rounded-lg bg-dark-bg border border-dark-border p-0.5 gap-0.5">
      <button
        onClick={() => onChange("table")}
        className={cn(
          "px-2 px-3 py-1.5 rounded-md text-sm font-medium transition-all duration-200 flex items-center gap-2",
          view === "table"
            ? "bg-dark-surface-active text-white shadow-sm"
            : "text-text-tertiary hover:text-text-primary hover:bg-dark-surface-hover"
        )}
        title="Table view"
      >
        <LayoutList className="h-4 w-4" />
        <span className="hidden sm:inline">Table</span>
      </button>

      <button
        onClick={() => onChange("cards")}
        className={cn(
          "px-2 px-3 py-1.5 rounded-md text-sm font-medium transition-all duration-200 flex items-center gap-2",
          view === "cards"
            ? "bg-dark-surface-active text-white shadow-sm"
            : "text-text-tertiary hover:text-text-primary hover:bg-dark-surface-hover"
        )}
        title="Card view"
      >
        <LayoutGrid className="h-4 w-4" />
        <span className="hidden sm:inline">Cards</span>
      </button>
    </div>
  );
}
