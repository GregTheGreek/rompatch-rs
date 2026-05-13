import { useEffect, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { ApplyPanel } from './components/ApplyPanel';
import { LibraryPage } from './components/LibraryPage';
import { Sidebar } from './components/Sidebar';
import type { Page } from './components/Sidebar';
import { SidebarToggle } from './components/SidebarToggle';
import { ToastProvider } from './components/Toast';

// Wire up window dragging for elements marked with data-tauri-drag-region.
// Tauri ships a built-in handler, but it can miss elements that are nested
// or under stacking transforms; doing it explicitly is reliable.
function useWindowDrag() {
  useEffect(() => {
    const appWindow = getCurrentWindow();

    function onMouseDown(e: MouseEvent) {
      if (e.button !== 0) return;
      const target = e.target as HTMLElement | null;
      if (!target) return;
      const dragEl = target.closest('[data-tauri-drag-region]');
      if (!dragEl) return;
      const blocker = target.closest(
        'button, a, input, textarea, select, [role="button"], [role="switch"]',
      );
      if (blocker && dragEl.contains(blocker)) return;
      e.preventDefault();
      if (e.detail === 2) {
        void appWindow.toggleMaximize();
      } else {
        void appWindow.startDragging();
      }
    }

    document.addEventListener('mousedown', onMouseDown);
    return () => document.removeEventListener('mousedown', onMouseDown);
  }, []);
}

export function App() {
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [page, setPage] = useState<Page>('library');

  useWindowDrag();

  return (
    <ToastProvider>
      <div className="flex h-full bg-bg text-fg font-sans overflow-hidden relative">
        <SidebarToggle
          open={sidebarOpen}
          onToggle={() => setSidebarOpen((v) => !v)}
        />
        <Sidebar
          open={sidebarOpen}
          currentPage={page}
          onPageChange={setPage}
        />
        <main className="flex-1 flex flex-col overflow-hidden relative">
          <div data-tauri-drag-region className="h-12 shrink-0" />
          {page === 'library' ? <LibraryPage /> : <ApplyPanel />}
        </main>
      </div>
    </ToastProvider>
  );
}
