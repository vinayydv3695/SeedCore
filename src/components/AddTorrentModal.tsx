import { useState, useEffect, useMemo } from "react";
import { formatBytes } from "../lib/utils";

// ============================================================================
// TYPES
// ============================================================================

export interface TorrentFile {
  path: string;
  size: number;
  index: number;
}

export interface TorrentMetadata {
  name: string;
  infoHash: string;
  totalSize: number;
  creationDate?: number;
  files: TorrentFile[];
  comment?: string;
}

export type DownloadMode = "smart" | "cloud" | "p2p" | "hybrid";
export type FilePriority = "high" | "normal" | "low" | "skip";

interface FileSelection {
  [index: number]: {
    selected: boolean;
    priority: FilePriority;
  };
}

interface AddTorrentModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: (config: TorrentConfig) => void;
  metadata: TorrentMetadata;
  defaultSavePath: string;
  availableDiskSpace?: number;
  cacheStatus?: {
    torbox?: boolean;
    realDebrid?: boolean;
  };
}

export interface TorrentConfig {
  savePath: string;
  useIncompletePath: boolean;
  incompletePath?: string;
  category: string;
  tags: string[];
  startImmediately: boolean;
  addToQueue: boolean;
  sequentialDownload: boolean;
  downloadFirstLast: boolean;
  skipHashCheck: boolean;
  downloadMode: DownloadMode;
  debridProvider?: string; // Added for cloud/hybrid modes
  selectedFiles: number[];
  filePriorities: { [index: number]: FilePriority };
}

// ============================================================================
// MAIN COMPONENT
// ============================================================================

