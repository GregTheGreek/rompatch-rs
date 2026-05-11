import { createContext, useContext, useId } from 'react';
import type { ReactNode } from 'react';
import { cn } from '../lib/cn';

interface TabsContextValue {
  value: string;
  onChange: (value: string) => void;
  baseId: string;
}

const TabsContext = createContext<TabsContextValue | null>(null);

function useTabs() {
  const ctx = useContext(TabsContext);
  if (!ctx) throw new Error('Tab components must be inside <Tabs>');
  return ctx;
}

interface TabsProps {
  value: string;
  onChange: (value: string) => void;
  children: ReactNode;
}

export function Tabs({ value, onChange, children }: TabsProps) {
  const baseId = useId();
  return (
    <TabsContext.Provider value={{ value, onChange, baseId }}>
      <div className="flex flex-col h-full">{children}</div>
    </TabsContext.Provider>
  );
}

export function TabList({ children }: { children: ReactNode }) {
  return (
    <div
      role="tablist"
      className="flex items-center gap-1 px-4 pt-3 pb-0 border-b border-bg-border"
    >
      {children}
    </div>
  );
}

interface TabProps {
  value: string;
  children: ReactNode;
}

export function Tab({ value, children }: TabProps) {
  const { value: active, onChange, baseId } = useTabs();
  const selected = active === value;
  return (
    <button
      role="tab"
      type="button"
      aria-selected={selected}
      aria-controls={`${baseId}-panel-${value}`}
      id={`${baseId}-tab-${value}`}
      onClick={() => onChange(value)}
      className={cn(
        'px-3 py-2 text-sm rounded-t-md transition-colors outline-none',
        'focus-visible:ring-2 focus-visible:ring-accent/40',
        selected
          ? 'text-fg border-b-2 border-accent -mb-px'
          : 'text-fg-muted hover:text-fg border-b-2 border-transparent -mb-px',
      )}
    >
      {children}
    </button>
  );
}

interface TabPanelProps {
  value: string;
  children: ReactNode;
}

export function TabPanel({ value, children }: TabPanelProps) {
  const { value: active, baseId } = useTabs();
  if (active !== value) return null;
  return (
    <div
      role="tabpanel"
      id={`${baseId}-panel-${value}`}
      aria-labelledby={`${baseId}-tab-${value}`}
      className="flex-1 overflow-auto p-5 animate-fade-in"
    >
      {children}
    </div>
  );
}
