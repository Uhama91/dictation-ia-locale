import { listen } from "@tauri-apps/api/event";
import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import "./RecordingOverlay.css";
import { commands } from "@/bindings";
import i18n, { syncLanguageFromSettings } from "@/i18n";
import { getLanguageDirection } from "@/lib/utils/rtl";

type OverlayState = "recording" | "transcribing" | "processing" | "success" | "error";

const LOOP_WIDTH = 10;
const NUM_LOOPS = 26;
const SVG_WIDTH = 200; // Full width
const SVG_HEIGHT = 40; // Full height
const CENTER_Y = 20; // Vertical center
const MIN_AMP = 6; // Plus petit à l'origine
const MAX_AMP = 36; // Maximum headroom
const SCROLL_SPEED = 0.5; // Légèrement plus lent
const WAVE_SPEED = 0.1; // Vitesse de propagation de l'onde interne

/** Renders animated cursive "e" loops that scroll and undulate, responding to voice */
const CursiveLoops: React.FC<{ level: number }> = React.memo(({ level }) => {
  const offsetRef = useRef(0);
  const timeRef = useRef(0);
  const svgRef = useRef<SVGPathElement>(null);
  const animRef = useRef<number>(0);
  const levelRef = useRef(level);
  const smoothAmpRef = useRef(0);

  // Pre-calculate random variations for loops (only once) so it stays consistent
  const loopVars = useRef(
    Array.from({ length: NUM_LOOPS + 2 }).map(() => ({
      widthMod: 0.8 + Math.random() * 0.4, // Between 0.8 and 1.2 x LOOP_WIDTH
      ampMod: 0.8 + Math.random() * 0.4,   // Between 0.8 and 1.2 x base loop height
    }))
  ).current;

  // We need the total width of one full loop cycle to know when to wrap
  const totalLoopsWidth = useMemo(() => {
    return loopVars.reduce((sum, v) => sum + LOOP_WIDTH * v.widthMod, 0);
  }, [loopVars]);

  levelRef.current = level;

  const buildPath = useCallback((xOffset: number, waveAmplitude: number, time: number): string => {
    const safeOffset = xOffset % totalLoopsWidth;
    let currentX = -safeOffset - (LOOP_WIDTH * 2);

    let d = "";

    for (let i = 0; i < loopVars.length * 2; i++) {
      const v = loopVars[i % loopVars.length];
      const loopW = LOOP_WIDTH * v.widthMod;

      const nextX = currentX + loopW;
      const midX = currentX + loopW * 0.5;

      // Distance from center (0 at SVG center, 1 at SVG edges)
      const distFromCenter = Math.abs(midX - (SVG_WIDTH / 2)) / (SVG_WIDTH / 2);

      // Smooth bell curve envelope (1 in middle, tapers near 0 at edges)
      const envelope = Math.max(0, 1 - Math.pow(distFromCenter, 1.5));

      // Organic traveling wave rippling outward from the center
      const travelingWave = Math.sin(-distFromCenter * 6 + time) * 0.4 + 0.6;

      // actual amplitude applies the envelope and the ripple to the voice level
      let actualAmp = MIN_AMP + (waveAmplitude * v.ampMod * envelope * travelingWave);
      actualAmp = Math.min(MAX_AMP, Math.max(MIN_AMP, actualAmp));

      // Calculate symmetric top and bottom, bounded to avoid clipping out of the 40px SVG
      const top = Math.max(2, CENTER_Y - actualAmp / 2);
      const bottom = Math.min(SVG_HEIGHT - 2, CENTER_Y + actualAmp / 2);

      if (i === 0) {
        d = `M ${currentX},${bottom}`;
      }

      // First curve: up to top
      d += ` C ${currentX + loopW * 0.5},${bottom} ${nextX + loopW * 0.1},${top} ${nextX - loopW * 0.2},${top}`;
      // Second curve: down to bottom
      d += ` C ${currentX + loopW * 0.2},${top} ${nextX - loopW * 0.2},${bottom} ${nextX},${bottom}`;

      currentX = nextX;

      if (currentX > SVG_WIDTH + (LOOP_WIDTH * 2)) break;
    }
    return d;
  }, [totalLoopsWidth, loopVars]);

  useEffect(() => {
    const animate = () => {
      offsetRef.current += SCROLL_SPEED;
      timeRef.current -= WAVE_SPEED;

      // Target amplitude (scale voice level to 38 effectively)
      const targetAmp = Math.max(0, levelRef.current * 38);

      // Increased smoothing for better fluidity
      smoothAmpRef.current += (targetAmp - smoothAmpRef.current) * 0.12;

      if (svgRef.current) {
        svgRef.current.setAttribute("d", buildPath(offsetRef.current, smoothAmpRef.current, timeRef.current));
      }
      animRef.current = requestAnimationFrame(animate);
    };
    animRef.current = requestAnimationFrame(animate);
    return () => cancelAnimationFrame(animRef.current);
  }, [buildPath]);

  return (
    <svg width={SVG_WIDTH} height={SVG_HEIGHT} style={{ overflow: "hidden", display: "block" }}>
      <path
        ref={svgRef}
        d={buildPath(0, 0, 0)}
        fill="none"
        stroke="#000000"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
});

const RecordingOverlay: React.FC = () => {
  const [isVisible, setIsVisible] = useState(false);
  const [state, setState] = useState<OverlayState>("recording");
  const [voiceLevel, setVoiceLevel] = useState(0);
  const smoothedLevelRef = useRef(0);
  const direction = getLanguageDirection(i18n.language);

  useEffect(() => {
    const setupEventListeners = async () => {
      const unlistenShow = await listen("show-overlay", async (event) => {
        await syncLanguageFromSettings();
        const overlayState = event.payload as OverlayState;
        setState(overlayState);

        // Only show for recording state; other states → just hide
        if (overlayState === "recording") {
          setIsVisible(true);
        } else {
          setIsVisible(false);
        }
      });

      const unlistenShowError = await listen<string>("show-overlay-error", async () => {
        await syncLanguageFromSettings();
        setIsVisible(false);
      });

      const unlistenHide = await listen("hide-overlay", () => {
        setIsVisible(false);
      });

      // Voice level for loop amplitude
      const unlistenLevel = await listen<number[]>("mic-level", (event) => {
        const levels = event.payload;
        // Average of all levels for a single amplitude value
        const avg = levels.reduce((sum, v) => sum + v, 0) / (levels.length || 1);
        // Boost low volumes with square root to make it react visibly to normal speech
        const boostedAvg = Math.sqrt(avg) * 4.0;
        // Smooth
        smoothedLevelRef.current = smoothedLevelRef.current * 0.4 + boostedAvg * 0.6;
        setVoiceLevel(Math.min(1, smoothedLevelRef.current));
      });

      return () => {
        unlistenShow();
        unlistenShowError();
        unlistenHide();
        unlistenLevel();
      };
    };

    setupEventListeners();
  }, []);

  return (
    <div
      dir={direction}
      className={`recording-overlay ${isVisible ? "fade-in" : ""}`}
    >
      <div className="overlay-middle">
        {state === "recording" && <CursiveLoops level={voiceLevel} />}
      </div>
    </div>
  );
};

export default RecordingOverlay;
