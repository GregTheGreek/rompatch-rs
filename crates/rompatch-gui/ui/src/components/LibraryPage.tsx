import { useCallback, useEffect, useRef, useState } from 'react';
import { LibraryEntryRow } from './LibraryEntryRow';
import { useToast } from './Toast';
import { cn } from '../lib/cn';
import { formatIpcError } from '../lib/errors';
import { CheckIcon, DatabaseIcon, PlusCircleIcon, TrashIcon, XIcon } from '../lib/icons';
import {
  libraryDeleteRom,
  libraryImportRom,
  libraryList,
  libraryListRoms,
  libraryRoot,
  pickFile,
} from '../lib/tauri';
import { useDropTarget } from '../lib/useDropTarget';
import { HEADER_DISPLAY } from '../lib/types';
import type { HeaderKind, LibraryEntry, LibraryRomEntry } from '../lib/types';

interface Group {
  source_rom_hash: string;
  source_rom_name: string;
  source_rom_size: number;
  header: HeaderKind | null;
  entries: LibraryEntry[];
}

function mergeIntoGroups(
  roms: LibraryRomEntry[],
  entries: LibraryEntry[],
): Group[] {
  const map = new Map<string, Group>();

  // Seed groups from explicit ROM entries so bare ROMs show up even with no patches.
  for (const rom of roms) {
    map.set(rom.rom_hash, {
      source_rom_hash: rom.rom_hash,
      source_rom_name: rom.rom_name,
      source_rom_size: rom.rom_size,
      header: rom.header,
      entries: [],
    });
  }

  // Layer patch applications on top.
  for (const entry of entries) {
    const existing = map.get(entry.source_rom_hash);
    if (existing) {
      existing.entries.push(entry);
      // Promote header/name info from the application if not set on the rom-only group.
      if (!existing.header && entry.header) existing.header = entry.header;
    } else {
      map.set(entry.source_rom_hash, {
        source_rom_hash: entry.source_rom_hash,
        source_rom_name: entry.source_rom_name,
        source_rom_size: entry.source_rom_size,
        header: entry.header,
        entries: [entry],
      });
    }
  }

  const groups = Array.from(map.values());
  // Sort: groups with patch applications first by most-recent application, then bare ROMs by name.
  groups.sort((a, b) => {
    const aLatest = a.entries[0]?.applied_at ?? '';
    const bLatest = b.entries[0]?.applied_at ?? '';
    if (aLatest && bLatest) return bLatest.localeCompare(aLatest);
    if (aLatest) return -1;
    if (bLatest) return 1;
    return a.source_rom_name.localeCompare(b.source_rom_name);
  });
  return groups;
}

