import { forwardRef } from 'react';
import type { SelectHTMLAttributes } from 'react';
import { cn } from '../lib/cn';
import { ChevronDownIcon } from '../lib/icons';

type SelectProps = SelectHTMLAttributes<HTMLSelectElement>;

// Native <select> styled to match the rest of the UI. We give up some
// keyboard polish (vs. a custom listbox) but avoid the entire Radix
// dependency tree.
export const Select = forwardRef<HTMLSelectElement, SelectProps>(function Select(
  { className, children, ...rest },
  ref,
) {
  return (
    <div className="relative">
      <select
        ref={ref}
        className={cn(
          'w-full h-9 pl-3 pr-8 rounded-lg bg-bg-input border border-bg-border',
          'text-sm text-fg appearance-none cursor-pointer',
          'outline-none transition-colors',
          'focus:border-accent/60 focus:ring-2 focus:ring-accent/20',
          'disabled:opacity-50 disabled:cursor-not-allowed',
          className,
        )}
        {...rest}
      >
        {children}
      </select>
      <ChevronDownIcon
        size={14}
        className="absolute right-2.5 top-1/2 -translate-y-1/2 text-fg-muted pointer-events-none"
      />
    </div>
  );
});
