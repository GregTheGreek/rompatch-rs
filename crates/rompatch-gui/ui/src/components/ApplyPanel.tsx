import { useEffect, useRef, useState } from 'react';
import { desktopDir } from '@tauri-apps/api/path';
import { AdvancedSection } from './AdvancedSection';
import { ApplyTile } from './ApplyTile';
import type { ApplyState } from './ApplyTile';
import { Badge } from './Badge';
import { DropTile } from './DropTile';
import { RomSourceMenu } from './RomSourceMenu';
import { useToast } from './Toast';
import { cn } from '../lib/cn';
import { formatIpcError } from '../lib/errors';
import {
  FolderIcon,
  PackageIcon,
  PlusCircleIcon,
  SaveIcon,
  SettingsIcon,
} from '../lib/icons';
import {
  applyPatch,
  defaultOutputPath,
  detectPatchFormat,
  detectRomHeader,
  libraryRecord,
  libraryRomPath,
  pickDirectory,
  pickFile,
} from '../lib/tauri';
import {
  CHECKSUM_FAMILY_DISPLAY,
  FORMAT_DISPLAY,
  HEADER_DISPLAY,
} from '../lib/types';
import type {
  ApplyOptions,
  ApplyReport,
  FormatKind,
  HashAlgo,
  HashSpec,
  HeaderKind,
  LibraryRomEntry,
} from '../lib/types';

function basename(path: string): string {
  const i = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
  return i >= 0 ? path.slice(i + 1) : path;
}

function dirname(path: string): string {
  const i = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
  return i >= 0 ? path.slice(0, i) : '';
}

function joinPath(dir: string, name: string): string {
  if (!dir) return name;
  const sep = dir.includes('\\') && !dir.includes('/') ? '\\' : '/';
  return dir.endsWith(sep) ? dir + name : dir + sep + name;
}

// Mirror of `rompatch_core::apply::default_output_path` for the case where
// we only have a filename (not a full path). Preserves the original ROM's
// extension so the patched output stays emulator-playable.
function deriveOutputName(romName: string): string {
  const dot = romName.lastIndexOf('.');
  if (dot <= 0) return `${romName}.patched`;
  return `${romName.slice(0, dot)}.patched${romName.slice(dot)}`;
}

