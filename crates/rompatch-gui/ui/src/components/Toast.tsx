import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from 'react';
import type { ReactNode } from 'react';
import { cn } from '../lib/cn';
import { AlertIcon, CheckIcon, XIcon } from '../lib/icons';

type ToastVariant = 'success' | 'error' | 'warning' | 'info';

interface ToastInput {
  title: string;
  description?: string;
  variant?: ToastVariant;
  durationMs?: number;
}

interface ToastEntry extends Required<Pick<ToastInput, 'title' | 'variant' | 'durationMs'>> {
  id: number;
  description: string | undefined;
}

interface ToastContextValue {
  toast: (input: ToastInput) => void;
}

const ToastContext = createContext<ToastContextValue | null>(null);

export function useToast(): ToastContextValue {
  const ctx = useContext(ToastContext);
  if (!ctx) throw new Error('useToast must be inside <ToastProvider>');
  return ctx;
}

const VARIANT_STYLES: Record<ToastVariant, string> = {
  success: 'border-success/30 bg-success/10 text-fg',
  error: 'border-danger/40 bg-danger/10 text-fg',
  warning: 'border-warning/30 bg-warning/10 text-fg',
  info: 'border-accent/30 bg-accent-subtle/30 text-fg',
};

const VARIANT_ICON: Record<ToastVariant, ReactNode> = {
  success: <CheckIcon size={16} className="text-success" />,
  error: <AlertIcon size={16} className="text-danger" />,
  warning: <AlertIcon size={16} className="text-warning" />,
  info: <AlertIcon size={16} className="text-accent" />,
};

export function ToastProvider({ children }: { children: ReactNode }) {
  const [entries, setEntries] = useState<ToastEntry[]>([]);
  const nextId = useRef(0);

  const dismiss = useCallback((id: number) => {
    setEntries((current) => current.filter((e) => e.id !== id));
  }, []);

  const toast = useCallback((input: ToastInput) => {
    const id = nextId.current++;
    const entry: ToastEntry = {
      id,
      title: input.title,
      description: input.description,
      variant: input.variant ?? 'info',
      durationMs: input.durationMs ?? 4000,
    };
    setEntries((current) => [...current, entry]);
  }, []);

  return (
    <ToastContext.Provider value={{ toast }}>
      {children}
      <div className="pointer-events-none fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-sm">
        {entries.map((entry) => (
          <ToastItem key={entry.id} entry={entry} onDismiss={dismiss} />
        ))}
      </div>
    </ToastContext.Provider>
  );
}

function ToastItem({
  entry,
  onDismiss,
}: {
  entry: ToastEntry;
  onDismiss: (id: number) => void;
}) {
  useEffect(() => {
    const timer = window.setTimeout(() => onDismiss(entry.id), entry.durationMs);
    return () => window.clearTimeout(timer);
  }, [entry.id, entry.durationMs, onDismiss]);

  return (
    <div
      role="status"
      className={cn(
        'pointer-events-auto rounded-xl border px-4 py-3 shadow-soft',
        'animate-slide-up',
        VARIANT_STYLES[entry.variant],
      )}
    >
      <div className="flex items-start gap-3">
        <div className="mt-0.5">{VARIANT_ICON[entry.variant]}</div>
        <div className="flex-1 min-w-0">
          <div className="text-sm font-semibold">{entry.title}</div>
          {entry.description && (
            <div className="text-xs text-fg-muted mt-0.5 break-words" data-selectable>
              {entry.description}
            </div>
          )}
        </div>
        <button
          type="button"
          onClick={() => onDismiss(entry.id)}
          className="text-fg-muted hover:text-fg transition-colors p-0.5 rounded"
          aria-label="Dismiss"
        >
          <XIcon size={14} />
        </button>
      </div>
    </div>
  );
}
