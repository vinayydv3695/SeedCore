import { useState, useEffect } from "react";
import { save, open } from "@tauri-apps/plugin-dialog";
import { Settings } from "../types";
import { api } from "../lib/api";
import { formatBytes } from "../lib/utils";
import { DebridSettings } from "./DebridSettings";

type Tab = "general" | "debrid" | "scheduler";

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
    if (isOpen) {
      loadSettings();
    }
  }, [isOpen]);

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

  const handleExport = async () => {
    try {
      const path = await save({
        filters: [{
          name: 'SeedCore Backup',
          extensions: ['json']
        }],
        defaultPath: 'seedcore-backup.json'
      });

      if (path) {
        setIsSaving(true);
        await api.exportBackup(path);
        setSuccessMessage("Backup exported successfully!");
        setTimeout(() => setSuccessMessage(null), 3000);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to export backup");
    } finally {
      setIsSaving(false);
    }
  };

  const handleImport = async () => {
    try {
      const path = await open({
        filters: [{
          name: 'SeedCore Backup',
          extensions: ['json']
        }],
        multiple: false
      });

      if (path) {
        setIsSaving(true);
        // path is string | string[] | null in dialog v2, but since multiple is false, it's string | null
        const filePath = Array.isArray(path) ? path[0] : path;
        if (filePath) {
          await api.importBackup(filePath);
          setSuccessMessage("Backup imported successfully! Reloading settings...");
          await loadSettings();
          setTimeout(() => setSuccessMessage(null), 3000);
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to import backup");
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

  const addRule = () => {
    if (!settings) return;
    const newRule = {
      start_time: "00:00",
      end_time: "08:00",
      days: [0, 1, 2, 3, 4, 5, 6],
      download_limit: 0,
      upload_limit: 0,
      enabled: true,
    };
    setSettings({
      ...settings,
      bandwidth_schedule: [...settings.bandwidth_schedule, newRule],
    });
  };

  const removeRule = (index: number) => {
    if (!settings) return;
    const newSchedule = [...settings.bandwidth_schedule];
    newSchedule.splice(index, 1);
    setSettings({ ...settings, bandwidth_schedule: newSchedule });
  };

  const updateRule = (index: number, updates: any) => {
    if (!settings) return;
    const newSchedule = [...settings.bandwidth_schedule];
    newSchedule[index] = { ...newSchedule[index], ...updates };
    setSettings({ ...settings, bandwidth_schedule: newSchedule });
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
            <TabButton
              active={activeTab === "scheduler"}
              onClick={() => setActiveTab("scheduler")}
              label="Scheduler"
              icon={<CalendarIcon />}
            />
          </div>
        </div>

        {/* Tab Content */}
        <div className="p-6">
          {isLoading ? (
            <div className="flex items-center justify-center py-12">
              <div className="h-8 w-8 animate-spin rounded-full border-4 border-gray-600 border-t-primary" />
            </div>
          ) : settings ? (
            activeTab === "general" ? (
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

                {/* Auto-Cleanup */}
                <Section title="Auto-Cleanup">
                  <div className="space-y-4">
                    <Checkbox
                      label="Enable Auto-Cleanup"
                      checked={settings.cleanup_enabled}
                      onChange={(checked) =>
                        setSettings({ ...settings, cleanup_enabled: checked })
                      }
                      description="Automatically manage finished torrents"
                    />

                    {settings.cleanup_enabled && (
                      <div className="grid gap-4 sm:grid-cols-2">
                        <NumberInput
                          label="Ratio Limit"
                          value={settings.cleanup_ratio}
                          onChange={(val) =>
                            setSettings({ ...settings, cleanup_ratio: val })
                          }
                          min={0}
                          max={100}
                          step={0.1}
                        />
                        <NumberInput
                          label="Time Limit (Hours)"
                          value={Math.round(settings.cleanup_time / 3600)}
                          onChange={(val) =>
                            setSettings({ ...settings, cleanup_time: val * 3600 })
                          }
                          min={0}
                          max={1000}
                        />
                        <div>
                          <label className="mb-1.5 block text-sm font-medium text-gray-300">
                            Action
                          </label>
                          <select
                            value={settings.cleanup_mode}
                            onChange={(e) =>
                              setSettings({ ...settings, cleanup_mode: e.target.value })
                            }
                            className="w-full rounded-lg border border-dark-border bg-dark-surface-elevated px-4 py-2 text-sm text-white focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
                          >
                            <option value="Pause">Pause</option>
                            <option value="Remove">Remove from list</option>
                            <option value="Delete">Delete files</option>
                          </select>
                        </div>
                      </div>
                    )}
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

                {/* Backup & Restore */}
                <Section title="Backup & Restore">
                  <div className="flex flex-wrap gap-4">
                    <button
                      onClick={handleExport}
                      disabled={isSaving}
                      className="flex items-center gap-2 rounded-lg bg-dark-surface-elevated border border-dark-border px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-dark-surface-elevated/80 disabled:opacity-50"
                    >
                      <DownloadIcon />
                      <span>Export Backup</span>
                    </button>
                    <button
                      onClick={handleImport}
                      disabled={isSaving}
                      className="flex items-center gap-2 rounded-lg bg-dark-surface-elevated border border-dark-border px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-dark-surface-elevated/80 disabled:opacity-50"
                    >
                      <UploadIcon />
                      <span>Import Backup</span>
                    </button>
                  </div>
                  <p className="mt-2 text-xs text-gray-500">
                    Export your settings and torrent list to a JSON file, or restore them from a previous backup.
                  </p>
                </Section>
              </div>
            ) : activeTab === "scheduler" ? (
              <div className="space-y-6">
                <Section title="Bandwidth Scheduler">
                  <div className="space-y-4">
                    <Checkbox
                      label="Enable Scheduler"
                      checked={settings.bandwidth_scheduler_enabled}
                      onChange={(checked) =>
                        setSettings({ ...settings, bandwidth_scheduler_enabled: checked })
                      }
                      description="Automatically change speed limits based on a schedule"
                    />

                    {settings.bandwidth_scheduler_enabled && (
                      <div className="space-y-4">
                        {settings.bandwidth_schedule.map((rule, index) => (
                          <div
                            key={index}
                            className="rounded-lg border border-dark-border bg-dark-surface-elevated p-4 space-y-4"
                          >
                            <div className="flex items-center justify-between">
                              <h4 className="text-sm font-medium text-white">Rule #{index + 1}</h4>
                              <button
                                onClick={() => removeRule(index)}
                                className="text-error hover:text-error/80 transition-colors"
                              >
                                <TrashIcon />
                              </button>
                            </div>

                            <div className="grid gap-4 sm:grid-cols-2">
                              <div>
                                <label className="mb-1.5 block text-sm font-medium text-gray-300">Start Time</label>
                                <input
                                  type="time"
                                  value={rule.start_time}
                                  onChange={(e) => updateRule(index, { start_time: e.target.value })}
                                  className="w-full rounded-lg border border-dark-border bg-dark-surface px-4 py-2 text-sm text-white focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
                                />
                              </div>
                              <div>
                                <label className="mb-1.5 block text-sm font-medium text-gray-300">End Time</label>
                                <input
                                  type="time"
                                  value={rule.end_time}
                                  onChange={(e) => updateRule(index, { end_time: e.target.value })}
                                  className="w-full rounded-lg border border-dark-border bg-dark-surface px-4 py-2 text-sm text-white focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
                                />
                              </div>
                            </div>

                            <div>
                              <label className="mb-1.5 block text-sm font-medium text-gray-300">Days of Week</label>
                              <div className="flex flex-wrap gap-2">
                                {["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"].map((day, dIdx) => (
                                  <button
                                    key={day}
                                    onClick={() => {
                                      const newDays = rule.days.includes(dIdx)
                                        ? rule.days.filter((d) => d !== dIdx)
                                        : [...rule.days, dIdx];
                                      updateRule(index, { days: newDays });
                                    }}
                                    className={`px-3 py-1 rounded-md text-xs font-medium transition-colors ${rule.days.includes(dIdx)
                                      ? "bg-primary text-white"
                                      : "bg-dark-surface text-gray-400 hover:text-white"
                                      }`}
                                  >
                                    {day}
                                  </button>
                                ))}
                              </div>
                            </div>

                            <div className="grid gap-4 sm:grid-cols-2">
                              <SpeedInput
                                label="Download Limit"
                                value={rule.download_limit}
                                onChange={(val) => updateRule(index, { download_limit: val })}
                                placeholder="Unlimited"
                              />
                              <SpeedInput
                                label="Upload Limit"
                                value={rule.upload_limit}
                                onChange={(val) => updateRule(index, { upload_limit: val })}
                                placeholder="Unlimited"
                              />
                            </div>

                            <Checkbox
                              label="Enabled"
                              checked={rule.enabled}
                              onChange={(checked) => updateRule(index, { enabled: checked })}
                            />
                          </div>
                        ))}

                        <button
                          onClick={addRule}
                          className="flex w-full items-center justify-center gap-2 rounded-lg border-2 border-dashed border-dark-border p-3 text-sm font-medium text-gray-400 transition-colors hover:border-primary hover:text-primary"
                        >
                          <PlusIcon />
                          <span>Add New Rule</span>
                        </button>
                      </div>
                    )}
                  </div>
                </Section>
              </div>
            ) : (
              <DebridSettings />
            )
          ) : (
            <div className="py-12 text-center text-gray-400">
              Failed to load settings
            </div>
          )}
        </div>

        {/* Footer - For all tabs except Debrid (which manages its own state) */}
        {activeTab !== "debrid" && (
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
  step?: number;
}

function NumberInput({ label, value, onChange, min, max, step = 1 }: NumberInputProps) {
  return (
    <div>
      <label className="mb-1.5 block text-sm font-medium text-gray-300">
        {label}
      </label>
      <input
        type="number"
        value={value}
        onChange={(e) => {
          const val = parseFloat(e.target.value);
          if (!isNaN(val) && val >= min && val <= max) {
            onChange(val);
          }
        }}
        min={min}
        max={max}
        step={step}
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
      className={`flex items-center gap-2 border-b-2 px-4 py-3 text-sm font-medium transition-colors ${active
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

function DownloadIcon() {
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
        d="M4 16v1a2 2 0 002 2h12a2 2 0 002-2v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
      />
    </svg>
  );
}

function UploadIcon() {
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
        d="M4 16v1a2 2 0 002 2h12a2 2 0 002-2v-1m-4-8l-4-4m0 0L8 8m4-4v12"
      />
    </svg>
  );
}

function CalendarIcon() {
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
        d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
      />
    </svg>
  );
}

function TrashIcon() {
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
        d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-4v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
      />
    </svg>
  );
}

function PlusIcon() {
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
        d="M12 4v16m8-8H4"
      />
    </svg>
  );
}
