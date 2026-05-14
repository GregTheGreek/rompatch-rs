import { useUpdater } from '../lib/updater';
import { DownloadIcon, RefreshIcon } from '../lib/icons';
import { Button } from './Button';

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

const SHELL =
  'pointer-events-auto fixed bottom-4 left-4 z-50 w-72 rounded-xl border border-bg-border bg-bg-raised px-4 py-3 shadow-soft animate-slide-up';

export function UpdateBanner() {
  const { status, install, dismiss } = useUpdater();

  if (status.kind === 'available') {
    return (
      <div className={SHELL}>
        <div className="flex items-start gap-3">
          <DownloadIcon size={16} className="mt-0.5 text-accent" />
          <div className="flex-1 min-w-0">
            <div className="text-sm font-semibold text-fg">New update available</div>
            <div className="text-xs text-fg-muted mt-0.5">
              Version {status.update.version} - install now or later?
            </div>
          </div>
        </div>
        <div className="mt-3 flex justify-end gap-2">
          <Button size="sm" variant="ghost" onClick={dismiss}>
            Later
          </Button>
          <Button size="sm" variant="primary" onClick={() => void install(status.update)}>
            Install now
          </Button>
        </div>
      </div>
    );
  }

  if (status.kind === 'downloading') {
    const pct =
      status.contentLength != null && status.contentLength > 0
        ? Math.min(100, Math.round((status.downloaded / status.contentLength) * 100))
        : null;
    return (
      <div className={SHELL}>
        <div className="flex items-center gap-3">
          <RefreshIcon size={16} className="text-accent animate-spin" />
          <div className="flex-1 min-w-0 text-sm text-fg">
            Downloading update{pct != null ? ` (${pct}%)` : ''}
            <div className="text-xs text-fg-muted mt-0.5">
              {formatBytes(status.downloaded)}
              {status.contentLength != null ? ` / ${formatBytes(status.contentLength)}` : ''}
            </div>
          </div>
        </div>
      </div>
    );
  }

  if (status.kind === 'ready') {
    return (
      <div className={SHELL}>
        <div className="flex items-center gap-3">
          <RefreshIcon size={16} className="text-accent" />
          <div className="flex-1 min-w-0 text-sm text-fg">Update installed - restarting...</div>
        </div>
      </div>
    );
  }

  return null;
}
