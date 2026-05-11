// Hand-rolled inline SVG icons. We pull a handful of glyphs from the
// lucide.dev set (ISC licensed) and inline them so we don't need an
// icon library dep. Each icon takes the standard `size` and `className`
// props and inherits stroke colour from `currentColor`.

interface IconProps {
  size?: number;
  className?: string;
  strokeWidth?: number;
}

function svg({ size = 16, className, strokeWidth = 2 }: IconProps, children: React.ReactNode) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={strokeWidth}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden="true"
    >
      {children}
    </svg>
  );
}

export const FileIcon = (p: IconProps) =>
  svg(p, (
    <>
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
      <path d="M14 2v6h6" />
    </>
  ));

export const UploadIcon = (p: IconProps) =>
  svg(p, (
    <>
      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
      <polyline points="17 8 12 3 7 8" />
      <line x1="12" y1="3" x2="12" y2="15" />
    </>
  ));

export const SaveIcon = (p: IconProps) =>
  svg(p, (
    <>
      <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
      <polyline points="17 21 17 13 7 13 7 21" />
      <polyline points="7 3 7 8 15 8" />
    </>
  ));

export const CopyIcon = (p: IconProps) =>
  svg(p, (
    <>
      <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
    </>
  ));

export const CheckIcon = (p: IconProps) =>
  svg(p, <polyline points="20 6 9 17 4 12" />);

export const XIcon = (p: IconProps) =>
  svg(p, (
    <>
      <line x1="18" y1="6" x2="6" y2="18" />
      <line x1="6" y1="6" x2="18" y2="18" />
    </>
  ));

export const AlertIcon = (p: IconProps) =>
  svg(p, (
    <>
      <circle cx="12" cy="12" r="10" />
      <line x1="12" y1="8" x2="12" y2="12" />
      <line x1="12" y1="16" x2="12.01" y2="16" />
    </>
  ));

export const PlayIcon = (p: IconProps) =>
  svg(p, <polygon points="5 3 19 12 5 21 5 3" />);

export const ChevronDownIcon = (p: IconProps) =>
  svg(p, <polyline points="6 9 12 15 18 9" />);
