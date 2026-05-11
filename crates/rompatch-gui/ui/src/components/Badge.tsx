import type { ReactNode } from 'react';
import { cn } from '../lib/cn';

type BadgeTone = 'neutral' | 'accent' | 'success' | 'warning';

interface BadgeProps {
  children: ReactNode;
  tone?: BadgeTone;
  className?: string;
}

const TONES: Record<BadgeTone, string> = {
  neutral: 'bg-bg-input text-fg-muted border-bg-border',
  accent: 'bg-accent-subtle/40 text-accent border-accent/30',
  success: 'bg-success/10 text-success border-success/30',
  warning: 'bg-warning/10 text-warning border-warning/30',
};

export function Badge({ children, tone = 'neutral', className }: BadgeProps) {
  return (
    <span
      className={cn(
        'inline-flex items-center h-5 px-2 rounded-full border text-[11px] font-medium font-mono uppercase tracking-wide',
        TONES[tone],
        className,
      )}
    >
      {children}
    </span>
  );
}