export function AddTorrentModal({
  isOpen,
  onClose,
  onConfirm,
  metadata,
  defaultSavePath,
  availableDiskSpace,
  cacheStatus,
}: AddTorrentModalProps) {
  // State
  const [savePath, setSavePath] = useState(defaultSavePath);
  const [useIncompletePath, setUseIncompletePath] = useState(false);
  const [incompletePath, setIncompletePath] = useState("");
  const [category, setCategory] = useState("General");
  const [tags, setTags] = useState<string[]>([]);
  const [tagInput, setTagInput] = useState("");
  const [startImmediately, setStartImmediately] = useState(true);
  const [addToQueue, setAddToQueue] = useState(false);
  const [sequentialDownload, setSequentialDownload] = useState(false);
  const [downloadFirstLast, setDownloadFirstLast] = useState(true);
  const [skipHashCheck, setSkipHashCheck] = useState(false);
  const [downloadMode, setDownloadMode] = useState<DownloadMode>("smart");
  const [debridProvider, setDebridProvider] = useState<string>("real-debrid");
  const [fileSelection, setFileSelection] = useState<FileSelection>({});
  const [searchQuery, setSearchQuery] = useState("");

  // Initialize file selection (all selected by default)
  useEffect(() => {
    if (metadata.files.length > 0) {
      const initial: FileSelection = {};
      metadata.files.forEach((file) => {
        initial[file.index] = { selected: true, priority: "normal" };
      });
      setFileSelection(initial);
    }
  }, [metadata.files]);

  // Auto-select download mode based on cache status
  useEffect(() => {
    if (cacheStatus?.torbox || cacheStatus?.realDebrid) {
      setDownloadMode("smart");
    }
  }, [cacheStatus]);

  // Computed values
  const selectedFiles = useMemo(() => {
    return metadata.files.filter((file) => fileSelection[file.index]?.selected);
  }, [metadata.files, fileSelection]);

  const totalSelectedSize = useMemo(() => {
    return selectedFiles.reduce((sum, file) => sum + file.size, 0);
  }, [selectedFiles]);

  const filteredFiles = useMemo(() => {
    if (!searchQuery) return metadata.files;
    return metadata.files.filter((file) =>
      file.path.toLowerCase().includes(searchQuery.toLowerCase()),
    );
  }, [metadata.files, searchQuery]);

  // Handlers
  const handleSelectAll = () => {
    const updated = { ...fileSelection };
    filteredFiles.forEach((file) => {
      updated[file.index] = { ...updated[file.index], selected: true };
    });
    setFileSelection(updated);
  };

  const handleSelectNone = () => {
    const updated = { ...fileSelection };
    filteredFiles.forEach((file) => {
      updated[file.index] = { ...updated[file.index], selected: false };
    });
    setFileSelection(updated);
  };

  const handleToggleFile = (index: number) => {
    setFileSelection({
      ...fileSelection,
      [index]: {
        ...fileSelection[index],
        selected: !fileSelection[index]?.selected,
      },
    });
  };

  const handleChangePriority = (index: number, priority: FilePriority) => {
    setFileSelection({
      ...fileSelection,
      [index]: {
        ...fileSelection[index],
        priority,
      },
    });
  };

  const handleAddTag = () => {
    if (tagInput.trim() && !tags.includes(tagInput.trim())) {
      setTags([...tags, tagInput.trim()]);
      setTagInput("");
    }
  };

  const handleRemoveTag = (tag: string) => {
    setTags(tags.filter((t) => t !== tag));
  };

  const handleConfirm = () => {
    const config: TorrentConfig = {
      savePath,
      useIncompletePath,
      incompletePath: useIncompletePath ? incompletePath : undefined,
      category,
      tags,
      startImmediately,
      addToQueue,
      sequentialDownload,
      downloadFirstLast,
      skipHashCheck,
      downloadMode,
      debridProvider:
        downloadMode === "cloud" ||
        downloadMode === "smart" ||
        downloadMode === "hybrid"
          ? debridProvider
          : undefined,
      selectedFiles: selectedFiles.map((f) => f.index),
      filePriorities: Object.fromEntries(
        Object.entries(fileSelection)
          .filter(([_, val]) => val.selected)
          .map(([idx, val]) => [idx, val.priority]),
      ),
    };
    onConfirm(config);
  };

  const handleBrowsePath = async () => {
    // Would integrate with Tauri file dialog
    console.log("Browse for folder");
  };

  if (!isOpen) return null;

  const hasSelectedFiles = selectedFiles.length > 0;
  const isCached = cacheStatus?.torbox || cacheStatus?.realDebrid;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm animate-fadeIn">
      <div className="w-[90vw] max-w-6xl h-[85vh] bg-dark-surface rounded-2xl shadow-2xl border border-dark-border overflow-hidden flex flex-col animate-slideUp">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-dark-border bg-dark-surface-elevated/50">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-primary/10 flex items-center justify-center">
              <TorrentIcon />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-white">
                {metadata.name}
              </h2>
              <p className="text-xs text-gray-500">
                {formatBytes(metadata.totalSize)} • {metadata.files.length}{" "}
                files
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="w-8 h-8 rounded-lg flex items-center justify-center text-gray-400 hover:bg-dark-surface-elevated hover:text-white transition-colors"
          >
            <CloseIcon />
          </button>
        </div>

        {/* Main Content - Two Panels */}
        <div className="flex-1 flex overflow-hidden">
          {/* LEFT PANEL - Settings */}
          <div className="w-[40%] border-r border-dark-border overflow-y-auto scrollbar-dark p-6 space-y-6">
            {/* Download Mode Selection */}
            <Section title="Download Mode" icon={<CloudIcon />}>
              <div className="space-y-2">
                <ModeCard
                  mode="smart"
                  title="Smart Mode"
                  description={
                    isCached
                      ? "Cloud available - instant download"
                      : "Auto-select best source"
                  }
                  icon={<SparkleIcon />}
                  selected={downloadMode === "smart"}
                  onClick={() => setDownloadMode("smart")}
                  recommended={isCached}
                />
                <ModeCard
                  mode="cloud"
                  title="Cloud Only"
                  description="Download from debrid service"
                  icon={<CloudIcon />}
                  selected={downloadMode === "cloud"}
                  onClick={() => setDownloadMode("cloud")}
                  disabled={!isCached}
                />
                <ModeCard
                  mode="p2p"
                  title="P2P Only"
                  description="Traditional BitTorrent download"
                  icon={<NetworkIcon />}
                  selected={downloadMode === "p2p"}
                  onClick={() => setDownloadMode("p2p")}
                />
                <ModeCard
                  mode="hybrid"
                  title="Hybrid Mode"
                  description="Combine cloud & P2P sources"
                  icon={<HybridIcon />}
                  selected={downloadMode === "hybrid"}
                  onClick={() => setDownloadMode("hybrid")}
                />
              </div>

              {/* Debrid Provider Selection (show when using cloud/smart/hybrid) */}
              {(downloadMode === "cloud" ||
                downloadMode === "smart" ||
                downloadMode === "hybrid") &&
                isCached && (
                  <div className="mt-3">
                    <Dropdown
                      label="Debrid Provider"
                      value={debridProvider}
                      onChange={setDebridProvider}
                      options={[
                        ...(cacheStatus?.realDebrid ? ["real-debrid"] : []),
                        ...(cacheStatus?.torbox ? ["torbox"] : []),
                      ]}
                    />
                  </div>
                )}
            </Section>

            {/* Save Location */}
            <Section title="Save Location" icon={<FolderIcon />}>
              <div className="space-y-3">
                <PathInput
                  value={savePath}
                  onChange={setSavePath}
                  onBrowse={handleBrowsePath}
                  placeholder="/path/to/downloads"
                />
                <Toggle
                  label="Use different path for incomplete downloads"
                  checked={useIncompletePath}
                  onChange={setUseIncompletePath}
                />
                {useIncompletePath && (
                  <PathInput
                    value={incompletePath}
                    onChange={setIncompletePath}
                    onBrowse={handleBrowsePath}
                    placeholder="/path/to/incomplete"
                  />
                )}
              </div>
            </Section>

            {/* Organization */}
            <Section title="Organization" icon={<TagIcon />}>
              <div className="space-y-3">
                <Dropdown
                  label="Category"
                  value={category}
                  onChange={setCategory}
                  options={[
                    "General",
                    "Movies",
                    "TV Shows",
                    "Music",
                    "Games",
                    "Software",
                  ]}
                />
                <TagInput
                  tags={tags}
                  value={tagInput}
                  onChange={setTagInput}
                  onAdd={handleAddTag}
                  onRemove={handleRemoveTag}
                />
              </div>
            </Section>

            {/* Download Options */}
            <Section title="Download Options" icon={<SettingsIcon />}>
              <div className="space-y-2">
                <Toggle
                  label="Start torrent immediately"
                  checked={startImmediately}
                  onChange={setStartImmediately}
                />
                <Toggle
                  label="Add to queue"
                  checked={addToQueue}
                  onChange={setAddToQueue}
                />
                <Toggle
                  label="Sequential download"
                  checked={sequentialDownload}
                  onChange={setSequentialDownload}
                  description="Download pieces in order"
                />
                <Toggle
                  label="Download first & last pieces first"
                  checked={downloadFirstLast}
                  onChange={setDownloadFirstLast}
                  description="Useful for media previews"
                />
                <Toggle
                  label="Skip hash check"
                  checked={skipHashCheck}
                  onChange={setSkipHashCheck}
                />
              </div>
            </Section>

            {/* Torrent Info */}
            <Section title="Torrent Info" icon={<InfoIcon />}>
              <InfoCard
                items={[
                  {
                    label: "Total Size",
                    value: formatBytes(metadata.totalSize),
                  },
                  { label: "Selected", value: formatBytes(totalSelectedSize) },
                  {
                    label: "Info Hash",
                    value: metadata.infoHash.substring(0, 16) + "...",
                  },
                  {
                    label: "Created",
                    value: metadata.creationDate
                      ? new Date(
                          metadata.creationDate * 1000,
                        ).toLocaleDateString()
                      : "Unknown",
                  },
                  {
                    label: "Free Space",
                    value: availableDiskSpace
                      ? formatBytes(availableDiskSpace)
                      : "Unknown",
                  },
                ]}
              />
            </Section>
          </div>

          {/* RIGHT PANEL - File Selection */}
          <div className="flex-1 flex flex-col overflow-hidden">
            {/* File List Header */}
            <div className="px-6 py-4 border-b border-dark-border bg-dark-surface-elevated/50">
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-2">
                  <button
                    onClick={handleSelectAll}
                    className="px-3 py-1.5 text-xs font-medium text-gray-400 hover:text-white hover:bg-dark-surface-elevated rounded-lg transition-colors"
                  >
                    Select All
                  </button>
                  <button
                    onClick={handleSelectNone}
                    className="px-3 py-1.5 text-xs font-medium text-gray-400 hover:text-white hover:bg-dark-surface-elevated rounded-lg transition-colors"
                  >
                    Select None
                  </button>
                  <span className="text-xs text-gray-500 ml-2">
                    {selectedFiles.length} of {metadata.files.length} selected
                  </span>
                </div>
              </div>
              <SearchInput value={searchQuery} onChange={setSearchQuery} />
            </div>

            {/* File List */}
            <div className="flex-1 overflow-y-auto scrollbar-dark">
              <table className="w-full">
                <thead className="sticky top-0 bg-dark-surface-elevated border-b border-dark-border">
                  <tr>
                    <th className="w-12 px-4 py-3 text-left">
                      <div className="w-4 h-4" />
                    </th>
                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">
                      File Name
                    </th>
                    <th className="px-4 py-3 text-right text-xs font-semibold text-gray-400 uppercase tracking-wider w-32">
                      Size
                    </th>
                    <th className="px-4 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider w-36">
                      Priority
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {filteredFiles.map((file) => {
                    const isSelected = fileSelection[file.index]?.selected;
                    const priority =
                      fileSelection[file.index]?.priority || "normal";

                    return (
                      <tr
                        key={file.index}
                        className={`border-b border-dark-border/50 transition-colors ${
                          isSelected
                            ? "bg-primary/5 hover:bg-primary/10"
                            : "hover:bg-dark-surface-elevated"
                        }`}
                      >
                        <td className="px-4 py-3">
                          <Checkbox
                            checked={isSelected}
                            onChange={() => handleToggleFile(file.index)}
                          />
                        </td>
                        <td className="px-4 py-3">
                          <div className="flex items-center gap-2">
                            <FileIcon />
                            <span className="text-sm text-white truncate">
                              {file.path}
                            </span>
                          </div>
                        </td>
                        <td className="px-4 py-3 text-right text-sm text-gray-400">
                          {formatBytes(file.size)}
                        </td>
                        <td className="px-4 py-3">
                          <PriorityDropdown
                            value={priority}
                            onChange={(p) =>
                              handleChangePriority(file.index, p)
                            }
                            disabled={!isSelected}
                          />
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-6 py-4 border-t border-dark-border bg-dark-surface-elevated/50">
          <div className="text-sm text-gray-400">
            {hasSelectedFiles ? (
              <span>
                <span className="text-white font-medium">
                  {formatBytes(totalSelectedSize)}
                </span>{" "}
                will be downloaded
              </span>
            ) : (
              <span className="text-error">No files selected</span>
            )}
          </div>
          <div className="flex items-center gap-3">
            <button
              onClick={onClose}
              className="px-4 py-2 text-sm font-medium text-gray-400 hover:text-white hover:bg-dark-surface-elevated rounded-lg transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={handleConfirm}
              disabled={!hasSelectedFiles}
              className="px-6 py-2 text-sm font-semibold bg-primary text-white rounded-lg hover:bg-primary-hover disabled:opacity-40 disabled:cursor-not-allowed transition-all shadow-lg shadow-primary/20 hover:shadow-primary/30"
            >
              Add Torrent
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// SUB-COMPONENTS
// ============================================================================

interface SectionProps {
  title: string;
  icon: React.ReactNode;
  children: React.ReactNode;
}

function Section({ title, icon, children }: SectionProps) {
  return (
    <div>
      <div className="flex items-center gap-2 mb-3">
        <div className="text-gray-400">{icon}</div>
        <h3 className="text-sm font-semibold text-white">{title}</h3>
      </div>
      {children}
    </div>
  );
}

interface ModeCardProps {
  mode: DownloadMode;
  title: string;
  description: string;
  icon: React.ReactNode;
  selected: boolean;
  onClick: () => void;
  recommended?: boolean;
  disabled?: boolean;
}

function ModeCard({
  title,
  description,
  icon,
  selected,
  onClick,
  recommended,
  disabled,
}: ModeCardProps) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className={`w-full p-3 rounded-xl border-2 text-left transition-all ${
        disabled
          ? "border-dark-border/30 opacity-40 cursor-not-allowed"
          : selected
            ? "border-primary bg-primary/10 shadow-lg shadow-primary/10"
            : "border-dark-border hover:border-dark-border-hover hover:bg-dark-surface-elevated"
      }`}
    >
      <div className="flex items-start gap-3">
        <div
          className={`mt-0.5 ${selected ? "text-primary" : "text-gray-400"}`}
        >
          {icon}
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-0.5">
            <span
              className={`text-sm font-semibold ${selected ? "text-primary" : "text-white"}`}
            >
              {title}
            </span>
            {recommended && (
              <span className="px-1.5 py-0.5 text-[10px] font-bold bg-success/20 text-success rounded uppercase tracking-wide">
                Recommended
              </span>
            )}
          </div>
          <p className="text-xs text-gray-500">{description}</p>
        </div>
        <div className="mt-1">
          <div
            className={`w-4 h-4 rounded-full border-2 flex items-center justify-center ${
              selected ? "border-primary bg-primary" : "border-gray-600"
            }`}
          >
            {selected && <div className="w-1.5 h-1.5 bg-white rounded-full" />}
          </div>
        </div>
      </div>
    </button>
  );
}

interface PathInputProps {
  value: string;
  onChange: (value: string) => void;
  onBrowse: () => void;
  placeholder: string;
}

function PathInput({ value, onChange, onBrowse, placeholder }: PathInputProps) {
  return (
    <div className="flex gap-2">
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="flex-1 px-3 py-2 bg-dark-surface-elevated border border-dark-border rounded-lg text-sm text-white placeholder-gray-500 focus:border-primary focus:ring-2 focus:ring-primary/20 outline-none transition-all"
      />
      <button
        onClick={onBrowse}
        className="px-3 py-2 bg-dark-surface-elevated border border-dark-border rounded-lg text-gray-400 hover:text-white hover:border-dark-border-hover transition-colors"
      >
        <FolderIcon size={16} />
      </button>
    </div>
  );
}

interface ToggleProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  description?: string;
}

function Toggle({ label, checked, onChange, description }: ToggleProps) {
  return (
    <label className="flex items-start gap-3 cursor-pointer group">
      <button
        type="button"
        onClick={() => onChange(!checked)}
        className={`mt-0.5 w-10 h-5 rounded-full transition-all relative ${
          checked ? "bg-primary" : "bg-gray-700"
        }`}
      >
        <div
          className={`absolute top-0.5 w-4 h-4 bg-white rounded-full transition-transform shadow-md ${
            checked ? "translate-x-5" : "translate-x-0.5"
          }`}
        />
      </button>
      <div className="flex-1">
        <div className="text-sm text-white group-hover:text-gray-200 transition-colors">
          {label}
        </div>
        {description && (
          <div className="text-xs text-gray-500 mt-0.5">{description}</div>
        )}
      </div>
    </label>
  );
}

interface DropdownProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  options: string[];
}

function Dropdown({ label, value, onChange, options }: DropdownProps) {
  return (
    <div>
      <label className="block text-xs font-medium text-gray-400 mb-1.5">
        {label}
      </label>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="w-full px-3 py-2 bg-dark-surface-elevated border border-dark-border rounded-lg text-sm text-white focus:border-primary focus:ring-2 focus:ring-primary/20 outline-none transition-all cursor-pointer [&>option]:bg-black [&>option]:text-white"
      >
        {options.map((option) => (
          <option key={option} value={option} className="bg-black text-white">
            {option}
          </option>
        ))}
      </select>
    </div>
  );
}

interface TagInputProps {
  tags: string[];
  value: string;
  onChange: (value: string) => void;
  onAdd: () => void;
  onRemove: (tag: string) => void;
}

function TagInput({ tags, value, onChange, onAdd, onRemove }: TagInputProps) {
  return (
    <div>
      <label className="block text-xs font-medium text-gray-400 mb-1.5">
        Tags
      </label>
      <div className="flex flex-wrap gap-1.5 mb-2">
        {tags.map((tag) => (
          <span
            key={tag}
            className="px-2 py-1 bg-primary/10 text-primary text-xs rounded-md flex items-center gap-1.5 border border-primary/20"
          >
            {tag}
            <button
              onClick={() => onRemove(tag)}
              className="hover:text-primary-hover"
            >
              <CloseIcon size={12} />
            </button>
          </span>
        ))}
      </div>
      <div className="flex gap-2">
        <input
          type="text"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyPress={(e) => e.key === "Enter" && onAdd()}
          placeholder="Add tag..."
          className="flex-1 px-3 py-2 bg-dark-surface-elevated border border-dark-border rounded-lg text-sm text-white placeholder-gray-500 focus:border-primary focus:ring-2 focus:ring-primary/20 outline-none transition-all"
        />
        <button
          onClick={onAdd}
          className="px-3 py-2 bg-primary/10 text-primary border border-primary/20 rounded-lg text-sm font-medium hover:bg-primary/20 transition-colors"
        >
          Add
        </button>
      </div>
    </div>
  );
}

interface InfoCardProps {
  items: Array<{ label: string; value: string }>;
}

function InfoCard({ items }: InfoCardProps) {
  return (
    <div className="bg-dark-surface-elevated rounded-lg border border-dark-border p-3 space-y-2">
      {items.map((item) => (
        <div
          key={item.label}
          className="flex items-center justify-between text-xs"
        >
          <span className="text-gray-500">{item.label}</span>
          <span className="text-white font-mono">{item.value}</span>
        </div>
      ))}
    </div>
  );
}

interface SearchInputProps {
  value: string;
  onChange: (value: string) => void;
}

function SearchInput({ value, onChange }: SearchInputProps) {
  return (
    <div className="relative">
      <div className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500">
        <SearchIcon />
      </div>
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder="Search files..."
        className="w-full pl-9 pr-4 py-2 bg-dark-surface-elevated border border-dark-border rounded-lg text-sm text-white placeholder-gray-500 focus:border-primary focus:ring-2 focus:ring-primary/20 outline-none transition-all"
      />
    </div>
  );
}

interface CheckboxProps {
  checked: boolean;
  onChange: () => void;
}

function Checkbox({ checked, onChange }: CheckboxProps) {
  return (
    <button
      onClick={onChange}
      className={`w-4 h-4 rounded border-2 flex items-center justify-center transition-all ${
        checked
          ? "bg-primary border-primary"
          : "border-gray-600 hover:border-gray-500"
      }`}
    >
      {checked && <CheckIcon />}
    </button>
  );
}

interface PriorityDropdownProps {
  value: FilePriority;
  onChange: (value: FilePriority) => void;
  disabled?: boolean;
}

function PriorityDropdown({
  value,
  onChange,
  disabled,
}: PriorityDropdownProps) {
  const colors = {
    high: "text-error",
    normal: "text-gray-400",
    low: "text-warning",
    skip: "text-gray-600",
  };

  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value as FilePriority)}
      disabled={disabled}
      className={`px-2 py-1 bg-dark-surface-elevated border border-dark-border rounded-md text-xs font-medium outline-none cursor-pointer transition-colors ${
        colors[value]
      } ${disabled ? "opacity-40 cursor-not-allowed" : "hover:border-dark-border-hover"}`}
    >
      <option value="high">High</option>
      <option value="normal">Normal</option>
      <option value="low">Low</option>
      <option value="skip">Skip</option>
    </select>
  );
}

