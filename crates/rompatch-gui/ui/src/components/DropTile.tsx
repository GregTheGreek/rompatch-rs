import { useState } from 'react';
import type { ReactNode } from 'react';
import { cn } from '../lib/cn';
import { XIcon } from '../lib/icons';
import { pickFile } from '../lib/tauri';
import { useDropTarget } from '../lib/useDropTarget';
import { useToast } from './Toast';

interface DropTileProps {
  label: string;
  filledLabel?: string;
  icon: ReactNode;
  value: string | null;
  onChange: (path: string | null) => void;
  dialogTitle?: string;
  badge?: ReactNode;
}

function basename(path: string): string {
  const i = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
  return i >= 0 ? path.slice(i + 1) : path;
}

export function DropTile({
  label,
  filledLabel,
  icon,
  value,
  onChange,
  dialogTitle,
  badge,
}: DropTileProps) {
  const { toast } = useToast();
  const [hovering, setHovering] = useState(false);
  const dropRef = useDropTarget<HTMLDivElement>((path) => onChange(path), setHovering);
  const filled = value !== null && value !== '';

  async function handlePick() {
    try {
      const picked = await pickFile(dialogTitle ?? `Select ${label.toLowerCase()}`);
      if (picked) onChange(picked);
    } catch (err) {
      toast({
        title: 'Failed to open file dialog',
        description: String(err),
        variant: 'error',
      });
    }
  }

  return (
    <div
      ref={dropRef}
      className="group flex-1 basis-0 flex flex-col items-center gap-5 min-w-0"
    >
      <div
        className={cn(
          'transition-all duration-200',
          'text-fg-subtle',
          filled && 'text-fg-muted',
          hovering && 'text-accent scale-110',
        )}
      >
        {icon}
      </div>

      <div className="relative max-w-full w-full flex justify-center">
        <button
          type="button"
          onClick={handlePick}
          className={cn(
            'inline-flex items-center justify-center gap-2 px-5 h-10 rounded-full',
            'text-sm font-medium select-none transition-all duration-150 max-w-full min-w-[10rem]',
            'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/50',
            !filled &&
              'bg-bg-raised text-fg-muted border border-bg-border hover:text-fg hover:border-fg-subtle/40',
            filled &&
              'bg-bg-raised text-fg border border-bg-border hover:border-fg-subtle/40',
            hovering && 'border-accent text-accent shadow-glow',
          )}
        >
          <span className="truncate">
            {filled ? basename(value!) : filledLabel ?? `Select ${label}`}
          </span>
          {filled && (
            <span
              role="button"
              tabIndex={0}
              aria-label={`Clear ${label.toLowerCase()}`}
              onClick={(e) => {
                e.stopPropagation();
                onChange(null);
              }}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  e.stopPropagation();
                  onChange(null);
                }
              }}
              className={cn(
                'inline-flex items-center justify-center h-5 w-5 -mr-2 rounded-full shrink-0',
                'text-fg-subtle hover:text-fg hover:bg-bg-input cursor-pointer transition-colors',
              )}
            >
              <XIcon size={11} />
            </span>
          )}
        </button>

        {filled && (
          <div
            role="tooltip"
            className={cn(
              'pointer-events-none absolute top-full left-1/2 -translate-x-1/2 mt-1.5',
              'px-2 py-1 rounded-md bg-bg-input border border-bg-border shadow-soft',
              'font-mono text-[11px] text-fg whitespace-nowrap',
              'opacity-0 group-hover:opacity-100 transition-opacity duration-150 z-30',
            )}
          >
            {basename(value!)}
          </div>
        )}
      </div>

      <div className="h-5 flex items-center">
        {filled && badge ? (
          badge
        ) : (
          <span className="text-[11px] uppercase tracking-wider font-mono text-fg-subtle">
            {label}
          </span>
        )}
      </div>
    </div>
  );
}
