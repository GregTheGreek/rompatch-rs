import { useState } from 'react';
import { Badge } from './Badge';
import { useToast } from './Toast';
import { cn } from '../lib/cn';
import { formatIpcError } from '../lib/errors';
import {
  CheckIcon,
  DownloadIcon,
  RefreshIcon,
  ShieldCheckIcon,
  TrashIcon,
  XIcon,
} from '../lib/icons';
import {
  libraryDeleteEntry,
  libraryExport,
  libraryReapply,
  libraryVerify,
  pickSavePath,
} from '../lib/tauri';
import { FORMAT_DISPLAY } from '../lib/types';
import type { LibraryEntry, VerifyStatus } from '../lib/types';

interface LibraryEntryRowProps {
  entry: LibraryEntry;
  onDeleted?: (entryId: string) => void;
}

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  if (n < 1024 * 1024 * 1024) return `${(n / (1024 * 1024)).toFixed(1)} MB`;
  return `${(n / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function formatDate(iso: string): string {
  // iso is "YYYY-MM-DDTHH:MM:SSZ" — render the date portion.
  return iso.slice(0, 10);
}

function statusToneClass(status: VerifyStatus | null): string {
  switch (status) {
    case 'match':
      return 'text-success bg-success/15 border-success/40';
    case 'mismatch':
      return 'text-danger bg-danger/15 border-danger/40';
    case 'missing':
      return 'text-warning bg-warning/15 border-warning/40';
    default:
      return 'text-fg-subtle bg-bg-input border-bg-border';
  }
}

function statusGlyph(status: VerifyStatus | null): string {
  switch (status) {
    case 'match':
      return '✓';
    case 'mismatch':
      return '⚠';
    case 'missing':
      return '?';
    default:
      return '·';
  }
}

export function LibraryEntryRow({ entry, onDeleted }: LibraryEntryRowProps) {
  const { toast } = useToast();
  const [status, setStatus] = useState<VerifyStatus | null>(null);
  const [busy, setBusy] = useState<'verify' | 'reapply' | 'delete' | null>(null);
  const [expanded, setExpanded] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);

  async function handleVerify(e: React.MouseEvent) {
    e.stopPropagation();
    setBusy('verify');
    try {
      const result = await libraryVerify(entry.id);
      setStatus(result);
    } catch (err) {
      toast({
        title: 'Verify failed',
        description: formatIpcError(err),
        variant: 'error',
      });
    } finally {
      setBusy(null);
    }
  }

  async function handleReapply(e: React.MouseEvent) {
    e.stopPropagation();
    setBusy('reapply');
    try {
      const result = await libraryReapply(entry.id);
      setStatus(result);
      toast({
        title: result === 'match' ? 'Re-applied' : 'Re-apply mismatch',
        description:
          result === 'match'
            ? 'Output regenerated and verified.'
            : 'Re-applied output hash did not match the recorded hash.',
        variant: result === 'match' ? 'success' : 'warning',
      });
    } catch (err) {
      toast({
        title: 'Re-apply failed',
        description: formatIpcError(err),
        variant: 'error',
      });
    } finally {
      setBusy(null);
    }
  }

  function handleDeleteRequest(e: React.MouseEvent) {
    e.stopPropagation();
    setConfirmDelete(true);
  }

  function handleDeleteCancel(e: React.MouseEvent) {
    e.stopPropagation();
    setConfirmDelete(false);
  }

  async function handleDeleteConfirm(e: React.MouseEvent) {
    e.stopPropagation();
    setBusy('delete');
    try {
      await libraryDeleteEntry(entry.id);
      toast({
        title: 'Patch deleted',
        description: entry.patch_name,
        variant: 'success',
      });
      onDeleted?.(entry.id);
    } catch (err) {
      toast({
        title: 'Delete failed',
        description: formatIpcError(err),
        variant: 'error',
      });
      setBusy(null);
      setConfirmDelete(false);
    }
  }

  async function handleExport(e: React.MouseEvent) {
    e.stopPropagation();
    try {
      const dest = await pickSavePath(entry.output_name, 'Export patched ROM');
      if (!dest) return;
      await libraryExport(entry.id, dest);
      toast({
        title: 'Exported',
        description: dest,
        variant: 'success',
      });
    } catch (err) {
      toast({
        title: 'Export failed',
        description: formatIpcError(err),
        variant: 'error',
      });
    }
  }

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={() => setExpanded((v) => !v)}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          setExpanded((v) => !v);
        }
      }}
      className={cn(
        'group flex flex-col gap-1 px-3 py-2 rounded-md select-none cursor-pointer',
        'hover:bg-bg-input/40 transition-colors',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40',
      )}
    >
      <div className="flex items-center gap-3 min-w-0">
        <span
          className={cn(
            'inline-flex items-center justify-center h-5 w-5 rounded-full border text-[10px] shrink-0',
            statusToneClass(status),
          )}
          title={status ?? 'unknown'}
        >
          {statusGlyph(status)}
        </span>
        <div className="flex-1 min-w-0">
          <div className="text-sm text-fg truncate" title={entry.patch_name}>
            {entry.patch_name}
          </div>
          <div className="text-[11px] text-fg-subtle font-mono truncate">
            {formatDate(entry.applied_at)} · {formatBytes(entry.output_size)}
          </div>
        </div>
        <Badge tone="accent">{FORMAT_DISPLAY[entry.patch_format]}</Badge>
        {confirmDelete ? (
          <div
            className="flex items-center gap-1.5 pl-1"
            role="group"
            aria-label="Confirm delete"
          >
            <span className="text-[11px] text-fg-muted">Delete?</span>
            <ActionButton
              label="Confirm delete"
              busy={busy === 'delete'}
              tone="danger"
              onClick={handleDeleteConfirm}
            >
              <CheckIcon size={13} />
            </ActionButton>
            <ActionButton
              label="Cancel delete"
              onClick={handleDeleteCancel}
            >
              <XIcon size={13} />
            </ActionButton>
          </div>
        ) : (
          <div className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 focus-within:opacity-100 transition-opacity">
            <ActionButton
              label="Verify"
              busy={busy === 'verify'}
              onClick={handleVerify}
            >
              <ShieldCheckIcon size={13} />
            </ActionButton>
            <ActionButton
              label="Re-apply"
              busy={busy === 'reapply'}
              onClick={handleReapply}
            >
              <RefreshIcon size={13} />
            </ActionButton>
            <ActionButton label="Export patched ROM" onClick={handleExport}>
              <DownloadIcon size={13} />
            </ActionButton>
            <ActionButton label="Delete" onClick={handleDeleteRequest}>
              <TrashIcon size={13} />
            </ActionButton>
          </div>
        )}
      </div>

      {expanded && (
        <div className="pl-8 pr-2 pt-2 text-[11px] font-mono text-fg-subtle space-y-0.5 animate-fade-in">
          <div>
            <span className="text-fg-muted">output:</span>{' '}
            <span className="text-fg break-all">{entry.output_name}</span>
          </div>
          <div>
            <span className="text-fg-muted">source sha256:</span> {entry.source_rom_hash}
          </div>
          <div>
            <span className="text-fg-muted">patch sha256:</span> {entry.patch_hash}
          </div>
          <div>
            <span className="text-fg-muted">output sha256:</span> {entry.output_hash}
          </div>
        </div>
      )}
    </div>
  );
}

interface ActionButtonProps {
  label: string;
  onClick: (e: React.MouseEvent) => void;
  busy?: boolean;
  tone?: 'default' | 'danger';
  children: React.ReactNode;
}

function ActionButton({ label, onClick, busy, tone = 'default', children }: ActionButtonProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      title={label}
      aria-label={label}
      disabled={busy}
      className={cn(
        'inline-flex items-center justify-center h-7 w-7 rounded-md transition-colors',
        tone === 'danger'
          ? 'text-danger hover:text-white hover:bg-danger focus-visible:ring-danger/40'
          : 'text-fg-subtle hover:text-fg hover:bg-bg-input focus-visible:ring-accent/40',
        'disabled:opacity-50 disabled:cursor-not-allowed',
        'focus-visible:outline-none focus-visible:ring-2',
      )}
    >
      {children}
    </button>
  );
}