// ============================================================================
// ICONS
// ============================================================================

function TorrentIcon() {
  return (
    <svg
      className="w-5 h-5 text-primary"
      fill="currentColor"
      viewBox="0 0 24 24"
    >
      <path d="M12 2L2 7v10c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V7l-10-5zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V13H5V9h7V2.99l8 3.99v3.01h-8v4z" />
    </svg>
  );
}

function CloseIcon({ size = 20 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M6 18L18 6M6 6l12 12"
      />
    </svg>
  );
}

function CloudIcon() {
  return (
    <svg
      className="w-4 h-4"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z"
      />
    </svg>
  );
}

function SparkleIcon() {
  return (
    <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
      <path d="M9 11.75A2.25 2.25 0 1111.75 9 2.25 2.25 0 019 11.75zM12.75 9A2.25 2.25 0 1115 11.75 2.25 2.25 0 0112.75 9zM9 14.25A2.25 2.25 0 1111.75 16.5 2.25 2.25 0 019 14.25zM12.75 14.25A2.25 2.25 0 1115 16.5a2.25 2.25 0 01-2.25-2.25z" />
    </svg>
  );
}

function NetworkIcon() {
  return (
    <svg
      className="w-4 h-4"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M8.111 16.404a5.5 5.5 0 017.778 0M12 20h.01m-7.08-7.071c3.904-3.905 10.236-3.905 14.141 0M1.394 9.393c5.857-5.857 15.355-5.857 21.213 0"
      />
    </svg>
  );
}

function HybridIcon() {
  return (
    <svg
      className="w-4 h-4"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4"
      />
    </svg>
  );
}

function FolderIcon({ size = 18 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
      />
    </svg>
  );
}

function TagIcon() {
  return (
    <svg
      className="w-4 h-4"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z"
      />
    </svg>
  );
}

function SettingsIcon() {
  return (
    <svg
      className="w-4 h-4"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4"
      />
    </svg>
  );
}

function InfoIcon() {
  return (
    <svg
      className="w-4 h-4"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
      />
    </svg>
  );
}

function SearchIcon() {
  return (
    <svg
      className="w-4 h-4"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
      />
    </svg>
  );
}

function FileIcon() {
  return (
    <svg
      className="w-4 h-4 text-gray-500"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
      />
    </svg>
  );
}

function CheckIcon() {
  return (
    <svg
      className="w-3 h-3 text-white"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={3}
        d="M5 13l4 4L19 7"
      />
    </svg>
  );
}
