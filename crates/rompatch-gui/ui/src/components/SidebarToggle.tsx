import { cn } from '../lib/cn';
import { PanelLeftIcon } from '../lib/icons';

interface SidebarToggleProps {
  open: boolean;
  onToggle: () => void;
}

export function SidebarToggle({ open, onToggle }: SidebarToggleProps) {
  return (
    <button
      type="button"
      onClick={onToggle}
      aria-label={open ? 'Hide sidebar' : 'Show sidebar'}
      aria-pressed={open}
      className={cn(
        'fixed top-2 left-[78px] z-30 inline-flex items-center justify-center',
        'h-7 w-7 rounded-full',
        'text-fg-subtle hover:text-fg hover:bg-bg-input/70 transition-colors',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40',
        open && 'text-fg-muted',
      )}
    >
      <PanelLeftIcon size={14} />
    </button>
  );
}
