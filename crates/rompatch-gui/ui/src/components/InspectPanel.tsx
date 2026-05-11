import { useEffect, useState } from 'react';
import { Badge } from './Badge';
import { Card } from './Card';
import { FilePicker } from './FilePicker';
import { useToast } from './Toast';
import { describePatch } from '../lib/tauri';
import { FORMAT_DISPLAY } from '../lib/types';
import type { PatchInfo } from '../lib/types';

export function InspectPanel() {
  const { toast } = useToast();
  const [patchPath, setPatchPath] = useState<string | null>(null);
  const [info, setInfo] = useState<PatchInfo | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!patchPath) {
      setInfo(null);
      return;
    }
    setLoading(true);
    describePatch(patchPath)
      .then(setInfo)
      .catch((err) => {
        setInfo(null);
        toast({
          title: 'Could not read patch',
          description: String(err && typeof err === 'object' && 'message' in err
            ? (err as { message: string }).message
            : err),
          variant: 'error',
        });
      })
      .finally(() => setLoading(false));
  }, [patchPath, toast]);

  return (
    <div className="flex flex-col gap-4 max-w-2xl mx-auto">
      <Card title="Patch">
        <FilePicker
          label="File"
          value={patchPath}
          onChange={setPatchPath}
          dialogTitle="Select patch to inspect"
        />
      </Card>

      {info && (
        <Card
          title={
            <span className="flex items-center gap-2">
              <span>Metadata</span>
              <Badge tone="accent">{FORMAT_DISPLAY[info.format]}</Badge>
            </span>
          }
          description={`${info.patch_size.toLocaleString()} bytes`}
        >
          {info.fields.length === 0 ? (
            <p className="text-sm text-fg-muted">
              This format exposes no header metadata.
            </p>
          ) : (
            <dl className="grid grid-cols-[max-content_1fr] gap-x-6 gap-y-2 text-sm">
              {info.fields.map(([key, value]) => (
                <div key={key} className="contents">
                  <dt className="text-fg-muted">{key}</dt>
                  <dd
                    className="text-fg font-mono text-[12.5px] truncate"
                    title={value}
                    data-selectable
                  >
                    {value}
                  </dd>
                </div>
              ))}
            </dl>
          )}
        </Card>
      )}

      {patchPath && !info && !loading && (
        <p className="text-sm text-fg-muted text-center">No metadata loaded.</p>
      )}
    </div>
  );
}
