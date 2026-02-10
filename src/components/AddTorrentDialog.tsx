import { useState } from "react";
import { api } from "../lib/api";
import { open } from "@tauri-apps/plugin-dialog";

interface AddTorrentDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onAdded: () => void;
}

export function AddTorrentDialog({
  isOpen,
  onClose,
  onAdded,
}: AddTorrentDialogProps) {
  const [magnetLink, setMagnetLink] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!isOpen) return null;

  const handleFileSelect = async () => {
    try {
      setError(null);
      setIsLoading(true);

      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "Torrent",
            extensions: ["torrent"],
          },
        ],
      });

      if (selected && typeof selected === "string") {
        await api.addTorrentFile(selected);
        onAdded();
        onClose();
        setMagnetLink("");
      }
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to add torrent file",
      );
    } finally {
      setIsLoading(false);
    }
  };

  const handleMagnetSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!magnetLink.trim()) return;

    try {
      setError(null);
      setIsLoading(true);
      await api.addMagnetLink(magnetLink.trim());
      onAdded();
      onClose();
      setMagnetLink("");
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to add magnet link",
      );
    } finally {
      setIsLoading(false);
    }
  };

  const handleClose = () => {
    if (!isLoading) {
      setMagnetLink("");
      setError(null);
      onClose();
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-md rounded-xl border border-dark-border bg-dark-surface p-6 shadow-2xl">
        {/* Header */}
        <div className="mb-6 flex items-center justify-between">
          <h2 className="text-xl font-bold text-white">Add Torrent</h2>
          <button
            onClick={handleClose}
            disabled={isLoading}
            className="rounded-lg p-1 text-gray-400 transition-colors hover:bg-dark-surface-elevated hover:text-white disabled:opacity-50"
          >
            <CloseIcon />
          </button>
        </div>

        {/* Error message */}
        {error && (
          <div className="mb-4 rounded-lg bg-error/10 border border-error/20 p-3 text-sm text-error">
            {error}
          </div>
        )}

        {/* File selector */}
        <div className="mb-6">
          <label className="mb-2 block text-sm font-medium text-gray-300">
            Torrent File
          </label>
          <button
            onClick={handleFileSelect}
            disabled={isLoading}
            className="flex w-full items-center justify-center gap-2 rounded-lg border border-dark-border bg-dark-surface-elevated px-4 py-3 text-sm font-medium text-white transition-colors hover:bg-dark-surface disabled:opacity-50"
          >
            <FileIcon />
            <span>{isLoading ? "Adding..." : "Select .torrent file"}</span>
          </button>
        </div>

        {/* Divider */}
        <div className="relative mb-6">
          <div className="absolute inset-0 flex items-center">
            <div className="w-full border-t border-dark-border" />
          </div>
          <div className="relative flex justify-center">
            <span className="bg-dark-surface px-3 text-sm text-gray-500">
              or
            </span>
          </div>
        </div>

        {/* Magnet link input */}
        <form onSubmit={handleMagnetSubmit}>
          <label className="mb-2 block text-sm font-medium text-gray-300">
            Magnet Link
          </label>
          <div className="flex gap-2">
            <input
              type="text"
              value={magnetLink}
              onChange={(e) => setMagnetLink(e.target.value)}
              placeholder="magnet:?xt=urn:btih:..."
              disabled={isLoading}
              className="flex-1 rounded-lg border border-dark-border bg-dark-surface-elevated px-4 py-2.5 text-sm text-white placeholder-gray-500 focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20 disabled:opacity-50"
            />
            <button
              type="submit"
              disabled={isLoading || !magnetLink.trim()}
              className="rounded-lg bg-primary px-6 py-2.5 text-sm font-medium text-white transition-colors hover:bg-primary-hover disabled:opacity-50"
            >
              {isLoading ? "Adding..." : "Add"}
            </button>
          </div>
        </form>

        {/* Info text */}
        <p className="mt-4 text-xs text-gray-500">
          Tip: You can also press Ctrl+V to paste a magnet link
        </p>
      </div>
    </div>
  );
}

function CloseIcon() {
  return (
    <svg
      className="h-6 w-6"
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

function FileIcon() {
  return (
    <svg
      className="h-5 w-5"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"
      />
    </svg>
  );
}
