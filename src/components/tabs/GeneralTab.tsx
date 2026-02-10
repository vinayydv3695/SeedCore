import { TorrentInfo } from "../../types";
import { formatBytes, formatSpeed, calculateETA } from "../../lib/utils";

interface GeneralTabProps {
  torrent: TorrentInfo;
}

export function GeneralTab({ torrent }: GeneralTabProps) {
  const progress = torrent.size > 0 ? (torrent.downloaded / torrent.size) * 100 : 0;
  const remaining = torrent.size - torrent.downloaded;
  const ratio = torrent.downloaded > 0 ? torrent.uploaded / torrent.downloaded : 0;
  const eta = calculateETA(remaining, torrent.download_speed);

  return (
    <div className="p-4">
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Transfer Section */}
        <Section title="Transfer">
          <InfoRow label="Downloaded" value={formatBytes(torrent.downloaded)} />
          <InfoRow label="Uploaded" value={formatBytes(torrent.uploaded)} />
          <InfoRow
            label="Download Speed"
            value={torrent.download_speed > 0 ? formatSpeed(torrent.download_speed) : "-"}
            valueColor="text-primary"
          />
          <InfoRow
            label="Upload Speed"
            value={torrent.upload_speed > 0 ? formatSpeed(torrent.upload_speed) : "-"}
            valueColor="text-success"
          />
          <InfoRow label="Share Ratio" value={ratio.toFixed(3)} />
          <InfoRow label="Time Active" value="-" />
        </Section>

        {/* Progress Section */}
        <Section title="Progress">
          <InfoRow label="Total Size" value={formatBytes(torrent.size)} />
          <InfoRow label="Downloaded" value={`${formatBytes(torrent.downloaded)} (${progress.toFixed(1)}%)`} />
          <InfoRow label="Remaining" value={formatBytes(remaining)} />
          <InfoRow label="ETA" value={eta} />
          <div className="mt-3">
            <div className="flex justify-between text-xs text-gray-400 mb-1">
              <span>Progress</span>
              <span>{progress.toFixed(2)}%</span>
            </div>
            <div className="w-full bg-dark-border rounded-full h-2 overflow-hidden">
              <div
                className={`h-full rounded-full transition-all duration-300 ${
                  progress >= 100 ? "bg-success" : progress >= 50 ? "bg-primary" : "bg-warning"
                }`}
                style={{ width: `${Math.min(progress, 100)}%` }}
              />
            </div>
          </div>
        </Section>

        {/* Connection Section */}
        <Section title="Connection">
          <InfoRow label="Peers" value={`${torrent.peers} connected`} />
          <InfoRow label="Seeds" value={`${torrent.seeds} connected`} />
          <InfoRow label="Total Peers" value="-" />
          <InfoRow label="Total Seeds" value="-" />
        </Section>

        {/* General Info Section */}
        <Section title="General">
          <InfoRow label="Name" value={torrent.name} />
          <InfoRow label="Status" value={torrent.state} />
          <InfoRow label="Info Hash" value={torrent.id.substring(0, 40)} mono />
          <InfoRow label="Save Path" value="-" />
          <InfoRow label="Created By" value="-" />
          <InfoRow label="Created On" value="-" />
          <InfoRow label="Added On" value="-" />
          <InfoRow label="Comment" value="-" />
        </Section>
      </div>
    </div>
  );
}

interface SectionProps {
  title: string;
  children: React.ReactNode;
}

function Section({ title, children }: SectionProps) {
  return (
    <div>
      <h3 className="text-sm font-semibold text-gray-400 uppercase tracking-wide mb-3 pb-2 border-b border-dark-border">
        {title}
      </h3>
      <div className="space-y-2">{children}</div>
    </div>
  );
}

interface InfoRowProps {
  label: string;
  value: string;
  valueColor?: string;
  mono?: boolean;
}

function InfoRow({ label, value, valueColor = "text-white", mono = false }: InfoRowProps) {
  return (
    <div className="flex justify-between items-start text-sm">
      <span className="text-gray-400 w-1/3">{label}:</span>
      <span className={`${valueColor} ${mono ? "font-mono text-xs" : ""} w-2/3 text-right break-all`}>
        {value}
      </span>
    </div>
  );
}
