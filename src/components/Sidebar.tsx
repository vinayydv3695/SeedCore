import { TorrentInfo, TorrentState } from "../types";
import {
  FolderIcon,
  DownloadIcon,
  UploadIcon,
  ActivityIcon,
  PauseIcon,
  CheckCircleIcon,
  AlertCircleIcon,
  FolderOpenIcon,
  TagIcon,
  PlusIcon,
  HardDriveIcon,
} from "./Icons";

interface SidebarProps {
  torrents: TorrentInfo[];
  selectedFilter: string;
  selectedCategory: string | null;
  selectedTags: string[];
  onFilterChange: (filter: string) => void;
  onCategoryChange: (category: string | null) => void;
  onTagToggle: (tag: string) => void;
  isCollapsed?: boolean;
}

interface FilterItem {
  id: string;
  label: string;
  icon: React.ReactNode;
  count: (torrents: TorrentInfo[]) => number;
}

export function Sidebar({
  torrents,
  selectedFilter,
  selectedCategory,
  selectedTags,
  onFilterChange,
  onCategoryChange,
  onTagToggle,
}: SidebarProps) {
  const filters: FilterItem[] = [
    {
      id: "all",
      label: "All",
      icon: <FolderIcon className="flex-shrink-0" size={18} />,
      count: (t) => t.length,
    },
    {
      id: "downloading",
      label: "Downloading",
      icon: <DownloadIcon className="flex-shrink-0" size={18} />,
      count: (t) =>
        t.filter((x) => x.state === TorrentState.Downloading).length,
    },
    {
      id: "active",
      label: "Active",
      icon: <ActivityIcon className="flex-shrink-0" size={18} />,
      count: (t) =>
        t.filter(
          (x) =>
            x.state === TorrentState.Downloading ||
            x.state === TorrentState.Seeding,
        ).length,
    },
    {
      id: "seeding",
      label: "Seeding",
      icon: <UploadIcon className="flex-shrink-0" size={18} />,
      count: (t) => t.filter((x) => x.state === TorrentState.Seeding).length,
    },
    {
      id: "paused",
      label: "Paused",
      icon: <PauseIcon className="flex-shrink-0" size={18} />,
      count: (t) => t.filter((x) => x.state === TorrentState.Paused).length,
    },
    {
      id: "completed",
      label: "Completed",
      icon: <CheckCircleIcon className="flex-shrink-0" size={18} />,
      count: (t) =>
        t.filter((x) => x.downloaded >= x.size && x.size > 0).length,
    },
    {
      id: "error",
      label: "Error",
      icon: <AlertCircleIcon className="flex-shrink-0" size={18} />,
      count: (t) => t.filter((x) => x.state === TorrentState.Error).length,
    },
  ];

  // Extract unique categories from torrents
  // For now, using mock categories - will be replaced with real data
  const categories = [
    { id: "movies", name: "Movies", count: 0 },
    { id: "tv", name: "TV Shows", count: 0 },
    { id: "games", name: "Games", count: 0 },
    { id: "music", name: "Music", count: 0 },
    { id: "software", name: "Software", count: 0 },
    { id: "books", name: "Books", count: 0 },
    { id: "uncategorized", name: "Uncategorized", count: torrents.length },
  ];

  // Extract unique tags from torrents
  // Mock tags for now
  const tags = [
    { name: "hd", count: 0 },
    { name: "favorite", count: 0 },
    { name: "low-priority", count: 0 },
  ];

  return (
    <div className="w-56 bg-dark-secondary border-r border-dark-border flex flex-col overflow-hidden">
      {/* Filters Section */}
      <div className="p-3 border-b border-dark-border">
        <h3 className="text-xs font-semibold text-gray-400 uppercase tracking-wide mb-2 px-2">
          Filters
        </h3>
        <div className="space-y-0.5">
          {filters.map((filter) => {
            const count = filter.count(torrents);
            const isActive = selectedFilter === filter.id;

            return (
              <button
                key={filter.id}
                onClick={() => onFilterChange(filter.id)}
                className={`
                  w-full flex items-center gap-3 px-3 py-2 rounded-lg
                  transition-all duration-200 text-sm
                  ${
                    isActive
                      ? "bg-primary text-white font-medium shadow-lg shadow-primary/20 scale-[1.02]"
                      : "text-gray-300 hover:bg-dark-elevated hover:text-white hover:scale-[1.01]"
                  }
                `}
              >
                <span className={isActive ? "text-white" : "text-gray-400"}>
                  {filter.icon}
                </span>
                <span className="flex-1 text-left">{filter.label}</span>
                <span
                  className={`
                  text-xs px-2 py-0.5 rounded-full font-medium
                  ${
                    isActive
                      ? "bg-white/20 text-white"
                      : "bg-dark-elevated text-gray-400"
                  }
                `}
                >
                  {count}
                </span>
              </button>
            );
          })}
        </div>
      </div>

      {/* Categories Section */}
      <div className="p-3 border-b border-dark-border">
        <div className="flex items-center justify-between mb-2 px-2">
          <h3 className="text-xs font-semibold text-gray-400 uppercase tracking-wide">
            Categories
          </h3>
          <button
            className="text-xs text-gray-400 hover:text-primary transition-colors p-1 rounded hover:bg-dark-elevated"
            title="Manage categories"
          >
            <PlusIcon size={14} />
          </button>
        </div>
        <div className="space-y-0.5 max-h-48 overflow-y-auto custom-scrollbar">
          {categories.map((category) => {
            const isActive = selectedCategory === category.id;

            return (
              <button
                key={category.id}
                onClick={() => onCategoryChange(isActive ? null : category.id)}
                className={`
                  w-full flex items-center gap-3 px-3 py-2 rounded-lg
                  transition-all duration-200 text-sm
                  ${
                    isActive
                      ? "bg-primary text-white font-medium shadow-lg shadow-primary/20 scale-[1.02]"
                      : "text-gray-300 hover:bg-dark-elevated hover:text-white hover:scale-[1.01]"
                  }
                `}
              >
                <FolderOpenIcon
                  size={18}
                  className={`flex-shrink-0 ${isActive ? "text-white" : "text-gray-400"}`}
                />
                <span className="flex-1 text-left truncate">
                  {category.name}
                </span>
                <span
                  className={`
                  text-xs px-2 py-0.5 rounded-full font-medium
                  ${
                    isActive
                      ? "bg-white/20 text-white"
                      : "bg-dark-elevated text-gray-400"
                  }
                `}
                >
                  {category.count}
                </span>
              </button>
            );
          })}
        </div>
      </div>

      {/* Tags Section */}
      <div className="p-3 flex-1 overflow-hidden">
        <div className="flex items-center justify-between mb-2 px-2">
          <h3 className="text-xs font-semibold text-gray-400 uppercase tracking-wide">
            Tags
          </h3>
          <button
            className="text-xs text-gray-400 hover:text-primary transition-colors p-1 rounded hover:bg-dark-elevated"
            title="Manage tags"
          >
            <PlusIcon size={14} />
          </button>
        </div>
        <div className="space-y-0.5 max-h-32 overflow-y-auto custom-scrollbar">
          {tags.map((tag) => {
            const isActive = selectedTags.includes(tag.name);

            return (
              <button
                key={tag.name}
                onClick={() => onTagToggle(tag.name)}
                className={`
                  w-full flex items-center gap-3 px-3 py-2 rounded-lg
                  transition-all duration-200 text-sm
                  ${
                    isActive
                      ? "bg-primary text-white font-medium shadow-lg shadow-primary/20 scale-[1.02]"
                      : "text-gray-300 hover:bg-dark-elevated hover:text-white hover:scale-[1.01]"
                  }
                `}
              >
                <TagIcon
                  size={18}
                  className={`flex-shrink-0 ${isActive ? "text-white" : "text-gray-400"}`}
                />
                <span className="flex-1 text-left truncate">{tag.name}</span>
                <span
                  className={`
                  text-xs px-2 py-0.5 rounded-full font-medium
                  ${
                    isActive
                      ? "bg-white/20 text-white"
                      : "bg-dark-elevated text-gray-400"
                  }
                `}
                >
                  {tag.count}
                </span>
              </button>
            );
          })}
        </div>
      </div>

      {/* Footer - Storage info */}
      <div className="p-3 border-t border-dark-border">
        <div className="text-xs text-gray-400 space-y-1">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <HardDriveIcon size={14} className="text-gray-500" />
              <span>Free space:</span>
            </div>
            <span className="text-white font-medium">120 GB</span>
          </div>
        </div>
      </div>
    </div>
  );
}
