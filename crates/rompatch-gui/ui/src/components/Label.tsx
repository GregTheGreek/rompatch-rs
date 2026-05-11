import type { LabelHTMLAttributes } from 'react';
import { cn } from '../lib/cn';

export function Label({
  className,
  children,
  ...rest
}: LabelHTMLAttributes<HTMLLabelElement>) {
  return (
    <label
      className={cn(
        'text-xs font-medium text-fg-muted uppercase tracking-wider',
        className,
      )}
      {...rest}
    >
      {children}
    </label>
  );
}
