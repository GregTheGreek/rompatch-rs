import { useId, useState } from 'react';
import { useToast } from './Toast';
import { Button } from './Button';
import { Label } from './Label';
import { cn } from '../lib/cn';
import { FileIcon, SaveIcon } from '../lib/icons';
import { pickFile, pickSavePath } from '../lib/tauri';

interface FilePickerProps {
  label: string;
  value: string | null;
  onChange: (path: string | null) => void;
  mode?: 'open' | 'save';
  dialogTitle?: string;
  defaultSavePath?: string;
  placeholder?: string;
}

function basename(path: string): string {
  const i = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
  return i >= 0 ? path.slice(i + 1) : path;
}

export function FilePicker({
  label,
  value,
  onChange,
  mode = 'open',
  dialogTitle,
  defaultSavePath,
  placeholder = 'No file selected',
}: FilePickerProps) {
  const id = useId();
  const [busy, setBusy] = useState(false);
  const { toast } = useToast();

  async function handlePick() {
    setBusy(true);
    try {
      const path =
        mode === 'save'
          ? await pickSavePath(defaultSavePath, dialogTitle)
          : await pickFile(dialogTitle);
      if (path) onChange(path);
    } catch (err) {
      toast({
        title: 'Failed to open file dialog',
        description: String(err),
        variant: 'error',
      });
    } finally {
      setBusy(false);
    }
  }

  const hasValue = value !== null && value !== '';

  return (
    <div className="flex flex-col gap-1.5">
      <Label htmlFor={id}>{label}</Label>
      <div
        id={id}
        className={cn(
          'flex items-center gap-2.5 rounded-lg border px-3 h-10',
          'bg-bg-input transition-colors',
          hasValue ? 'border-bg-border' : 'border-bg-border/60',
        )}
      >
        <FileIcon size={14} className="text-fg-subtle shrink-0" />
        <div
          className={cn(
            'flex-1 min-w-0 text-sm truncate',
            hasValue ? 'text-fg font-mono text-[12.5px]' : 'text-fg-subtle',
          )}
          title={value ?? undefined}
          data-selectable
        >
          {hasValue ? basename(value) : placeholder}
        </div>
        <Button
          size="sm"
          variant="secondary"
          onClick={handlePick}
          loading={busy}
          leftIcon={mode === 'save' ? <SaveIcon size={12} /> : undefined}
        >
          {mode === 'save' ? 'Save as...' : 'Browse...'}
        </Button>
      </div>
      {hasValue && (
        <div
          className="text-[11px] text-fg-subtle font-mono truncate"
          title={value}
          data-selectable
        >
          {value}
        </div>
      )}
    </div>
  );
}