export function LibraryPage() {
  const { toast } = useToast();
  const [roms, setRoms] = useState<LibraryRomEntry[] | null>(null);
  const [entries, setEntries] = useState<LibraryEntry[] | null>(null);
  const [root, setRoot] = useState<string | null>(null);
  const [importing, setImporting] = useState(false);
  const [dragHover, setDragHover] = useState(false);

  // Avoid stale closures over `importing` in the drop handler.
  const importingRef = useRef(false);
  importingRef.current = importing;

  const refresh = useCallback(async () => {
    try {
      const [r, romsList, entriesList] = await Promise.all([
        libraryRoot(),
        libraryListRoms(),
        libraryList(),
      ]);
      setRoot(r);
      setRoms(romsList);
      setEntries(entriesList);
    } catch (err) {
      toast({
        title: 'Could not load library',
        description: formatIpcError(err),
        variant: 'error',
      });
      setRoms([]);
      setEntries([]);
    }
  }, [toast]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  async function handleImportRom() {
    setImporting(true);
    try {
      const picked = await pickFile('Select ROM to import');
      if (!picked) return;
      const entry = await libraryImportRom(picked);
      toast({
        title: 'ROM imported',
        description: entry.rom_name,
        variant: 'success',
      });
      await refresh();
    } catch (err) {
      toast({
        title: 'Import failed',
        description: formatIpcError(err),
        variant: 'error',
      });
    } finally {
      setImporting(false);
    }
  }

  const handleDrop = useCallback(
    async (paths: string[]) => {
      if (importingRef.current) return;
      setImporting(true);
      let ok = 0;
      let fail = 0;
      try {
        for (const path of paths) {
          try {
            await libraryImportRom(path);
            ok += 1;
          } catch (err) {
            fail += 1;
            // Show a single toast per failure so the user knows which path failed.
            toast({
              title: 'Import failed',
              description: `${path}: ${formatIpcError(err)}`,
              variant: 'error',
            });
          }
        }
        if (ok > 0) {
          toast({
            title: ok === 1 ? 'ROM imported' : `${ok} ROMs imported`,
            description: fail > 0 ? `${fail} failed` : undefined,
            variant: fail > 0 ? 'warning' : 'success',
          });
          await refresh();
        }
      } finally {
        setImporting(false);
      }
    },
    [refresh, toast],
  );

  const dropRef = useDropTarget<HTMLDivElement>(handleDrop, setDragHover);

  if (roms === null || entries === null) {
    return (
      <div className="flex-1 flex items-center justify-center text-fg-subtle text-sm">
        Loading library…
      </div>
    );
  }

  const groups = mergeIntoGroups(roms, entries);
  const totalRoms = roms.length;
  const totalPatches = entries.length;

  return (
    <div
      ref={dropRef}
      className={cn(
        'flex-1 flex flex-col min-h-0 px-6 pb-6 pt-2 relative',
        'transition-colors duration-150',
        dragHover && 'bg-accent-subtle/10',
      )}
    >
      {dragHover && (
        <div
          aria-hidden
          className={cn(
            'pointer-events-none absolute inset-3 rounded-xl',
            'border-2 border-dashed border-accent/60 bg-bg/40 backdrop-blur-sm',
            'flex items-center justify-center z-10',
            'animate-fade-in',
          )}
        >
          <div className="flex flex-col items-center gap-2 text-accent">
            <PlusCircleIcon size={28} strokeWidth={1.5} />
            <div className="text-sm font-medium">Drop ROMs to import</div>
            <div className="text-[11px] text-fg-muted">
              Multiple files supported. SHA-256 deduplicated automatically.
            </div>
          </div>
        </div>
      )}
      <header className="flex items-center justify-between gap-3 mb-4 px-1">
        <div className="flex items-center gap-2 min-w-0">
          <DatabaseIcon size={14} className="text-fg-subtle shrink-0" />
          <h2 className="text-sm font-semibold text-fg">Library</h2>
          <span className="text-[11px] text-fg-subtle font-mono uppercase tracking-wider">
            {totalRoms} {totalRoms === 1 ? 'rom' : 'roms'} · {totalPatches}{' '}
            {totalPatches === 1 ? 'patch' : 'patches'}
          </span>
        </div>
        <div className="flex items-center gap-3 min-w-0">
          {root && (
            <div
              className="text-[11px] font-mono text-fg-subtle truncate max-w-[18rem]"
              title={root}
            >
              {root}
            </div>
          )}
          <button
            type="button"
            onClick={handleImportRom}
            disabled={importing}
            className={cn(
              'inline-flex items-center gap-1.5 h-7 px-3 rounded-md text-xs font-medium shrink-0',
              'bg-accent text-white hover:bg-accent-hover active:bg-accent-active',
              'disabled:opacity-50 disabled:cursor-not-allowed',
              'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40',
            )}
          >
            <PlusCircleIcon size={12} strokeWidth={2.2} />
            <span>{importing ? 'Importing…' : 'Import ROM'}</span>
          </button>
        </div>
      </header>

      {groups.length === 0 ? (
        <EmptyState onImport={handleImportRom} importing={importing} />
      ) : (
        <div className="flex-1 overflow-y-auto flex flex-col gap-5">
          {groups.map((group) => (
            <GroupBlock
              key={group.source_rom_hash}
              group={group}
              onChanged={() => void refresh()}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function GroupBlock({
  group,
  onChanged,
}: {
  group: Group;
  onChanged: () => void;
}) {
  const { toast } = useToast();
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [deleting, setDeleting] = useState(false);

  const patchCount = group.entries.length;
  const cascadeLabel =
    patchCount === 0
      ? 'Delete ROM?'
      : `Delete ROM + ${patchCount} ${patchCount === 1 ? 'patch' : 'patches'}?`;

  async function handleDeleteConfirm() {
    setDeleting(true);
    try {
      await libraryDeleteRom(group.source_rom_hash);
      toast({
        title: 'ROM deleted',
        description: group.source_rom_name,
        variant: 'success',
      });
      onChanged();
    } catch (err) {
      toast({
        title: 'Delete failed',
        description: formatIpcError(err),
        variant: 'error',
      });
      setDeleting(false);
      setConfirmDelete(false);
    }
  }

  return (
    <section className="group/section rounded-xl border border-bg-border bg-bg-raised/40">
      <header className="flex items-center justify-between gap-2 px-4 py-2 border-b border-bg-border/60">
        <div className="flex items-center gap-2 min-w-0">
          <span className="text-sm font-medium text-fg truncate" title={group.source_rom_name}>
            {group.source_rom_name}
          </span>
          <span className="text-[11px] text-fg-subtle font-mono">
            {group.source_rom_hash.slice(0, 7)}…
          </span>
          {group.header && (
            <span className="text-[10px] uppercase tracking-wider font-mono text-fg-subtle">
              {HEADER_DISPLAY[group.header]}
            </span>
          )}
        </div>
        {confirmDelete ? (
          <div className="flex items-center gap-1.5" role="group" aria-label="Confirm delete ROM">
            <span className="text-[11px] text-fg-muted">{cascadeLabel}</span>
            <GroupActionButton
              label="Confirm delete"
              tone="danger"
              busy={deleting}
              onClick={handleDeleteConfirm}
            >
              <CheckIcon size={13} />
            </GroupActionButton>
            <GroupActionButton
              label="Cancel delete"
              onClick={() => setConfirmDelete(false)}
            >
              <XIcon size={13} />
            </GroupActionButton>
          </div>
        ) : (
          <div className="flex items-center gap-2">
            <span className="text-[11px] text-fg-subtle font-mono uppercase tracking-wider">
              {patchCount === 0
                ? 'no patches yet'
                : `${patchCount} ${patchCount === 1 ? 'patch' : 'patches'}`}
            </span>
            <div className="opacity-0 group-hover/section:opacity-100 focus-within:opacity-100 transition-opacity">
              <GroupActionButton label="Delete ROM" onClick={() => setConfirmDelete(true)}>
                <TrashIcon size={13} />
              </GroupActionButton>
            </div>
          </div>
        )}
      </header>
      {group.entries.length > 0 && (
        <div className="flex flex-col gap-0.5 p-1.5">
          {group.entries.map((entry) => (
            <LibraryEntryRow
              key={entry.id}
              entry={entry}
              onDeleted={onChanged}
            />
          ))}
        </div>
      )}
    </section>
  );
}

function GroupActionButton({
  label,
  onClick,
  busy,
  tone = 'default',
  children,
}: {
  label: string;
  onClick: () => void;
  busy?: boolean;
  tone?: 'default' | 'danger';
  children: React.ReactNode;
}) {
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

function EmptyState({
  onImport,
  importing,
}: {
  onImport: () => void;
  importing: boolean;
}) {
  return (
    <div className="flex-1 flex flex-col items-center justify-center text-center px-6 gap-4">
      <div className="text-fg-subtle">
        <DatabaseIcon size={36} strokeWidth={1.5} />
      </div>
      <div className="text-sm text-fg">Your library is empty</div>
      <div className="text-xs text-fg-muted max-w-sm">
        Import a ROM directly, or apply a patch from the Patch page — both flows
        auto-deduplicate by SHA-256.
      </div>
      <button
        type="button"
        onClick={onImport}
        disabled={importing}
        className={cn(
          'inline-flex items-center gap-1.5 h-8 px-3.5 rounded-md text-xs font-medium mt-2',
          'bg-accent text-white hover:bg-accent-hover active:bg-accent-active',
          'disabled:opacity-50 disabled:cursor-not-allowed',
          'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40',
        )}
      >
        <PlusCircleIcon size={13} strokeWidth={2.2} />
        <span>{importing ? 'Importing…' : 'Import ROM'}</span>
      </button>
    </div>
  );
}
