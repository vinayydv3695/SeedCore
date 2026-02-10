import { TorrentInfo, TorrentState } from "../types";
import { formatBytes, formatSpeed, formatProgress, calculateETA } from "../lib/utils";

interface TorrentDetailsProps {
  torrent: TorrentInfo | null;
  isOpen: boolean;
  onClose: () => void;
}

export function TorrentDetails({ torrent, isOpen, onClose }: TorrentDetailsProps) {
  if (!isOpen || !torrent) return null;

  const progress = formatProgress(torrent.downloaded, torrent.size);
  const eta = calculateETA(torrent.size - torrent.downloaded, torrent.download_speed);
  const ratio = torrent.downloaded > 0 ? (torrent.uploaded / torrent.downloaded).toFixed(2) : "0.00";

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-3xl max-h-[90vh] overflow-y-auto scrollbar-dark rounded-xl border border-dark-border bg-dark-surface shadow-2xl">
        {/* Header */}
        <div className="sticky top-0 z-10 flex items-center justify-between border-b border-dark-border bg-dark-surface px-6 py-4">
          <div className="flex-1 min-w-0 pr-4">
            <h2 className="truncate text-xl font-bold text-white">{torrent.name}</h2>
            <div className="mt-1 flex items-center gap-3 text-sm text-gray-400">
              <span className={getStateColor(torrent.state)}>
                {torrent.state}
              </span>
              <span>•</span>
              <span>{formatBytes(torrent.size)}</span>
            </div>
          </div>
          <button
            onClick={onClose}
            className="rounded-lg p-1 text-gray-400 transition-colors hover:bg-dark-surface-elevated hover:text-white"
          >
            <CloseIcon />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 space-y-6">
          {/* Progress Section */}
          <Section title="Progress">
            <div className="space-y-4">
              <div>
                <div className="mb-2 flex items-center justify-between text-sm">
                  <span className="text-gray-400">Downloaded</span>
                  <span className="font-mono text-white">{progress.toFixed(2)}%</span>
                </div>
                <div className="h-3 w-full overflow-hidden rounded-full bg-dark-surface-elevated">
                  <div
                    className={`h-full transition-all duration-300 ${getProgressColor(torrent.state, progress)}`}
                    style={{ width: `${progress}%` }}
                  />
                </div>
                <div className="mt-2 flex items-center justify-between text-xs text-gray-500">
                  <span>{formatBytes(torrent.downloaded)} of {formatBytes(torrent.size)}</span>
                  {torrent.state === TorrentState.Downloading && (
                    <span>ETA: {eta}</span>
                  )}
                </div>
              </div>
            </div>
          </Section>

          {/* Transfer Stats */}
          <Section title="Transfer">
            <div className="grid grid-cols-2 gap-4">
              <StatItem
                label="Download Speed"
                value={formatSpeed(torrent.download_speed)}
                icon={<DownloadIcon />}
                color="text-primary"
              />
              <StatItem
                label="Upload Speed"
                value={formatSpeed(torrent.upload_speed)}
                icon={<UploadIcon />}
                color="text-success"
              />
              <StatItem
                label="Total Downloaded"
                value={formatBytes(torrent.downloaded)}
                icon={<ArrowDownIcon />}
                color="text-blue-400"
              />
              <StatItem
                label="Total Uploaded"
                value={formatBytes(torrent.uploaded)}
                icon={<ArrowUpIcon />}
                color="text-green-400"
              />
              <StatItem
                label="Share Ratio"
                value={ratio}
                icon={<RatioIcon />}
                color="text-purple-400"
              />
              <StatItem
                label="Remaining"
                value={formatBytes(torrent.size - torrent.downloaded)}
                icon={<ClockIcon />}
                color="text-orange-400"
              />
            </div>
          </Section>

          {/* Peers & Seeds */}
          <Section title="Connections">
            <div className="grid grid-cols-2 gap-4">
              <StatItem
                label="Peers"
                value={torrent.peers.toString()}
                icon={<PeersIcon />}
                color="text-cyan-400"
              />
              <StatItem
                label="Seeds"
                value={torrent.seeds.toString()}
                icon={<SeedsIcon />}
                color="text-green-400"
              />
            </div>
          </Section>

          {/* General Info */}
          <Section title="General">
            <div className="space-y-3">
              <InfoRow label="Info Hash" value={torrent.id} mono />
              <InfoRow label="Size" value={formatBytes(torrent.size)} />
              <InfoRow label="State" value={torrent.state} />
            </div>
          </Section>
        </div>

        {/* Footer */}
        <div className="sticky bottom-0 flex items-center justify-end border-t border-dark-border bg-dark-surface px-6 py-4">
          <button
            onClick={onClose}
            className="rounded-lg bg-primary px-6 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-hover"
          >
            Close
          </button>
        </div>
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
      <h3 className="mb-3 text-sm font-semibold uppercase tracking-wider text-gray-500">
        {title}
      </h3>
      {children}
    </div>
  );
}

interface StatItemProps {
  label: string;
  value: string;
  icon: React.ReactNode;
  color: string;
}

function StatItem({ label, value, icon, color }: StatItemProps) {
  return (
    <div className="rounded-lg border border-dark-border bg-dark-surface-elevated p-4">
      <div className="flex items-center gap-3">
        <div className={color}>{icon}</div>
        <div className="flex-1 min-w-0">
          <div className="text-xs text-gray-500">{label}</div>
          <div className="truncate text-lg font-semibold text-white">{value}</div>
        </div>
      </div>
    </div>
  );
}

interface InfoRowProps {
  label: string;
  value: string;
  mono?: boolean;
}

function InfoRow({ label, value, mono }: InfoRowProps) {
  return (
    <div className="flex items-start justify-between gap-4 rounded-lg border border-dark-border bg-dark-surface-elevated p-3">
      <span className="text-sm text-gray-500">{label}</span>
      <span className={`text-sm text-white ${mono ? "font-mono break-all" : ""}`}>
        {value}
      </span>
    </div>
  );
}

// Helper Functions

function getStateColor(state: TorrentState): string {
  switch (state) {
    case TorrentState.Downloading:
      return "text-primary font-medium";
    case TorrentState.Seeding:
      return "text-success font-medium";
    case TorrentState.Paused:
      return "text-gray-400";
    case TorrentState.Error:
      return "text-error font-medium";
    default:
      return "text-gray-400";
  }
}

function getProgressColor(state: TorrentState, progress: number): string {
  if (progress >= 100) return "bg-success";
  if (state === TorrentState.Downloading) return "bg-primary";
  return "bg-gray-500";
}

// Icons

function CloseIcon() {
  return (
    <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
    </svg>
  );
}

function DownloadIcon() {
  return (
    <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10" />
    </svg>
  );
}

function UploadIcon() {
  return (
    <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
    </svg>
  );
}

function ArrowDownIcon() {
  return (
    <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
    </svg>
  );
}

function ArrowUpIcon() {
  return (
    <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 10l7-7m0 0l7 7m-7-7v18" />
    </svg>
  );
}

function RatioIcon() {
  return (
    <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
    </svg>
  );
}

function ClockIcon() {
  return (
    <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
  );
}

function PeersIcon() {
  return (
    <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
    </svg>
  );
}

function SeedsIcon() {
  return (
    <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z" />
    </svg>
  );
}
