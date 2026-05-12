import { cn } from '../lib/cn';
import { AlertIcon, BoltIcon, CheckIcon } from '../lib/icons';

export type ApplyState = 'empty' | 'ready' | 'running' | 'success' | 'error';

interface ApplyTileProps {
  state: ApplyState;
  onApply: () => void;
  successMessage?: string;
  errorMessage?: string;
}

export function ApplyTile({
  state,
  onApply,
  successMessage,
  errorMessage,
}: ApplyTileProps) {
  const disabled = state === 'empty' || state === 'running';
  const Icon =
    state === 'success' ? CheckIcon : state === 'error' ? AlertIcon : BoltIcon;

  const buttonLabel =
    state === 'running'
      ? 'Applying…'
      : state === 'success'
        ? (successMessage ?? 'Done')
        : state === 'error'
          ? 'Retry'
          : 'Apply!';

  const captionLabel =
    state === 'empty'
      ? 'Apply'
      : state === 'running'
        ? 'Working'
        : state === 'success'
          ? 'Success'
          : state === 'error'
            ? (errorMessage ?? 'Failed')
            : 'Ready';

  return (
    <div className="group flex-1 basis-0 flex flex-col items-center gap-5 min-w-0">
      <div
        className={cn(
          'transition-all duration-200',
          state === 'empty' && 'text-fg-subtle',
          state === 'ready' && 'text-accent',
          state === 'running' && 'text-accent',
          state === 'success' && 'text-success',
          state === 'error' && 'text-danger',
        )}
      >
        <Icon
          size={56}
          strokeWidth={state === 'success' ? 2.5 : 2}
          className={cn(state === 'running' && 'animate-pulse')}
        />
      </div>

      <button
        type="button"
        onClick={onApply}
        disabled={disabled}
        aria-live="polite"
        className={cn(
          'inline-flex items-center justify-center px-5 h-10 rounded-full',
          'text-sm font-semibold select-none transition-all duration-150 max-w-full min-w-[10rem]',
          'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/50',
          'disabled:cursor-not-allowed',
          state === 'empty' &&
            'bg-bg-raised text-fg-subtle border border-bg-border opacity-60',
          state === 'ready' &&
            'bg-accent text-white hover:bg-accent-hover active:bg-accent-active shadow-glow',
          state === 'running' &&
            'bg-accent text-white animate-pulse-glow',
          state === 'success' &&
            'bg-success/15 text-success border border-success/30 hover:bg-success/20',
          state === 'error' &&
            'bg-danger/15 text-danger border border-danger/30 hover:bg-danger/20',
        )}
      >
        <span className="truncate">{buttonLabel}</span>
      </button>

      <div className="h-5 flex items-center">
        <span
          className={cn(
            'text-[11px] uppercase tracking-wider font-mono truncate max-w-full px-2',
            state === 'success' && 'text-success/80',
            state === 'error' && 'text-danger/80',
            state !== 'success' && state !== 'error' && 'text-fg-subtle',
          )}
          title={state === 'error' ? errorMessage ?? undefined : undefined}
        >
          {captionLabel}
        </span>
      </div>
    </div>
  );
}
