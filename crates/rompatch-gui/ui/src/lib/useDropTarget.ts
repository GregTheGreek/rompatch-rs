import { useEffect, useRef } from 'react';
import { getCurrentWebview } from '@tauri-apps/api/webview';

// Tauri 2's webview drag-drop events arrive at the window, not at any DOM
// element - the OS reports physical-pixel coordinates and the file paths.
// To dispatch them per-tile we register a callback against a ref and
// hit-test using getBoundingClientRect() on each event.

export interface DropTarget {
  ref: React.RefObject<HTMLElement>;
  onDrop: (paths: string[]) => void;
  onHoverChange?: (hovered: boolean) => void;
}

interface Registry {
  targets: Set<DropTarget>;
  currentHover: DropTarget | null;
}

const registry: Registry = { targets: new Set(), currentHover: null };

let subscribed = false;
let unlistenPromise: Promise<() => void> | null = null;

function hitTest(x: number, y: number): DropTarget | null {
  // Tauri reports physical pixels; CSS pixels = physical / devicePixelRatio.
  const dpr = window.devicePixelRatio || 1;
  const lx = x / dpr;
  const ly = y / dpr;
  for (const t of registry.targets) {
    const el = t.ref.current;
    if (!el) continue;
    const r = el.getBoundingClientRect();
    if (lx >= r.left && lx <= r.right && ly >= r.top && ly <= r.bottom) {
      return t;
    }
  }
  return null;
}

function setHover(next: DropTarget | null) {
  if (registry.currentHover === next) return;
  registry.currentHover?.onHoverChange?.(false);
  next?.onHoverChange?.(true);
  registry.currentHover = next;
}

function ensureSubscribed() {
  if (subscribed) return;
  subscribed = true;
  unlistenPromise = getCurrentWebview().onDragDropEvent((event) => {
    const p = event.payload;
    if (p.type === 'leave') {
      setHover(null);
      return;
    }
    if (p.type === 'enter' || p.type === 'over') {
      setHover(hitTest(p.position.x, p.position.y));
      return;
    }
    if (p.type === 'drop') {
      const target = hitTest(p.position.x, p.position.y);
      setHover(null);
      if (target && p.paths.length > 0) {
        target.onDrop(p.paths);
      }
    }
  });
}

/**
 * Register a DOM element as a file-drop target. `onDrop` receives every
 * dropped path; consumers that only care about a single file should pick
 * `paths[0]`. `onHoverChange` reflects whether the cursor (with a file
 * being dragged) is currently over this target.
 */
export function useDropTarget<T extends HTMLElement = HTMLDivElement>(
  onDrop: (paths: string[]) => void,
  onHoverChange?: (hovered: boolean) => void,
): React.RefObject<T> {
  const ref = useRef<T>(null);

  useEffect(() => {
    const target: DropTarget = {
      ref: ref as React.RefObject<HTMLElement>,
      onDrop,
      onHoverChange,
    };
    registry.targets.add(target);
    ensureSubscribed();
    return () => {
      if (registry.currentHover === target) {
        target.onHoverChange?.(false);
        registry.currentHover = null;
      }
      registry.targets.delete(target);
    };
  }, [onDrop, onHoverChange]);

  return ref;
}

// Exposed for completeness; the subscription leaks one listener for the
// lifetime of the app, which is fine in a single-window utility.
export async function _teardownDropSubscription(): Promise<void> {
  if (!unlistenPromise) return;
  const unlisten = await unlistenPromise;
  unlisten();
  subscribed = false;
  unlistenPromise = null;
}
