import type { HTMLAttributes, ReactNode } from 'react';
import { cn } from '../lib/cn';

interface CardProps extends Omit<HTMLAttributes<HTMLDivElement>, 'title'> {
  title?: ReactNode;
  description?: ReactNode;
  footer?: ReactNode;
}

export function Card({ title, description, footer, className, children, ...rest }: CardProps) {
  return (
    <div
      className={cn(
        'rounded-xl bg-bg-raised border border-bg-border shadow-soft',
        'animate-fade-in',
        className,
      )}
      {...rest}
    >
      {(title || description) && (
        <div className="px-5 pt-4 pb-3 border-b border-bg-border/60">
          {title && <div className="text-sm font-semibold text-fg">{title}</div>}
          {description && (
            <div className="text-xs text-fg-muted mt-0.5">{description}</div>
          )}
        </div>
      )}
      <div className="px-5 py-4">{children}</div>
      {footer && (
        <div className="px-5 py-3 border-t border-bg-border/60 bg-bg-input/30 rounded-b-xl">
          {footer}
        </div>
      )}
    </div>
  );
}
