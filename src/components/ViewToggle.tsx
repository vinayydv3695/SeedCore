interface ViewToggleProps {
  view: "table" | "cards";
  onChange: (view: "table" | "cards") => void;
}

export function ViewToggle({ view, onChange }: ViewToggleProps) {
  return (
    <div className="inline-flex rounded-lg bg-dark-tertiary p-1 gap-1">
      <button
        onClick={() => onChange("table")}
        className={`
          px-3 py-1.5 rounded-md text-sm font-medium transition-all duration-200
          flex items-center gap-2
          ${
            view === "table"
              ? "bg-primary text-white shadow-sm"
              : "text-gray-400 hover:text-white hover:bg-dark-elevated"
          }
        `}
        title="Table view"
      >
        <svg
          className="w-4 h-4"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M3 10h18M3 14h18m-9-4v8m-7 0h14a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z"
          />
        </svg>
        <span className="hidden sm:inline">Table</span>
      </button>

      <button
        onClick={() => onChange("cards")}
        className={`
          px-3 py-1.5 rounded-md text-sm font-medium transition-all duration-200
          flex items-center gap-2
          ${
            view === "cards"
              ? "bg-primary text-white shadow-sm"
              : "text-gray-400 hover:text-white hover:bg-dark-elevated"
          }
        `}
        title="Card view"
      >
        <svg
          className="w-4 h-4"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"
          />
        </svg>
        <span className="hidden sm:inline">Cards</span>
      </button>
    </div>
  );
}
