import { useEffect, useState } from 'react';
import { Badge } from './Badge';
import { Button } from './Button';
import { Card } from './Card';
import { FilePicker } from './FilePicker';
import { Input } from './Input';
import { Label } from './Label';
import { Select } from './Select';
import { Switch } from './Switch';
import { useToast } from './Toast';
import { cn } from '../lib/cn';
import { PlayIcon, ChevronDownIcon } from '../lib/icons';
import {
  applyPatch,
  defaultOutputPath,
  detectPatchFormat,
  detectRomHeader,
} from '../lib/tauri';
import {
  FORMAT_DISPLAY,
  HEADER_DISPLAY,
  HASH_ALGO_DISPLAY,
  CHECKSUM_FAMILY_DISPLAY,
} from '../lib/types';
import type {
  ApplyOptions,
  ApplyReport,
  FormatKind,
  HashAlgo,
  HashSpec,
  HeaderKind,
} from '../lib/types';

const ALL_FORMATS: FormatKind[] = [
  'Ips',
  'Ups',
  'Bps',
  'Pmsr',
  'ApsGba',
  'ApsN64',
  'Ppf',
  'Rup',
  'Bdf',
];

const ALL_ALGOS: HashAlgo[] = ['Crc32', 'Md5', 'Sha1', 'Adler32'];

export function ApplyPanel() {
  const { toast } = useToast();

  const [romPath, setRomPath] = useState<string | null>(null);
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

  // Auto-detect format on patch selection.
  useEffect(() => {
    if (!patchPath) {
      setDetectedFormat(null);
      return;
    }
    detectPatchFormat(patchPath)
      .then(setDetectedFormat)
      .catch((err) => {
        toast({
          title: 'Could not read patch',
          description: String(err),
          variant: 'error',
        });
      });
  }, [patchPath, toast]);

  // Auto-detect header + suggested output path on ROM selection.
  useEffect(() => {
    if (!romPath) {
      setDetectedHeader(null);
      setOutPath(null);
      return;
    }
    detectRomHeader(romPath).then(setDetectedHeader).catch(() => {});
    defaultOutputPath(romPath).then(setOutPath).catch(() => {});
  }, [romPath]);

  const canApply = romPath !== null && patchPath !== null && outPath !== null && !running;

  function buildSpec(algo: HashAlgo, hex: string): HashSpec | null {
    const trimmed = hex.trim().toLowerCase();
    if (!trimmed) return null;
    return { algo, expected_hex: trimmed };
  }

  async function handleApply() {
    if (!canApply || !romPath || !patchPath || !outPath) return;
    setRunning(true);
    setLastReport(null);
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
    } catch (err) {
      const message = formatIpcError(err);
      toast({
        title: 'Apply failed',
        description: message,
        variant: 'error',
      });
    } finally {
      setRunning(false);
    }
  }

  return (
    <div className="flex flex-col gap-4 max-w-2xl mx-auto">
      <Card title="Input">
        <div className="flex flex-col gap-4">
          <FilePicker
            label="ROM"
            value={romPath}
            onChange={setRomPath}
            dialogTitle="Select ROM file"
          />
          <FilePicker
            label="Patch"
            value={patchPath}
            onChange={setPatchPath}
            dialogTitle="Select patch file"
          />
          {(detectedFormat || detectedHeader) && (
            <div className="flex items-center gap-2 text-xs text-fg-muted">
              {detectedFormat && (
                <Badge tone="accent">{FORMAT_DISPLAY[detectedFormat]}</Badge>
              )}
              {detectedHeader && (
                <Badge tone="neutral">{HEADER_DISPLAY[detectedHeader]} header</Badge>
              )}
            </div>
          )}
          <details className="group">
            <summary className="flex items-center gap-1.5 cursor-pointer text-xs text-fg-muted hover:text-fg select-none">
              <ChevronDownIcon
                size={12}
                className="transition-transform group-open:rotate-180"
              />
              Override detected format
            </summary>
            <div className="mt-2">
              <Select
                value={formatOverride ?? ''}
                onChange={(e) =>
                  setFormatOverride(e.target.value ? (e.target.value as FormatKind) : null)
                }
              >
                <option value="">(auto-detect)</option>
                {ALL_FORMATS.map((f) => (
                  <option key={f} value={f}>
                    {FORMAT_DISPLAY[f]}
                  </option>
                ))}
              </Select>
            </div>
          </details>
        </div>
      </Card>

      <Card title="Options">
        <div className="flex flex-col gap-3.5">
          <OptionRow
            id="strip-header"
            label="Strip ROM header"
            description="Detect SMC/iNES/FDS/LYNX, strip before patching, reattach to output."
            checked={stripHeader}
            onCheckedChange={setStripHeader}
          />
          <OptionRow
            id="fix-checksum"
            label="Fix cartridge checksum"
            description="Recompute Game Boy or Mega Drive header checksum on the output."
            checked={fixChecksum}
            onCheckedChange={setFixChecksum}
          />
          <OptionRow
            id="show-verify"
            label="Verify hashes"
            description="Reject mismatched input/output ROMs before writing the file."
            checked={showVerify}
            onCheckedChange={setShowVerify}
          />
          {showVerify && (
            <div className="grid grid-cols-1 gap-3 pl-4 border-l-2 border-accent/30 ml-1">
              <VerifyRow
                title="Input hash"
                algo={verifyInputAlgo}
                onAlgoChange={setVerifyInputAlgo}
                hex={verifyInputHex}
                onHexChange={setVerifyInputHex}
              />
              <VerifyRow
                title="Output hash"
                algo={verifyOutputAlgo}
                onAlgoChange={setVerifyOutputAlgo}
                hex={verifyOutputHex}
                onHexChange={setVerifyOutputHex}
              />
            </div>
          )}
        </div>
      </Card>

      <Card title="Output">
        <FilePicker
          label="Save to"
          value={outPath}
          onChange={setOutPath}
          mode="save"
          dialogTitle="Save patched ROM as"
          defaultSavePath={outPath ?? undefined}
        />
      </Card>

      <div className="flex items-center justify-end gap-3 pt-1">
        {lastReport && (
          <div className="text-xs text-fg-muted">
            Last: {FORMAT_DISPLAY[lastReport.format]} -{' '}
            {lastReport.out_size.toLocaleString()} bytes
            {lastReport.stripped_header
              ? ` - stripped ${HEADER_DISPLAY[lastReport.stripped_header]}`
              : ''}
            {lastReport.fixed_checksum
              ? ` - fixed ${CHECKSUM_FAMILY_DISPLAY[lastReport.fixed_checksum]}`
              : ''}
          </div>
        )}
        <Button
          variant="primary"
          size="lg"
          onClick={handleApply}
          disabled={!canApply}
          loading={running}
          leftIcon={<PlayIcon size={14} />}
        >
          {running ? 'Applying...' : 'Apply patch'}
        </Button>
      </div>
    </div>
  );
}

