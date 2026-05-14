import { useUpdater } from '../lib/updater';
import { DownloadIcon, RefreshIcon } from '../lib/icons';
import { Button } from './Button';

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

export function UpdateBanner() {
  const { status, install } = useUpdater();

  if (status.kind === 'idle' || status.kind === 'checking') return null;

  if (status.kind === 'available') {
    return (
      <div className="flex items-center gap-3 border-b border-accent/30 bg-accent-subtle/30 px-4 py-2 text-sm">
        <DownloadIcon size={16} className="text-accent" />
        <span className="flex-1">
          Update available: <span className="font-medium">v{status.update.version}</span>
        </span>
        <Button size="sm" variant="primary" onClick={() => void install(status.update)}>
          Install &amp; restart
        </Button>
      </div>
    );
  }

  if (status.kind === 'downloading') {
    const pct =
      status.contentLength != null && status.contentLength > 0
        ? Math.min(100, Math.round((status.downloaded / status.contentLength) * 100))
        : null;
    return (
      <div className="flex items-center gap-3 border-b border-accent/30 bg-accent-subtle/30 px-4 py-2 text-sm">
        <RefreshIcon size={16} className="text-accent animate-spin" />
        <span className="flex-1">
          Downloading update{pct != null ? ` (${pct}%)` : ''} - {formatBytes(status.downloaded)}
          {status.contentLength != null ? ` / ${formatBytes(status.contentLength)}` : ''}
        </span>
      </div>
    );
  }

  if (status.kind === 'ready') {
    return (
      <div className="flex items-center gap-3 border-b border-accent/30 bg-accent-subtle/30 px-4 py-2 text-sm">
        <RefreshIcon size={16} className="text-accent" />
        <span className="flex-1">Update installed - restarting...</span>
      </div>
    );
  }

  // error
  return null;
}
