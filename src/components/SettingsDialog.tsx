import { useState, useEffect } from "react";
import { Settings } from "../types";
import { api } from "../lib/api";
import { formatBytes } from "../lib/utils";
import { DebridSettings } from "./DebridSettings";

type Tab = "general" | "debrid";

interface SettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export function SettingsDialog({ isOpen, onClose }: SettingsDialogProps) {
  const [activeTab, setActiveTab] = useState<Tab>("general");
  const [settings, setSettings] = useState<Settings | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  // Load settings when dialog opens
  useEffect(() => {
    if (isOpen && activeTab === "general") {
      loadSettings();
    }
  }, [isOpen, activeTab]);

  const loadSettings = async () => {
    try {
      setIsLoading(true);
      setError(null);
      const currentSettings = await api.getSettings();
      setSettings(currentSettings);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load settings");
    } finally {
      setIsLoading(false);
    }
  };

  const handleSave = async () => {
    if (!settings) return;

    try {
      setIsSaving(true);
      setError(null);
      await api.updateSettings(settings);
      setSuccessMessage("Settings saved successfully!");
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save settings");
    } finally {
      setIsSaving(false);
    }
  };

  const handleClose = () => {
    if (!isSaving) {
      setError(null);
      setSuccessMessage(null);
      onClose();
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-2xl max-h-[90vh] overflow-y-auto scrollbar-dark rounded-xl border border-dark-border bg-dark-surface shadow-2xl">
        {/* Header */}
        <div className="sticky top-0 z-10 flex items-center justify-between border-b border-dark-border bg-dark-surface px-6 py-4">
          <div className="flex items-center gap-3">
            <SettingsIcon />
            <h2 className="text-xl font-bold text-white">Settings</h2>
          </div>
          <button
            onClick={handleClose}
            disabled={isSaving}
            className="rounded-lg p-1 text-gray-400 transition-colors hover:bg-dark-surface-elevated hover:text-white disabled:opacity-50"
          >
            <CloseIcon />
          </button>
        </div>

        {/* Tabs */}
        <div className="border-b border-dark-border px-6">
          <div className="flex gap-1">
            <TabButton
              active={activeTab === "general"}
              onClick={() => setActiveTab("general")}
              label="General"
            />
            <TabButton
              active={activeTab === "debrid"}
              onClick={() => setActiveTab("debrid")}
              label="Debrid Services"
              icon={<CloudIcon />}
            />
          </div>
        </div>

        {/* Tab Content */}
        <div className="p-6">
          {activeTab === "general" ? (
            isLoading ? (
              <div className="flex items-center justify-center py-12">
                <div className="h-8 w-8 animate-spin rounded-full border-4 border-gray-600 border-t-primary" />
              </div>
            ) : settings ? (
              <div className="space-y-6">
                {/* Success message */}
                {successMessage && (
                  <div className="rounded-lg bg-success/10 border border-success/20 p-3 text-sm text-success">
                    {successMessage}
                  </div>
                )}

                {/* Error message */}
                {error && (
                  <div className="rounded-lg bg-error/10 border border-error/20 p-3 text-sm text-error">
                    {error}
                  </div>
                )}

                {/* Bandwidth Limits */}
                <Section title="Bandwidth Limits">
                  <div className="grid gap-4 sm:grid-cols-2">
                    <SpeedInput
                      label="Download Limit"
                      value={settings.download_limit}
                      onChange={(val) =>
                        setSettings({ ...settings, download_limit: val })
                      }
                      placeholder="Unlimited"
                    />
                    <SpeedInput
                      label="Upload Limit"
                      value={settings.upload_limit}
                      onChange={(val) =>
                        setSettings({ ...settings, upload_limit: val })
                      }
                      placeholder="Unlimited"
                    />
                  </div>
                  <p className="mt-2 text-xs text-gray-500">
                    Set to 0 for unlimited. Values are in bytes per second.
                  </p>
                </Section>

                {/* Active Torrents */}
                <Section title="Active Torrents">
                  <div className="grid gap-4 sm:grid-cols-2">
                    <NumberInput
                      label="Max Active Downloads"
                      value={settings.max_active_downloads}
                      onChange={(val) =>
                        setSettings({ ...settings, max_active_downloads: val })
                      }
                      min={1}
                      max={10}
                    />
                    <NumberInput
                      label="Max Active Uploads"
                      value={settings.max_active_uploads}
                      onChange={(val) =>
                        setSettings({ ...settings, max_active_uploads: val })
                      }
                      min={1}
                      max={10}
                    />
                  </div>
                </Section>

                {/* Network */}
                <Section title="Network">
                  <div className="space-y-4">
                    <NumberInput
                      label="Listen Port"
                      value={settings.listen_port}
                      onChange={(val) =>
                        setSettings({ ...settings, listen_port: val })
                      }
                      min={1024}
                      max={65535}
                    />
                    <div className="space-y-2">
                      <Checkbox
                        label="Enable DHT (Distributed Hash Table)"
                        checked={settings.enable_dht}
                        onChange={(checked) =>
                          setSettings({ ...settings, enable_dht: checked })
                        }
                        description="Find peers without trackers (recommended)"
                      />
                      <Checkbox
                        label="Enable PEX (Peer Exchange)"
                        checked={settings.enable_pex}
                        onChange={(checked) =>
                          setSettings({ ...settings, enable_pex: checked })
                        }
                        description="Share peer information with other peers"
                      />
                    </div>
                  </div>
                </Section>

                {/* Appearance */}
                <Section title="Appearance">
                  <Checkbox
                    label="Dark Mode"
                    checked={settings.dark_mode}
                    onChange={(checked) =>
                      setSettings({ ...settings, dark_mode: checked })
                    }
                    description="Use dark theme (light theme coming soon)"
                  />
                </Section>
              </div>
            ) : (
              <div className="py-12 text-center text-gray-400">
                Failed to load settings
              </div>
            )
          ) : (
            <DebridSettings />
          )}
        </div>

        {/* Footer - Only for General tab */}
        {activeTab === "general" && (
          <div className="sticky bottom-0 flex items-center justify-end gap-3 border-t border-dark-border bg-dark-surface px-6 py-4">
            <button
              onClick={handleClose}
              disabled={isSaving}
              className="rounded-lg px-4 py-2 text-sm font-medium text-gray-400 transition-colors hover:bg-dark-surface-elevated hover:text-white disabled:opacity-50"
            >
              Cancel
            </button>
            <button
              onClick={handleSave}
              disabled={isSaving || !settings}
              className="flex items-center gap-2 rounded-lg bg-primary px-6 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-hover disabled:opacity-50"
            >
              {isSaving && (
                <div className="h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent" />
              )}
              <span>{isSaving ? "Saving..." : "Save Changes"}</span>
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

// Helper Components

interface SectionProps {
  title: string;
  children: React.ReactNode;
}

function Section({ title, children }: SectionProps) {
  return (
    <div>
      <h3 className="mb-3 text-sm font-semibold text-white">{title}</h3>
      {children}
    </div>
  );
}

interface SpeedInputProps {
  label: string;
  value: number;
  onChange: (value: number) => void;
  placeholder?: string;
}

function SpeedInput({ label, value, onChange, placeholder }: SpeedInputProps) {
  const [displayValue, setDisplayValue] = useState(
    value === 0 ? "" : String(value),
  );

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = e.target.value;
    setDisplayValue(val);

    const numVal = parseInt(val, 10);
    if (!isNaN(numVal) && numVal >= 0) {
      onChange(numVal);
    } else if (val === "") {
      onChange(0);
    }
  };

  return (
    <div>
      <label className="mb-1.5 block text-sm font-medium text-gray-300">
        {label}
      </label>
      <div className="relative">
        <input
          type="text"
          value={displayValue}
          onChange={handleChange}
          placeholder={placeholder}
          className="w-full rounded-lg border border-dark-border bg-dark-surface-elevated px-4 py-2 pr-16 text-sm text-white placeholder-gray-500 focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
        />
        {value > 0 && (
          <span className="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-gray-500">
            {formatBytes(value)}/s
          </span>
        )}
      </div>
    </div>
  );
}

interface NumberInputProps {
  label: string;
  value: number;
  onChange: (value: number) => void;
  min: number;
  max: number;
}

function NumberInput({ label, value, onChange, min, max }: NumberInputProps) {
  return (
    <div>
      <label className="mb-1.5 block text-sm font-medium text-gray-300">
        {label}
      </label>
      <input
        type="number"
        value={value}
        onChange={(e) => {
          const val = parseInt(e.target.value, 10);
          if (!isNaN(val) && val >= min && val <= max) {
            onChange(val);
          }
        }}
        min={min}
        max={max}
        className="w-full rounded-lg border border-dark-border bg-dark-surface-elevated px-4 py-2 text-sm text-white focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
      />
    </div>
  );
}

interface CheckboxProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  description?: string;
}

function Checkbox({ label, checked, onChange, description }: CheckboxProps) {
  return (
    <label className="flex cursor-pointer items-start gap-3 rounded-lg p-3 transition-colors hover:bg-dark-surface-elevated">
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        className="mt-0.5 h-4 w-4 rounded border-dark-border bg-dark-surface-elevated text-primary focus:ring-2 focus:ring-primary/20"
      />
      <div className="flex-1">
        <div className="text-sm font-medium text-white">{label}</div>
        {description && (
          <div className="mt-0.5 text-xs text-gray-500">{description}</div>
        )}
      </div>
    </label>
  );
}

// Icons

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

function SettingsIcon() {
  return (
    <svg
      className="h-6 w-6 text-gray-400"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
      />
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
      />
    </svg>
  );
}

interface TabButtonProps {
  active: boolean;
  onClick: () => void;
  label: string;
  icon?: React.ReactNode;
}

function TabButton({ active, onClick, label, icon }: TabButtonProps) {
  return (
    <button
      onClick={onClick}
      className={`flex items-center gap-2 border-b-2 px-4 py-3 text-sm font-medium transition-colors ${
        active
          ? "border-primary text-primary"
          : "border-transparent text-gray-400 hover:text-gray-300"
      }`}
    >
      {icon}
      <span>{label}</span>
    </button>
  );
}

function CloudIcon() {
  return (
    <svg
      className="h-4 w-4"
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