interface OptionRowProps {
  id: string;
  label: string;
  description: string;
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
}

function OptionRow({ id, label, description, checked, onCheckedChange }: OptionRowProps) {
  return (
    <div className={cn('flex items-start justify-between gap-4')}>
      <div className="flex-1 min-w-0">
        <label
          htmlFor={id}
          className="block text-sm text-fg cursor-pointer select-none"
        >
          {label}
        </label>
        <p className="text-xs text-fg-muted mt-0.5">{description}</p>
      </div>
      <Switch id={id} checked={checked} onCheckedChange={onCheckedChange} />
    </div>
  );
}

interface VerifyRowProps {
  title: string;
  algo: HashAlgo;
  onAlgoChange: (algo: HashAlgo) => void;
  hex: string;
  onHexChange: (hex: string) => void;
}

function VerifyRow({ title, algo, onAlgoChange, hex, onHexChange }: VerifyRowProps) {
  return (
    <div className="flex flex-col gap-1.5">
      <Label>{title}</Label>
      <div className="flex gap-2">
        <div className="w-32 shrink-0">
          <Select value={algo} onChange={(e) => onAlgoChange(e.target.value as HashAlgo)}>
            {ALL_ALGOS.map((a) => (
              <option key={a} value={a}>
                {HASH_ALGO_DISPLAY[a]}
              </option>
            ))}
          </Select>
        </div>
        <Input
          value={hex}
          onChange={(e) => onHexChange(e.target.value)}
          placeholder="expected hex digest..."
          monospace
          spellCheck={false}
        />
      </div>
    </div>
  );
}

function formatIpcError(err: unknown): string {
  if (err && typeof err === 'object' && 'message' in err) {
    const msg = (err as { message: unknown }).message;
    if (typeof msg === 'string') return msg;
  }
  return String(err);
}
