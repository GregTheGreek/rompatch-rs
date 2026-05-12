import { Input } from './Input';
import { Label } from './Label';
import { Select } from './Select';
import { Switch } from './Switch';
import { cn } from '../lib/cn';
import {
  FORMAT_DISPLAY,
  HASH_ALGO_DISPLAY,
} from '../lib/types';
import type { FormatKind, HashAlgo } from '../lib/types';

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

export interface AdvancedSectionProps {
  formatOverride: FormatKind | null;
  onFormatOverrideChange: (v: FormatKind | null) => void;

  stripHeader: boolean;
  onStripHeaderChange: (v: boolean) => void;

  fixChecksum: boolean;
  onFixChecksumChange: (v: boolean) => void;

  showVerify: boolean;
  onShowVerifyChange: (v: boolean) => void;

  verifyInputAlgo: HashAlgo;
  onVerifyInputAlgoChange: (v: HashAlgo) => void;
  verifyInputHex: string;
  onVerifyInputHexChange: (v: string) => void;

  verifyOutputAlgo: HashAlgo;
  onVerifyOutputAlgoChange: (v: HashAlgo) => void;
  verifyOutputHex: string;
  onVerifyOutputHexChange: (v: string) => void;
}

export function AdvancedSection(props: AdvancedSectionProps) {
  return (
    <div className="h-full overflow-y-auto px-5 pt-12 pb-6 flex flex-col gap-5">
      <div className="text-[11px] uppercase tracking-wider font-mono text-fg-subtle">
        Advanced
      </div>

      <Row>
        <Label>Format override</Label>
        <Select
          value={props.formatOverride ?? ''}
          onChange={(e) =>
            props.onFormatOverrideChange(
              e.target.value ? (e.target.value as FormatKind) : null,
            )
          }
        >
          <option value="">Auto-detect</option>
          {ALL_FORMATS.map((f) => (
            <option key={f} value={f}>
              {FORMAT_DISPLAY[f]}
            </option>
          ))}
        </Select>
      </Row>

      <div className="h-px bg-bg-border/60" />

      <Toggle
        id="adv-strip-header"
        label="Strip ROM header"
        checked={props.stripHeader}
        onCheckedChange={props.onStripHeaderChange}
      />
      <Toggle
        id="adv-fix-checksum"
        label="Fix cartridge checksum"
        checked={props.fixChecksum}
        onCheckedChange={props.onFixChecksumChange}
      />
      <Toggle
        id="adv-show-verify"
        label="Verify hashes"
        checked={props.showVerify}
        onCheckedChange={props.onShowVerifyChange}
      />

      {props.showVerify && (
        <div className="flex flex-col gap-3 pl-3 border-l border-accent/30 -mt-1">
          <VerifyRow
            title="Input"
            algo={props.verifyInputAlgo}
            onAlgoChange={props.onVerifyInputAlgoChange}
            hex={props.verifyInputHex}
            onHexChange={props.onVerifyInputHexChange}
          />
          <VerifyRow
            title="Output"
            algo={props.verifyOutputAlgo}
            onAlgoChange={props.onVerifyOutputAlgoChange}
            hex={props.verifyOutputHex}
            onHexChange={props.onVerifyOutputHexChange}
          />
        </div>
      )}
    </div>
  );
}

function Row({ children }: { children: React.ReactNode }) {
  return <div className="flex flex-col gap-1.5">{children}</div>;
}

interface ToggleProps {
  id: string;
  label: string;
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
}

function Toggle({ id, label, checked, onCheckedChange }: ToggleProps) {
  return (
    <label
      htmlFor={id}
      className={cn(
        'flex items-center justify-between gap-3 cursor-pointer select-none',
        'text-sm text-fg-muted hover:text-fg transition-colors',
      )}
    >
      <span>{label}</span>
      <Switch id={id} checked={checked} onCheckedChange={onCheckedChange} />
    </label>
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
      <Select value={algo} onChange={(e) => onAlgoChange(e.target.value as HashAlgo)}>
        {ALL_ALGOS.map((a) => (
          <option key={a} value={a}>
            {HASH_ALGO_DISPLAY[a]}
          </option>
        ))}
      </Select>
      <Input
        value={hex}
        onChange={(e) => onHexChange(e.target.value)}
        placeholder="expected hex digest…"
        monospace
        spellCheck={false}
      />
    </div>
  );
}
