import { FORMAT_BADGE_TONE } from "../lib/formats";

interface FormatBadgeProps {
  format: string;
  className?: string;
}

export function FormatBadge({ format, className }: FormatBadgeProps) {
  const tone = FORMAT_BADGE_TONE[format] ?? "neutral";
  return (
    <span
      className={className ? `format-badge ${className}` : "format-badge"}
      data-cat={tone}
      aria-label={`Formato ${format}`}
    >
      {format}
    </span>
  );
}