export function ApplyPanel() {
  const { toast } = useToast();

  const [romPath, setRomPath] = useState<string | null>(null);
  const [romDisplayName, setRomDisplayName] = useState<string | null>(null);
  const [patchPath, setPatchPath] = useState<string | null>(null);
  const [outPath, setOutPath] = useState<string | null>(null);

  const [detectedFormat, setDetectedFormat] = useState<FormatKind | null>(null);
  const [detectedHeader, setDetectedHeader] = useState<HeaderKind | null>(null);
  const [formatOverride, setFormatOverride] = useState<FormatKind | null>(null);

  const [stripHeader, setStripHeader] = useState(false);
  const [fixChecksum, setFixChecksum] = useState(false);

  const [showVerify, setShowVerify] = useState(false);
  const [verifyInputAlgo, setVerifyInputAlgo] = useState<HashAlgo>('Sha1');
  const [verifyInputHex, setVerifyInputHex] = useState('');
  const [verifyOutputAlgo, setVerifyOutputAlgo] = useState<HashAlgo>('Sha1');
  const [verifyOutputHex, setVerifyOutputHex] = useState('');

  const [running, setRunning] = useState(false);
  const [lastReport, setLastReport] = useState<ApplyReport | null>(null);
  const [lastError, setLastError] = useState<string | null>(null);
  const [advancedOpen, setAdvancedOpen] = useState(false);

  // Success state auto-reverts to ready after a few seconds.
  const successTimer = useRef<number | null>(null);
  useEffect(() => {
    return () => {
      if (successTimer.current !== null) window.clearTimeout(successTimer.current);
    };
  }, []);

  // Auto-detect format on patch selection. Cancellation flag prevents
  // a slow earlier read from clobbering state after the user swaps files.
  useEffect(() => {
    if (!patchPath) {
      setDetectedFormat(null);
      return;
    }
    let cancelled = false;
    detectPatchFormat(patchPath)
      .then((v) => {
        if (!cancelled) setDetectedFormat(v);
      })
      .catch((err) => {
        if (cancelled) return;
        toast({
          title: 'Could not read patch',
          description: formatIpcError(err),
          variant: 'error',
        });
      });
    return () => {
      cancelled = true;
    };
  }, [patchPath, toast]);

  // Auto-detect header + suggested output path on ROM selection.
  useEffect(() => {
    if (!romPath) {
      setDetectedHeader(null);
      setOutPath(null);
      return;
    }
    let cancelled = false;
    detectRomHeader(romPath)
      .then((v) => {
        if (!cancelled) setDetectedHeader(v);
      })
      .catch((err) => {
        if (cancelled) return;
        toast({
          title: 'Could not read ROM header',
          description: formatIpcError(err),
          variant: 'error',
        });
      });
    // Skip the default-output-path autosuggest when the ROM came from the
    // library: `romPath` is then `<library>/roms/<hash>.bin`, and the IPC
    // would suggest `<hash>.patched.bin` - opaque to emulators.
    // handleRomFromLibrary sets a proper playable path explicitly.
    if (romDisplayName === null) {
      defaultOutputPath(romPath)
        .then((v) => {
          if (!cancelled) setOutPath(v);
        })
        .catch((err) => {
          if (cancelled) return;
          toast({
            title: 'Could not compute output path',
            description: formatIpcError(err),
            variant: 'error',
          });
        });
    }
    return () => {
      cancelled = true;
    };
  }, [romPath, romDisplayName, toast]);

  const filesReady = romPath !== null && patchPath !== null && outPath !== null;

  let applyState: ApplyState;
  if (running) applyState = 'running';
  else if (lastError) applyState = 'error';
  else if (lastReport) applyState = 'success';
  else if (filesReady) applyState = 'ready';
  else applyState = 'empty';

  function buildSpec(algo: HashAlgo, hex: string): HashSpec | null {
    const trimmed = hex.trim().toLowerCase();
    if (!trimmed) return null;
    return { algo, expected_hex: trimmed };
  }

  async function handleApply() {
    // From success or error, a click clears the state and re-arms.
    if (applyState === 'success' || applyState === 'error') {
      if (successTimer.current !== null) {
        window.clearTimeout(successTimer.current);
        successTimer.current = null;
      }
      setLastReport(null);
      setLastError(null);
      return;
    }
    if (applyState !== 'ready' || !romPath || !patchPath || !outPath) return;

    setRunning(true);
    setLastReport(null);
    setLastError(null);
    const options: ApplyOptions = {
      strip_header: stripHeader,
      fix_checksum: fixChecksum,
      verify_input: showVerify ? buildSpec(verifyInputAlgo, verifyInputHex) : null,
      verify_output: showVerify ? buildSpec(verifyOutputAlgo, verifyOutputHex) : null,
      format_override: formatOverride,
    };
    try {
      const report = await applyPatch(romPath, patchPath, outPath, options);
      setLastReport(report);
      toast({
        title: 'Patch applied',
        description: `${FORMAT_DISPLAY[report.format]} — wrote ${report.out_size.toLocaleString()} bytes`,
        variant: 'success',
      });
      if (successTimer.current !== null) window.clearTimeout(successTimer.current);
      successTimer.current = window.setTimeout(() => {
        setLastReport(null);
      }, 4000);
      // Auto-import into the local library. Failure here must not turn the
      // apply into an error - the user's file is still on disk.
      try {
        await libraryRecord({
          source_path: romPath,
          patch_path: patchPath,
          output_path: outPath,
          format: report.format,
          header: report.stripped_header,
          fixed_checksum: report.fixed_checksum,
          apply_options: options,
        });
      } catch (libErr) {
        toast({
          title: 'Library import skipped',
          description: formatIpcError(libErr),
          variant: 'warning',
        });
      }
    } catch (err) {
      const message = formatIpcError(err);
      setLastError(message);
      toast({
        title: 'Apply failed',
        description: message,
        variant: 'error',
      });
    } finally {
      setRunning(false);
    }
  }

  // ROM came from a file picker or drag-drop: real on-disk path, no library
  // entry tied to it. Clear any prior library display name.
  function handleRomFromTile(path: string | null) {
    setRomPath(path);
    setRomDisplayName(null);
  }

  async function handleImportRomViaDialog() {
    try {
      const picked = await pickFile('Select ROM file');
      if (picked) handleRomFromTile(picked);
    } catch (err) {
      toast({
        title: 'Failed to open file dialog',
        description: String(err),
        variant: 'error',
      });
    }
  }

  // ROM came from the library picker: use the content-addressed path but
  // surface the friendly name in the pill / tooltip. Manually set outPath to
  // `<Desktop>/<rom_name>.patched.<ext>` so the patched output stays
  // emulator-playable - the auto-suggest would otherwise produce
  // `<hash>.patched.bin` from the content-addressed source path.
  async function handleRomFromLibrary(entry: LibraryRomEntry) {
    try {
      const [path, desktop] = await Promise.all([
        libraryRomPath(entry.rom_hash),
        desktopDir(),
      ]);
      setRomPath(path);
      setRomDisplayName(entry.rom_name);
      setOutPath(joinPath(desktop, deriveOutputName(entry.rom_name)));
    } catch (err) {
      toast({
        title: 'Could not load library ROM',
        description: formatIpcError(err),
        variant: 'error',
      });
    }
  }

  async function handleChangeLocation() {
    if (!outPath) return;
    try {
      const picked = await pickDirectory(dirname(outPath), 'Choose save folder');
      if (picked) setOutPath(joinPath(picked, basename(outPath)));
    } catch (err) {
      toast({
        title: 'Failed to open folder dialog',
        description: String(err),
        variant: 'error',
      });
    }
  }

  function handleRename(newName: string) {
    const trimmed = newName.trim();
    if (!outPath || !trimmed || trimmed === basename(outPath)) return;
    setOutPath(joinPath(dirname(outPath), trimmed));
  }

  const successHeadline = lastReport
    ? `Wrote ${lastReport.out_size.toLocaleString()} bytes`
    : undefined;

  const lastReportLine = lastReport
    ? [
        FORMAT_DISPLAY[lastReport.format],
        `${lastReport.out_size.toLocaleString()} bytes`,
        lastReport.stripped_header && `stripped ${HEADER_DISPLAY[lastReport.stripped_header]}`,
        lastReport.fixed_checksum && `fixed ${CHECKSUM_FAMILY_DISPLAY[lastReport.fixed_checksum]}`,
      ]
        .filter(Boolean)
        .join(' — ')
    : null;

  const drawerWidth = '18rem';

  return (
    <div className="flex-1 flex relative overflow-hidden">
      <button
        type="button"
        onClick={() => setAdvancedOpen((v) => !v)}
        aria-expanded={advancedOpen}
        aria-controls="advanced-section"
        aria-label={advancedOpen ? 'Hide advanced settings' : 'Show advanced settings'}
        className={cn(
          'fixed top-2 right-3 z-20 inline-flex items-center justify-center h-7 w-7 rounded-full',
          'text-fg-subtle hover:text-fg hover:bg-bg-input/70 transition-colors',
          'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40',
          advancedOpen && 'text-accent bg-accent-subtle/30',
        )}
      >
        <SettingsIcon size={14} />
      </button>

      <main
        className="flex-1 flex items-center justify-center transition-[padding] duration-200"
        style={{ paddingRight: advancedOpen ? drawerWidth : 0 }}
      >
        <div className="flex flex-col gap-8 max-w-2xl w-full px-6">
          <div className="flex items-start gap-2 w-full">
        <RomSourceMenu
          onImport={handleImportRomViaDialog}
          onPickLibrary={handleRomFromLibrary}
          trigger={({ onClick }) => (
            <DropTile
              label="ROM"
              filledLabel="Select ROM"
              icon={<PlusCircleIcon size={56} strokeWidth={1.5} />}
              value={romPath}
              displayName={romDisplayName}
              onChange={handleRomFromTile}
              onPick={onClick}
              dialogTitle="Select ROM file"
              badge={
                detectedHeader ? (
                  <Badge tone="neutral">{HEADER_DISPLAY[detectedHeader]} header</Badge>
                ) : null
              }
            />
          )}
        />
        <Connector lit={romPath !== null} />
        <DropTile
          label="Patch"
          filledLabel="Select patch"
          icon={<PackageIcon size={56} strokeWidth={1.5} />}
          value={patchPath}
          onChange={setPatchPath}
          dialogTitle="Select patch file"
          badge={
            detectedFormat ? (
              <Badge tone="accent">{FORMAT_DISPLAY[detectedFormat]}</Badge>
            ) : null
          }
        />
        <Connector lit={filesReady} />
        <ApplyTile
          state={applyState}
          onApply={handleApply}
          successMessage={successHeadline}
          errorMessage={lastError ?? undefined}
        />
      </div>

      <div className="flex items-center gap-2 px-1 text-xs text-fg-muted min-h-[1.5rem]">
        {outPath ? (
          <>
            <SaveIcon size={12} className="shrink-0 text-fg-subtle" />
            <span className="text-fg-subtle shrink-0">Save to</span>
            <EditableFilename
              filename={basename(outPath)}
              fullPath={outPath}
              onCommit={handleRename}
            />
            <button
              type="button"
              onClick={handleChangeLocation}
              className={cn(
                'ml-auto inline-flex items-center gap-1.5 h-6 px-2 rounded-md shrink-0',
                'text-fg-subtle hover:text-fg hover:bg-bg-input/60 transition-colors',
                'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40',
              )}
              title={dirname(outPath)}
            >
              <FolderIcon size={12} />
              <span>Change folder</span>
            </button>
          </>
        ) : (
          <span className="text-fg-subtle mx-auto">Pick a ROM to start.</span>
        )}
      </div>

          {lastReportLine && (
            <div className="px-1 text-[11px] text-fg-subtle font-mono truncate">
              {lastReportLine}
            </div>
          )}
        </div>
      </main>

      <aside
        id="advanced-section"
        aria-hidden={!advancedOpen}
        className={cn(
          'fixed top-0 right-0 bottom-0 border-l border-bg-border bg-bg-raised',
          'transition-transform duration-200 ease-out z-10',
          !advancedOpen && 'pointer-events-none',
        )}
        style={{
          width: drawerWidth,
          transform: advancedOpen ? 'translateX(0)' : 'translateX(100%)',
        }}
      >
        <AdvancedSection
          formatOverride={formatOverride}
          onFormatOverrideChange={setFormatOverride}
          stripHeader={stripHeader}
          onStripHeaderChange={setStripHeader}
          fixChecksum={fixChecksum}
          onFixChecksumChange={setFixChecksum}
          showVerify={showVerify}
          onShowVerifyChange={setShowVerify}
          verifyInputAlgo={verifyInputAlgo}
          onVerifyInputAlgoChange={setVerifyInputAlgo}
          verifyInputHex={verifyInputHex}
          onVerifyInputHexChange={setVerifyInputHex}
          verifyOutputAlgo={verifyOutputAlgo}
          onVerifyOutputAlgoChange={setVerifyOutputAlgo}
          verifyOutputHex={verifyOutputHex}
          onVerifyOutputHexChange={setVerifyOutputHex}
        />
      </aside>
    </div>
  );
}

