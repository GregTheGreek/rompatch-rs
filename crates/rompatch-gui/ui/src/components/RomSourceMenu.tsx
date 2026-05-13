import { useCallback, useEffect, useRef, useState } from 'react';
import { cn } from '../lib/cn';
import { DatabaseIcon, UploadIcon } from '../lib/icons';
import { libraryListRoms } from '../lib/tauri';
import { HEADER_DISPLAY } from '../lib/types';
import type { LibraryRomEntry } from '../lib/types';

interface RomSourceMenuProps {
  /** User chose "Import" - the parent should open a file dialog. */
  onImport: () => void;
  /** User chose a library entry. */
  onPickLibrary: (entry: LibraryRomEntry) => void;
  /**
   * Render the trigger. Receives `onClick` to open/close the menu and `open`
   * so the trigger can reflect open state visually.
   */
  trigger: (props: { onClick: () => void; open: boolean }) => React.ReactNode;
}

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  if (n < 1024 * 1024 * 1024) return `${(n / (1024 * 1024)).toFixed(1)} MB`;
  return `${(n / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

export function RomSourceMenu({ onImport, onPickLibrary, trigger }: RomSourceMenuProps) {
  const [open, setOpen] = useState(false);
  const [roms, setRoms] = useState<LibraryRomEntry[]>([]);
  const containerRef = useRef<HTMLDivElement>(null);

  const close = useCallback(() => setOpen(false), []);

  // Refresh library list every time the menu opens (cheap, single JSON read).
  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    libraryListRoms()
      .then((list) => {
        if (!cancelled) setRoms(list);
      })
      .catch(() => {
        if (!cancelled) setRoms([]);
      });
    return () => {
      cancelled = true;
    };
  }, [open]);

  // Click-outside / Escape to dismiss.
  useEffect(() => {
    if (!open) return;
    function onClick(e: MouseEvent) {
      if (!containerRef.current) return;
      if (!containerRef.current.contains(e.target as Node)) close();
    }
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') close();
    }
    document.addEventListener('mousedown', onClick);
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('mousedown', onClick);
      document.removeEventListener('keydown', onKey);
    };
  }, [open, close]);

  return (
    <div
      ref={containerRef}
      className="relative flex-1 basis-0 min-w-0 flex"
    >
      {trigger({
        onClick: () => setOpen((v) => !v),
        open,
      })}
      {open && (
        <div
          role="menu"
          // Pill is `min-w-[10rem]` (5rem half) centered inside this wrapper;
          // `left: calc(50% - 5rem)` aligns the menu's left edge with the
          // pill's left edge.
          // The label slot (h-5) + the gap-5 above it (1.25rem each) sit
          // below the pill; `top: calc(100% - 2.5rem)` lifts the menu up so
          // its top sits flush with the pill's bottom edge.
          style={{ left: 'calc(50% - 5rem)', top: 'calc(100% - 2.5rem)' }}
          className={cn(
            'absolute mt-2 z-40',
            'min-w-[18rem] max-w-[26rem]',
            'rounded-lg border border-bg-border bg-bg-raised shadow-soft',
            'flex flex-col p-1 animate-fade-in',
          )}
        >
          <MenuItem
            icon={<UploadIcon size={13} />}
            label="Import"
            sublabel="Pick a ROM file from disk"
            onClick={() => {
              close();
              onImport();
            }}
          />

          <div className="my-1 h-px bg-bg-border/60" />

          <div className="px-2.5 pt-1 pb-1.5 text-[10px] uppercase tracking-wider font-mono text-fg-subtle">
            From library
          </div>

          {roms.length === 0 ? (
            <div className="px-3 pb-2 text-[11px] text-fg-subtle">
              Library is empty. Import a ROM here or from the Library page.
            </div>
          ) : (
            // ~52px per row × 3 = 156px ≈ 10rem before scroll kicks in.
            <div className="flex flex-col max-h-[10rem] overflow-y-auto">
              {roms.map((entry) => (
                <MenuItem
                  key={entry.id}
                  icon={<DatabaseIcon size={13} />}
                  label={entry.rom_name}
                  sublabel={`${entry.rom_hash.slice(0, 12)}… · ${formatBytes(entry.rom_size)}${
                    entry.header ? ` · ${HEADER_DISPLAY[entry.header]}` : ''
                  }`}
                  onClick={() => {
                    close();
                    onPickLibrary(entry);
                  }}
                />
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

interface MenuItemProps {
  icon: React.ReactNode;
  label: string;
  sublabel?: string;
  onClick: () => void;
}

function MenuItem({ icon, label, sublabel, onClick }: MenuItemProps) {
  return (
    <button
      type="button"
      role="menuitem"
      onClick={onClick}
      className={cn(
        'flex items-center gap-2.5 px-2.5 py-2 rounded-md text-left',
        'hover:bg-bg-input/70 transition-colors',
        'focus-visible:outline-none focus-visible:bg-bg-input/70',
      )}
    >
      <span className="text-fg-subtle shrink-0">{icon}</span>
      <div className="flex-1 min-w-0">
        <div className="text-sm text-fg truncate" title={label}>
          {label}
        </div>
        {sublabel && (
          <div className="text-[10px] text-fg-subtle font-mono truncate">{sublabel}</div>
        )}
      </div>
    </button>
  );
}
