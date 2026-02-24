/// DictationLogo — Logo "Dictation IA" pour le header du sidebar
/// Remplace HandyTextLogo (branding Handy supprimé)

interface DictationLogoProps {
  width?: number | string;
  className?: string;
}

export default function DictationLogo({ width = 120, className = "" }: DictationLogoProps) {
  return (
    <div
      className={`flex flex-col items-center gap-1.5 select-none ${className}`}
      style={{ width }}
    >
      {/* Icône micro */}
      <svg
        viewBox="0 0 48 48"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        style={{ width: 36, height: 36 }}
      >
        {/* Corps du micro */}
        <rect x="18" y="4" width="12" height="22" rx="6" className="logo-primary" />
        {/* Arc de la base */}
        <path
          d="M10 24c0 7.732 6.268 14 14 14s14-6.268 14-14"
          stroke="currentColor"
          strokeWidth="3"
          strokeLinecap="round"
          className="logo-stroke"
          style={{ fill: "none" }}
        />
        {/* Pied */}
        <line
          x1="24" y1="38" x2="24" y2="44"
          stroke="currentColor"
          strokeWidth="3"
          strokeLinecap="round"
          className="logo-stroke"
        />
        <line
          x1="16" y1="44" x2="32" y2="44"
          stroke="currentColor"
          strokeWidth="3"
          strokeLinecap="round"
          className="logo-stroke"
        />
      </svg>

      {/* Texte app */}
      <div className="flex flex-col items-center leading-tight">
        <span
          className="text-xs font-bold tracking-wide"
          style={{ color: "var(--color-logo-primary)" }}
        >
          Dictation
        </span>
        <span className="text-[10px] font-medium opacity-60">IA Locale</span>
      </div>
    </div>
  );
}
