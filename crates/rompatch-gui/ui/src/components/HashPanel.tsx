import { useEffect, useState } from 'react';
import { Button } from './Button';
import { Card } from './Card';
import { FilePicker } from './FilePicker';
import { useToast } from './Toast';
import { cn } from '../lib/cn';
import { CheckIcon, CopyIcon } from '../lib/icons';
import { computeHashes } from '../lib/tauri';
import { HASH_ALGO_DISPLAY } from '../lib/types';
import type { HashAlgo, HashReport } from '../lib/types';

const ROWS: Array<{ algo: HashAlgo; key: keyof HashReport }> = [
  { algo: 'Crc32', key: 'crc32' },
  { algo: 'Md5', key: 'md5' },
  { algo: 'Sha1', key: 'sha1' },
  { algo: 'Adler32', key: 'adler32' },
];

export function HashPanel() {
  const { toast } = useToast();
  const [filePath, setFilePath] = useState<string | null>(null);
  const [report, setReport] = useState<HashReport | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!filePath) {
      setReport(null);
      return;
    }
    setLoading(true);
    computeHashes(filePath)
      .then(setReport)
      .catch((err) => {
        setReport(null);
        toast({
          title: 'Could not hash file',
          description: String(err && typeof err === 'object' && 'message' in err
            ? (err as { message: string }).message
            : err),
          variant: 'error',
        });
      })
      .finally(() => setLoading(false));
  }, [filePath, toast]);

  return (
    <div className="flex flex-col gap-4 max-w-2xl mx-auto">
      <Card title="File">
        <FilePicker
          label="Any file"
          value={filePath}
          onChange={setFilePath}
          dialogTitle="Select file to hash"
        />
      </Card>

      {report && (
        <Card
          title="Hashes"
          description={`${report.file_size.toLocaleString()} bytes`}
        >
          <div className="flex flex-col divide-y divide-bg-border/60">
            {ROWS.map(({ algo, key }) => (
              <HashRow
                key={algo}
                label={HASH_ALGO_DISPLAY[algo]}
                value={report[key] as string}
              />
            ))}
          </div>
        </Card>
      )}

      {filePath && !report && !loading && (
        <p className="text-sm text-fg-muted text-center">No hashes loaded.</p>
      )}
    </div>
  );
}

function HashRow({ label, value }: { label: string; value: string }) {
  const [copied, setCopied] = useState(false);
  const { toast } = useToast();

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(value);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1500);
    } catch (err) {
      toast({
        title: 'Copy failed',
        description: String(err),
        variant: 'error',
      });
    }
  }

  return (
    <div className="flex items-center gap-3 py-2.5 first:pt-0 last:pb-0">
      <div className="w-20 shrink-0 text-xs font-medium text-fg-muted uppercase tracking-wider">
        {label}
      </div>
      <code
        className="flex-1 min-w-0 truncate font-mono text-[12.5px] text-fg"
        title={value}
        data-selectable
      >
        {value}
      </code>
      <Button
        size="sm"
        variant="ghost"
        onClick={handleCopy}
        leftIcon={
          copied ? (
            <CheckIcon size={12} className={cn('text-success')} />
          ) : (
            <CopyIcon size={12} />
          )
        }
      >
        {copied ? 'Copied' : 'Copy'}
      </Button>
    </div>
  );
}
