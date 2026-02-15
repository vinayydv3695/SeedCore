import { TorrentInfo } from "../../types";
import { formatBytes, formatSpeed, calculateETA } from "../../lib/utils";
import {
  HardDrive,
  Activity,
  Network,
  FileText
} from "lucide-react";
import { cn } from "../../lib/utils";

interface GeneralTabProps {
  torrent: TorrentInfo;
}

export function GeneralTab({ torrent }: GeneralTabProps) {
  const progress = torrent.size > 0 ? (torrent.downloaded / torrent.size) * 100 : 0;
  const remaining = torrent.size - torrent.downloaded;
  const ratio = torrent.downloaded > 0 ? torrent.uploaded / torrent.downloaded : 0;
  const eta = calculateETA(remaining, torrent.download_speed);

  return (
    <div className="p-4 space-y-6">
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Transfer Section */}
        <Section title="Transfer" icon={<Activity className="h-4 w-4" />}>
          <InfoRow label="Downloaded" value={formatBytes(torrent.downloaded)} />
          <InfoRow label="Uploaded" value={formatBytes(torrent.uploaded)} />
          <InfoRow
            label="Download Speed"
            value={torrent.download_speed > 0 ? formatSpeed(torrent.download_speed) : "-"}
            valueClassName={torrent.download_speed > 0 ? "text-primary font-bold" : ""}
          />
          <InfoRow
            label="Upload Speed"
            value={torrent.upload_speed > 0 ? formatSpeed(torrent.upload_speed) : "-"}
            valueClassName={torrent.upload_speed > 0 ? "text-success font-bold" : ""}
          />
          <InfoRow label="Share Ratio" value={ratio.toFixed(2)} />
          <InfoRow label="Time Active" value="-" />
        </Section>

        {/* Progress Section */}
        <Section title="Progress" icon={<HardDrive className="h-4 w-4" />}>
          <InfoRow label="Total Size" value={formatBytes(torrent.size)} />
          <InfoRow label="Downloaded" value={`${formatBytes(torrent.downloaded)} (${progress.toFixed(1)}%)`} />
          <InfoRow label="Remaining" value={formatBytes(remaining)} />
          <InfoRow label="ETA" value={eta} />
          <div className="mt-4">
            <div className="flex justify-between text-xs text-text-tertiary mb-1.5">
              <span>Progress</span>
              <span>{progress.toFixed(1)}%</span>
            </div>
            <div className="w-full bg-dark-bg border border-dark-border rounded-full h-2.5 overflow-hidden">
              <div
                className={cn("h-full relative overflow-hidden transition-all duration-300",
                  progress >= 100 ? "bg-success" : "bg-primary"
                )}
                style={{ width: `${Math.min(progress, 100)}%` }}
              >
                {progress < 100 && <div className="absolute inset-0 bg-white/20 animate-[shimmer_2s_infinite]" />}
              </div>
            </div>
          </div>
        </Section>

        {/* Connection Section */}
        <Section title="Connection" icon={<Network className="h-4 w-4" />}>
          <InfoRow label="Peers" value={`${torrent.peers} connected`} />
          <InfoRow label="Seeds" value={`${torrent.seeds} connected`} />
        </Section>

        {/* General Info Section */}
        <Section title="General" icon={<FileText className="h-4 w-4" />}>
          <InfoRow label="Name" value={torrent.name} className="truncate" />
          <InfoRow label="Status" value={torrent.state} />
          <InfoRow label="Info Hash" value={torrent.id.substring(0, 8)} mono />
          <InfoRow label="Save Path" value="/downloads" />
          <InfoRow label="Added On" value="Just now" />
        </Section>
      </div>
    </div>
  );
}

interface SectionProps {
  title: string;
  icon?: React.ReactNode;
  children: React.ReactNode;
}

function Section({ title, icon, children }: SectionProps) {
  return (
    <div className="bg-dark-surface-elevated/50 rounded-lg p-4 border border-dark-border/50">
      <div className="flex items-center gap-2 mb-4 pb-2 border-b border-dark-border/50">
        {icon && <span className="text-text-tertiary">{icon}</span>}
        <h3 className="text-xs font-semibold text-text-secondary uppercase tracking-wider">
          {title}
        </h3>
      </div>
      <div className="space-y-2.5">{children}</div>
    </div>
  );
}

interface InfoRowProps {
  label: string;
  value: string | React.ReactNode;
  valueClassName?: string;
  mono?: boolean;
  className?: string; // for container
}

function InfoRow({ label, value, valueClassName, mono = false, className }: InfoRowProps) {
  return (
    <div className={cn("flex justify-between items-start text-sm", className)}>
      <span className="text-text-tertiary w-1/3 shrink-0">{label}</span>
      <span className={cn(
        "text-right break-all text-text-primary",
        mono && "font-mono text-xs",
        valueClassName
      )}>
        {value}
      </span>
    </div>
  );
}
