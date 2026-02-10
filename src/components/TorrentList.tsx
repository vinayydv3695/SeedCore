import { TorrentInfo, TorrentState } from "../types";
import { TorrentItem } from "./TorrentItem";
import { useState } from "react";

interface TorrentListProps {
  torrents: TorrentInfo[];
  onUpdate: () => void;
  onShowDetails: (torrent: TorrentInfo) => void;
}

export function TorrentList({ torrents, onUpdate, onShowDetails }: TorrentListProps) {
  const [filter, setFilter] = useState<"all" | TorrentState | "active">("all");

  const filteredTorrents = torrents.filter((torrent) => {
    if (filter === "all") return true;
    if (filter === "active") {
      return torrent.state === TorrentState.Downloading || torrent.state === TorrentState.Seeding;
    }
    return torrent.state === filter;
  });

  const stats = {
    all: torrents.length,
    active: torrents.filter(
      (t) => t.state === TorrentState.Downloading || t.state === TorrentState.Seeding
    ).length,
    downloading: torrents.filter((t) => t.state === TorrentState.Downloading).length,
    seeding: torrents.filter((t) => t.state === TorrentState.Seeding).length,
    paused: torrents.filter((t) => t.state === TorrentState.Paused).length,
  };

  return (
    <div className="flex h-full flex-col">
      {/* Filter tabs */}
      <div className="mb-4 flex gap-2 overflow-x-auto border-b border-dark-border pb-2">
        <FilterButton
          active={filter === "all"}
          onClick={() => setFilter("all")}
          count={stats.all}
        >
          All
        </FilterButton>
        <FilterButton
          active={filter === "active"}
          onClick={() => setFilter("active")}
          count={stats.active}
        >
          Active
        </FilterButton>
        <FilterButton
          active={filter === TorrentState.Downloading}
          onClick={() => setFilter(TorrentState.Downloading)}
          count={stats.downloading}
        >
          Downloading
        </FilterButton>
        <FilterButton
          active={filter === TorrentState.Seeding}
          onClick={() => setFilter(TorrentState.Seeding)}
          count={stats.seeding}
        >
          Seeding
        </FilterButton>
        <FilterButton
          active={filter === TorrentState.Paused}
          onClick={() => setFilter(TorrentState.Paused)}
          count={stats.paused}
        >
          Paused
        </FilterButton>
      </div>

      {/* Torrent list */}
      <div className="flex-1 space-y-3 overflow-y-auto scrollbar-dark pr-2">
        {filteredTorrents.length === 0 ? (
          <div className="flex h-full flex-col items-center justify-center text-center">
            <div className="mb-4">
              <EmptyIcon />
            </div>
            <h3 className="mb-2 text-lg font-semibold text-white">
              {filter === "all" ? "No torrents yet" : `No ${filter.toLowerCase()} torrents`}
            </h3>
            <p className="text-sm text-gray-400">
              {filter === "all"
                ? "Add a .torrent file or magnet link to get started"
                : `There are no torrents in the ${filter.toLowerCase()} state`}
            </p>
          </div>
        ) : (
          filteredTorrents.map((torrent) => (
            <TorrentItem 
              key={torrent.id} 
              torrent={torrent} 
              onUpdate={onUpdate}
              onShowDetails={onShowDetails}
            />
          ))
        )}
      </div>
    </div>
  );
}

interface FilterButtonProps {
  active: boolean;
  onClick: () => void;
  count: number;
  children: React.ReactNode;
}

function FilterButton({ active, onClick, count, children }: FilterButtonProps) {
  return (
    <button
      onClick={onClick}
      className={`
        flex items-center gap-2 whitespace-nowrap rounded-lg px-4 py-2 text-sm font-medium transition-colors
        ${
          active
            ? "bg-primary text-white"
            : "text-gray-400 hover:bg-dark-surface-elevated hover:text-white"
        }
      `}
    >
      <span>{children}</span>
      <span
        className={`
        rounded-full px-2 py-0.5 text-xs font-semibold
        ${active ? "bg-white/20" : "bg-dark-surface-elevated"}
      `}
      >
        {count}
      </span>
    </button>
  );
}

function EmptyIcon() {
  return (
    <svg className="h-24 w-24 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={1.5}
        d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
      />
    </svg>
  );
}