interface EditableFilenameProps {
  filename: string;
  fullPath: string;
  onCommit: (newName: string) => void;
}

function EditableFilename({ filename, fullPath, onCommit }: EditableFilenameProps) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(filename);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!editing) setDraft(filename);
  }, [filename, editing]);

  useEffect(() => {
    if (editing && inputRef.current) {
      inputRef.current.focus();
      const dot = filename.lastIndexOf('.');
      inputRef.current.setSelectionRange(0, dot > 0 ? dot : filename.length);
    }
  }, [editing, filename]);

  function commit() {
    setEditing(false);
    onCommit(draft);
  }
  function cancel() {
    setDraft(filename);
    setEditing(false);
  }

  if (editing) {
    return (
      <input
        ref={inputRef}
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => {
          if (e.key === 'Enter') {
            e.preventDefault();
            commit();
          } else if (e.key === 'Escape') {
            e.preventDefault();
            cancel();
          }
        }}
        spellCheck={false}
        className={cn(
          'min-w-0 flex-1 bg-transparent font-mono text-[12px] text-fg outline-none',
          'border-b border-accent/60 focus:border-accent px-0.5',
        )}
      />
    );
  }

  return (
    <button
      type="button"
      onClick={() => setEditing(true)}
      title={fullPath}
      className={cn(
        'min-w-0 flex-1 text-left font-mono text-[12px] text-fg truncate',
        'hover:text-accent transition-colors',
        'focus-visible:outline-none focus-visible:text-accent',
      )}
    >
      {filename}
    </button>
  );
}

function Connector({ lit }: { lit: boolean }) {
  return (
    <div
      aria-hidden
      className={cn(
        'shrink-0 w-12 self-start mt-7 h-px transition-colors duration-200',
        lit ? 'bg-accent/55' : 'bg-bg-border',
      )}
    />
  );
}
