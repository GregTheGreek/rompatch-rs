// Tiny className joiner — a focused replacement for clsx/classnames that we
// don't want as a dependency. Falsy values are dropped; truthy values are
// joined with a single space.
export function cn(...parts: Array<string | false | null | undefined>): string {
  return parts.filter(Boolean).join(' ');
}
