import { forwardRef } from 'react';
import type { InputHTMLAttributes } from 'react';
import { cn } from '../lib/cn';

type InputProps = InputHTMLAttributes<HTMLInputElement> & {
  monospace?: boolean;
};

export const Input = forwardRef<HTMLInputElement, InputProps>(function Input(
  { className, monospace, ...rest },
  ref,
) {
  return (
    <input
      ref={ref}
      className={cn(
        'w-full h-9 px-3 rounded-lg bg-bg-input border border-bg-border',
        'text-sm text-fg placeholder:text-fg-subtle',
        'outline-none transition-colors',
        'focus:border-accent/60 focus:ring-2 focus:ring-accent/20',
        'disabled:opacity-50 disabled:cursor-not-allowed',
        monospace && 'font-mono text-[12.5px]',
        className,
      )}
      {...rest}
    />
  );
});
