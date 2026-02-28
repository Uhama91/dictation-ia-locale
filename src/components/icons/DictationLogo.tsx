interface DictationLogoProps {
  width?: number | string;
  className?: string;
}

export default function DictationLogo({ width = 120, className = "" }: DictationLogoProps) {
  return (
    <div
      className={`flex flex-col items-center gap-1 select-none ${className}`}
      style={{ width }}
    >
      {/* Icone micro — encre verte */}
      <svg
        viewBox="0 0 48 48"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        style={{ width: 36, height: 36 }}
        aria-hidden="true"
      >
        <rect x="18" y="4" width="12" height="22" rx="6" fill="var(--color-primary)" />
        <path
          d="M10 24c0 7.732 6.268 14 14 14s14-6.268 14-14"
          stroke="var(--color-text)"
          strokeWidth="3"
          strokeLinecap="round"
          fill="none"
        />
        <line
          x1="24" y1="38" x2="24" y2="44"
          stroke="var(--color-text)"
          strokeWidth="3"
          strokeLinecap="round"
        />
        <line
          x1="16" y1="44" x2="32" y2="44"
          stroke="var(--color-text)"
          strokeWidth="3"
          strokeLinecap="round"
        />
      </svg>

      {/* Branding — police Caveat */}
      <div className="flex flex-col items-center leading-tight">
        <span
          className="font-brand text-xl font-bold tracking-wide"
          style={{ color: "var(--color-primary)" }}
        >
          DictAI
        </span>
        <span className="text-[10px] font-medium opacity-50">
          Dictation IA Locale
        </span>
      </div>
    </div>
  );
}
