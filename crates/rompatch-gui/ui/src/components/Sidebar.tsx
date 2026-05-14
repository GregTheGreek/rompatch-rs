import { useEffect, useState } from 'react';
import type { ReactNode } from 'react';
import { getVersion } from '@tauri-apps/api/app';
import { cn } from '../lib/cn';
import { DatabaseIcon, PackageIcon } from '../lib/icons';

export type Page = 'library' | 'patch';

function useAppVersion(): string | null {
  const [version, setVersion] = useState<string | null>(null);
  useEffect(() => {
    let cancelled = false;
    getVersion()
      .then((v) => {
        if (!cancelled) setVersion(v);
      })
      .catch(() => {
        // Tauri API can fail in non-Tauri contexts (e.g. browser dev); stay
        // silent and just don't render the footer.
      });
    return () => {
      cancelled = true;
    };
  }, []);
  return version;
}

interface SidebarProps {
  open: boolean;
  currentPage: Page;
  onPageChange: (page: Page) => void;
}

export function Sidebar({ open, currentPage, onPageChange }: SidebarProps) {
  const version = useAppVersion();
  return (
    <aside
      data-tauri-drag-region
      aria-label="Primary navigation"
      className={cn(
        'shrink-0 h-full bg-bg-raised border-r border-bg-border',
        'flex flex-col overflow-hidden',
        'transition-[width,transform] duration-200 ease-out',
        open ? 'w-[14rem] translate-x-0' : 'w-0 -translate-x-full',
      )}
    >
      <div
        data-tauri-drag-region
        className="pt-12 pb-5 px-5 select-none shrink-0"
      >
        <div
          data-tauri-drag-region
          className="text-sm font-semibold text-fg tracking-tight"
        >
          Rom Library
        </div>
        <div
          data-tauri-drag-region
          className="text-[11px] uppercase tracking-wider font-mono text-fg-subtle"
        >
          ROM patcher
        </div>
      </div>

      <nav className="flex flex-col gap-0.5 px-2">
        <NavItem
          icon={<DatabaseIcon size={15} />}
          label="Library"
          active={currentPage === 'library'}
          onClick={() => onPageChange('library')}
        />
        <NavItem
          icon={<PackageIcon size={15} />}
          label="Patch"
          active={currentPage === 'patch'}
          onClick={() => onPageChange('patch')}
        />
      </nav>

      {version && (
        <div
          data-tauri-drag-region
          className="mt-auto px-5 py-3 text-[11px] font-mono text-fg-subtle select-none"
        >
          v{version}
        </div>
      )}
    </aside>
  );
}

interface NavItemProps {
  icon: ReactNode;
  label: string;
  active: boolean;
  onClick: () => void;
}

function NavItem({ icon, label, active, onClick }: NavItemProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-current={active ? 'page' : undefined}
      className={cn(
        'flex items-center gap-2.5 h-8 px-3 rounded-md text-sm text-left select-none',
        'transition-colors duration-100',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40',
        active
          ? 'bg-bg-input/70 text-fg'
          : 'text-fg-muted hover:text-fg hover:bg-bg-input/30',
      )}
    >
      <span className={cn('shrink-0', active ? 'text-fg' : 'text-fg-subtle')}>
        {icon}
      </span>
      <span className="truncate">{label}</span>
    </button>
  );
}
