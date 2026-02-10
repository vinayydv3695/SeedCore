import { useState, useEffect } from "react";
import { api } from "../lib/api";
import {
  DebridSettings as DebridSettingsType,
  CredentialStatus,
} from "../types";

export function DebridSettings() {
  const [settings, setSettings] = useState<DebridSettingsType | null>(null);
  const [credentials, setCredentials] = useState<CredentialStatus[]>([]);
  const [isMasterPasswordSet, setIsMasterPasswordSet] = useState(false);
  const [isUnlocked, setIsUnlocked] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Form states
  const [masterPassword, setMasterPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [torboxApiKey, setTorboxApiKey] = useState("");
  const [realDebridApiKey, setRealDebridApiKey] = useState("");

  // Validation states
  const [isValidatingTorbox, setIsValidatingTorbox] = useState(false);
  const [isValidatingRealDebrid, setIsValidatingRealDebrid] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      setIsLoading(true);
      setError(null);

      // Check if master password is set
      const passwordSet = await api.checkMasterPasswordSet();
      setIsMasterPasswordSet(passwordSet);

      // If password is set, we need to unlock to load credentials
      // For now, we'll just load settings
      const debridSettings = await api.getDebridSettings();
      setSettings(debridSettings);

      // Try to load credentials status (will fail if not unlocked)
      try {
        const creds = await api.getDebridCredentialsStatus();
        setCredentials(creds);
        setIsUnlocked(true);
      } catch {
        // Not unlocked yet
        setIsUnlocked(false);
      }
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to load debrid settings",
      );
    } finally {
      setIsLoading(false);
    }
  };

  const handleSetMasterPassword = async () => {
    if (masterPassword !== confirmPassword) {
      setError("Passwords do not match");
      return;
    }

    if (masterPassword.length < 8) {
      setError("Password must be at least 8 characters");
      return;
    }

    try {
      setIsSaving(true);
      setError(null);
      await api.setMasterPassword(masterPassword);
      setIsMasterPasswordSet(true);
      setIsUnlocked(true);
      setSuccess("Master password set successfully!");
      setMasterPassword("");
      setConfirmPassword("");
      setTimeout(() => setSuccess(null), 3000);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to set master password",
      );
    } finally {
      setIsSaving(false);
    }
  };

  const handleUnlock = async () => {
    try {
      setIsSaving(true);
      setError(null);
      const success = await api.unlockWithMasterPassword(masterPassword);
      if (success) {
        setIsUnlocked(true);
        setSuccess("Unlocked successfully!");
        setMasterPassword("");
        setTimeout(() => setSuccess(null), 3000);
        await loadData();
      } else {
        setError("Invalid master password");
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to unlock");
    } finally {
      setIsSaving(false);
    }
  };

  const handleSaveCredentials = async (provider: string, apiKey: string) => {
    if (!isUnlocked) {
      setError("Please unlock with master password first");
      return;
    }

    if (!apiKey.trim()) {
      setError("API key cannot be empty");
      return;
    }

    try {
      if (provider === "torbox") {
        setIsValidatingTorbox(true);
      } else {
        setIsValidatingRealDebrid(true);
      }

      setError(null);
      console.log(`[DebridSettings] Saving credentials for ${provider}...`);
      await api.saveDebridCredentials(provider, apiKey);
      console.log(`[DebridSettings] Credentials saved, now validating...`);

      // Validate the credentials
      const isValid = await api.validateDebridProvider(provider);
      console.log(
        `[DebridSettings] Validation result for ${provider}: ${isValid}`,
      );

      if (isValid) {
        setSuccess(
          `${provider === "torbox" ? "Torbox" : "Real-Debrid"} credentials saved and validated!`,
        );
        setTimeout(() => setSuccess(null), 3000);

        // Clear the API key input
        if (provider === "torbox") {
          setTorboxApiKey("");
        } else {
          setRealDebridApiKey("");
        }

        // Reload credentials status
        await loadData();
      } else {
        console.error(`[DebridSettings] Validation failed for ${provider}`);
        setError(
          `Invalid API key for ${provider === "torbox" ? "Torbox" : "Real-Debrid"}`,
        );
      }
    } catch (err) {
      console.error(
        `[DebridSettings] Error saving credentials for ${provider}:`,
        err,
      );
      setError(
        err instanceof Error ? err.message : "Failed to save credentials",
      );
    } finally {
      if (provider === "torbox") {
        setIsValidatingTorbox(false);
      } else {
        setIsValidatingRealDebrid(false);
      }
    }
  };

  const handleToggleProvider = (provider: string) => {
    if (!settings) return;

    const newPreference = settings.debrid_preference.includes(provider)
      ? settings.debrid_preference.filter((p) => p !== provider)
      : [...settings.debrid_preference, provider];

    setSettings({ ...settings, debrid_preference: newPreference });
  };

  const handleSaveSettings = async () => {
    if (!settings) return;

    try {
      setIsSaving(true);
      setError(null);
      await api.updateDebridSettings(settings);
      setSuccess("Settings saved successfully!");
      setTimeout(() => setSuccess(null), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save settings");
    } finally {
      setIsSaving(false);
    }
  };

  const getCredentialStatus = (provider: string): CredentialStatus | null => {
    return credentials.find((c) => c.provider === provider) || null;
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="h-8 w-8 animate-spin rounded-full border-4 border-gray-600 border-t-primary" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Success/Error Messages */}
      {success && (
        <div className="rounded-lg bg-success/10 border border-success/20 p-3 text-sm text-success">
          {success}
        </div>
      )}

      {error && (
        <div className="rounded-lg bg-error/10 border border-error/20 p-3 text-sm text-error">
          {error}
        </div>
      )}

      {/* Master Password Section */}
      <Section title="Master Password">
        <p className="text-sm text-gray-400 mb-4">
          A master password is required to encrypt and protect your API keys.
        </p>

        {!isMasterPasswordSet ? (
          <div className="space-y-4">
            <div>
              <label className="mb-1.5 block text-sm font-medium text-gray-300">
                Master Password
              </label>
              <input
                type="password"
                value={masterPassword}
                onChange={(e) => setMasterPassword(e.target.value)}
                placeholder="Enter master password (min 8 characters)"
                className="w-full rounded-lg border border-dark-border bg-dark-surface-elevated px-4 py-2 text-sm text-white placeholder-gray-500 focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
              />
            </div>
            <div>
              <label className="mb-1.5 block text-sm font-medium text-gray-300">
                Confirm Password
              </label>
              <input
                type="password"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                placeholder="Confirm master password"
                className="w-full rounded-lg border border-dark-border bg-dark-surface-elevated px-4 py-2 text-sm text-white placeholder-gray-500 focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
              />
            </div>
            <button
              onClick={handleSetMasterPassword}
              disabled={isSaving || !masterPassword || !confirmPassword}
              className="rounded-lg bg-primary px-6 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-hover disabled:opacity-50"
            >
              {isSaving ? "Setting..." : "Set Master Password"}
            </button>
          </div>
        ) : !isUnlocked ? (
          <div className="space-y-4">
            <div>
              <label className="mb-1.5 block text-sm font-medium text-gray-300">
                Enter Master Password to Unlock
              </label>
              <input
                type="password"
                value={masterPassword}
                onChange={(e) => setMasterPassword(e.target.value)}
                placeholder="Enter master password"
                className="w-full rounded-lg border border-dark-border bg-dark-surface-elevated px-4 py-2 text-sm text-white placeholder-gray-500 focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
              />
            </div>
            <button
              onClick={handleUnlock}
              disabled={isSaving || !masterPassword}
              className="rounded-lg bg-primary px-6 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-hover disabled:opacity-50"
            >
              {isSaving ? "Unlocking..." : "Unlock"}
            </button>
          </div>
        ) : (
          <div className="flex items-center gap-2 text-sm text-success">
            <CheckIcon />
            <span>Unlocked and ready</span>
          </div>
        )}
      </Section>

      {/* Provider Credentials - Only show when unlocked */}
      {isUnlocked && (
        <>
          <Section title="Torbox">
            <div className="space-y-4">
              <div>
                <div className="flex items-center justify-between mb-2">
                  <label className="text-sm font-medium text-gray-300">
                    API Key
                  </label>
                  {getCredentialStatus("torbox") && (
                    <StatusBadge status={getCredentialStatus("torbox")!} />
                  )}
                </div>
                <div className="flex gap-2">
                  <input
                    type="password"
                    value={torboxApiKey}
                    onChange={(e) => setTorboxApiKey(e.target.value)}
                    placeholder="Enter Torbox API key"
                    className="flex-1 rounded-lg border border-dark-border bg-dark-surface-elevated px-4 py-2 text-sm text-white placeholder-gray-500 focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
                  />
                  <button
                    onClick={() =>
                      handleSaveCredentials("torbox", torboxApiKey)
                    }
                    disabled={isValidatingTorbox || !torboxApiKey.trim()}
                    className="rounded-lg bg-primary px-6 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-hover disabled:opacity-50"
                  >
                    {isValidatingTorbox ? "Validating..." : "Save"}
                  </button>
                </div>
                <p className="mt-2 text-xs text-gray-500">
                  Get your API key from{" "}
                  <a
                    href="https://torbox.app"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-primary hover:underline"
                  >
                    torbox.app
                  </a>
                </p>
              </div>
            </div>
          </Section>

          <Section title="Real-Debrid">
            <div className="space-y-4">
              <div>
                <div className="flex items-center justify-between mb-2">
                  <label className="text-sm font-medium text-gray-300">
                    API Key
                  </label>
                  {getCredentialStatus("real-debrid") && (
                    <StatusBadge status={getCredentialStatus("real-debrid")!} />
                  )}
                </div>
                <div className="flex gap-2">
                  <input
                    type="password"
                    value={realDebridApiKey}
                    onChange={(e) => setRealDebridApiKey(e.target.value)}
                    placeholder="Enter Real-Debrid API key"
                    className="flex-1 rounded-lg border border-dark-border bg-dark-surface-elevated px-4 py-2 text-sm text-white placeholder-gray-500 focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
                  />
                  <button
                    onClick={() =>
                      handleSaveCredentials("real-debrid", realDebridApiKey)
                    }
                    disabled={
                      isValidatingRealDebrid || !realDebridApiKey.trim()
                    }
                    className="rounded-lg bg-primary px-6 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-hover disabled:opacity-50"
                  >
                    {isValidatingRealDebrid ? "Validating..." : "Save"}
                  </button>
                </div>
                <p className="mt-2 text-xs text-gray-500">
                  Get your API key from{" "}
                  <a
                    href="https://real-debrid.com/apitoken"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-primary hover:underline"
                  >
                    real-debrid.com/apitoken
                  </a>
                </p>
              </div>
            </div>
          </Section>

          {/* Debrid Settings */}
          {settings && (
            <Section title="Debrid Settings">
              <div className="space-y-4">
                <Checkbox
                  label="Enable Debrid Services"
                  checked={settings.enable_debrid}
                  onChange={(checked) =>
                    setSettings({ ...settings, enable_debrid: checked })
                  }
                  description="Use cloud debrid services for faster downloads"
                />

                <Checkbox
                  label="Smart Mode"
                  checked={settings.smart_mode_enabled}
                  onChange={(checked) =>
                    setSettings({ ...settings, smart_mode_enabled: checked })
                  }
                  description="Automatically choose best source (cloud vs P2P) for each torrent"
                />

                <div>
                  <label className="mb-2 block text-sm font-medium text-gray-300">
                    Provider Preference
                  </label>
                  <p className="mb-3 text-xs text-gray-500">
                    Select which providers to use (in order of preference)
                  </p>
                  <div className="space-y-2">
                    <ProviderCheckbox
                      label="Torbox"
                      checked={settings.debrid_preference.includes("torbox")}
                      onChange={() => handleToggleProvider("torbox")}
                      hasCredentials={credentials.some(
                        (c) => c.provider === "torbox" && c.is_configured,
                      )}
                    />
                    <ProviderCheckbox
                      label="Real-Debrid"
                      checked={settings.debrid_preference.includes(
                        "real-debrid",
                      )}
                      onChange={() => handleToggleProvider("real-debrid")}
                      hasCredentials={credentials.some(
                        (c) => c.provider === "real-debrid" && c.is_configured,
                      )}
                    />
                  </div>
                </div>

                <button
                  onClick={handleSaveSettings}
                  disabled={isSaving}
                  className="rounded-lg bg-primary px-6 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-hover disabled:opacity-50"
                >
                  {isSaving ? "Saving..." : "Save Debrid Settings"}
                </button>
              </div>
            </Section>
          )}
        </>
      )}
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

interface ProviderCheckboxProps {
  label: string;
  checked: boolean;
  onChange: () => void;
  hasCredentials: boolean;
}

function ProviderCheckbox({
  label,
  checked,
  onChange,
  hasCredentials,
}: ProviderCheckboxProps) {
  return (
    <label
      className={`flex cursor-pointer items-center gap-3 rounded-lg border p-3 transition-colors ${
        hasCredentials
          ? "border-dark-border hover:bg-dark-surface-elevated"
          : "border-dark-border/50 opacity-50 cursor-not-allowed"
      }`}
    >
      <input
        type="checkbox"
        checked={checked}
        onChange={onChange}
        disabled={!hasCredentials}
        className="h-4 w-4 rounded border-dark-border bg-dark-surface-elevated text-primary focus:ring-2 focus:ring-primary/20 disabled:opacity-50"
      />
      <div className="flex-1 text-sm font-medium text-white">{label}</div>
      {!hasCredentials && (
        <span className="text-xs text-gray-500">API key required</span>
      )}
    </label>
  );
}

interface StatusBadgeProps {
  status: CredentialStatus;
}

function StatusBadge({ status }: StatusBadgeProps) {
  if (!status.is_configured) {
    return <span className="text-xs text-gray-500">Not configured</span>;
  }

  if (status.is_valid === null) {
    return <span className="text-xs text-gray-500">Not validated</span>;
  }

  return status.is_valid ? (
    <span className="flex items-center gap-1 text-xs text-success">
      <CheckIcon size={12} />
      <span>Valid</span>
    </span>
  ) : (
    <span className="flex items-center gap-1 text-xs text-error">
      <span>Invalid</span>
    </span>
  );
}

function CheckIcon({ size = 16 }: { size?: number }) {
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
        d="M5 13l4 4L19 7"
      />
    </svg>
  );
}
