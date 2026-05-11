import { useState } from 'react';
import { ApplyPanel } from './components/ApplyPanel';
import { HashPanel } from './components/HashPanel';
import { InspectPanel } from './components/InspectPanel';
import { Tab, TabList, TabPanel, Tabs } from './components/Tabs';
import { ToastProvider } from './components/Toast';

type TabId = 'apply' | 'inspect' | 'hash';

export function App() {
  const [tab, setTab] = useState<TabId>('apply');

  return (
    <ToastProvider>
      <div className="flex flex-col h-full bg-bg text-fg font-sans">
        <header className="px-5 pt-5 pb-2 flex items-baseline gap-2">
          <h1 className="text-lg font-semibold tracking-tight">rompatch</h1>
          <span className="text-xs text-fg-muted">
            ROM patcher
          </span>
        </header>
        <Tabs value={tab} onChange={(v) => setTab(v as TabId)}>
          <TabList>
            <Tab value="apply">Apply</Tab>
            <Tab value="inspect">Inspect</Tab>
            <Tab value="hash">Hash</Tab>
          </TabList>
          <TabPanel value="apply">
            <ApplyPanel />
          </TabPanel>
          <TabPanel value="inspect">
            <InspectPanel />
          </TabPanel>
          <TabPanel value="hash">
            <HashPanel />
          </TabPanel>
        </Tabs>
      </div>
    </ToastProvider>
  );
}
