import { useEffect, useState } from 'react';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export type UpdateStatus =
  | { kind: 'idle' }
  | { kind: 'checking' }
  | { kind: 'available'; update: Update }
  | { kind: 'downloading'; downloaded: number; contentLength: number | null }
  | { kind: 'ready' }
  | { kind: 'error'; message: string };

export function useUpdater() {
  const [status, setStatus] = useState<UpdateStatus>({ kind: 'idle' });

  useEffect(() => {
    let cancelled = false;
    setStatus({ kind: 'checking' });
    check()
      .then((update) => {
        if (cancelled) return;
        if (update) {
          setStatus({ kind: 'available', update });
        } else {
          setStatus({ kind: 'idle' });
        }
      })
      .catch((err: unknown) => {
        if (cancelled) return;
        // Network failures, missing latest.json on first release, etc. — log
        // and stay silent so a broken updater never blocks app launch.
        console.warn('updater check failed:', err);
        setStatus({ kind: 'idle' });
      });
    return () => {
      cancelled = true;
    };
  }, []);

  async function install(update: Update) {
    let downloaded = 0;
    let contentLength: number | null = null;
    setStatus({ kind: 'downloading', downloaded, contentLength });
    try {
      await update.downloadAndInstall((event) => {
        if (event.event === 'Started') {
          contentLength = event.data.contentLength ?? null;
          setStatus({ kind: 'downloading', downloaded: 0, contentLength });
        } else if (event.event === 'Progress') {
          downloaded += event.data.chunkLength;
          setStatus({ kind: 'downloading', downloaded, contentLength });
        } else if (event.event === 'Finished') {
          setStatus({ kind: 'ready' });
        }
      });
      await relaunch();
    } catch (err) {
      console.error('updater install failed:', err);
      setStatus({
        kind: 'error',
        message: err instanceof Error ? err.message : String(err),
      });
    }
  }

  return { status, install };
}
