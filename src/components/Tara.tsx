import { useEffect, useRef, useState } from "react";

export type TaraState =
  | "curious"
  | "attentive"
  | "encouraging"
  | "supportive"
  | "pleased"
  | "content"
  | "celebrating"
  | "welcoming";

type EyeVariant = "dot" | "arcSoft" | "arcUp";
type MouthVariant =
  | "flat"
  | "content"
  | "smallUp"
  | "wideUp"
  | "gentleUp"
  | "gentleUpPlus"
  | "fullUp";

const EYE_FOR_STATE: Record<TaraState, EyeVariant> = {
  curious: "dot",
  attentive: "dot",
  encouraging: "dot",
  supportive: "arcSoft",
  pleased: "arcUp",
  content: "dot",
  celebrating: "arcUp",
  welcoming: "dot",
};

const MOUTH_FOR_STATE: Record<TaraState, MouthVariant> = {
  curious: "flat",
  attentive: "smallUp",
  encouraging: "wideUp",
  supportive: "gentleUp",
  pleased: "fullUp",
  content: "content",
  celebrating: "fullUp",
  welcoming: "fullUp",
};

// One fixed path per mouth shape, all sharing the same M/Q command structure
// so they occupy the same layout slot — every variant is rendered at all
// times and cross-faded via opacity (see Eyes/Mouths below), never morphed.
const MOUTH_PATH: Record<MouthVariant, string> = {
  flat: "M42,58 Q50,58 58,58",
  content: "M42,58 Q50,56.5 58,58",
  smallUp: "M42,58 Q50,54 58,58",
  wideUp: "M41,57 Q50,51 59,57",
  gentleUp: "M42,58 Q50,53 58,58",
  gentleUpPlus: "M42,58 Q50,51.5 58,58",
  fullUp: "M40,56 Q50,48 60,56",
};
const MOUTH_VARIANTS = Object.keys(MOUTH_PATH) as MouthVariant[];

const EYE_CX = [42, 58] as const;
const EYE_CY = 46;

const ANNOUNCE_KEY: Record<TaraState, string> = {
  curious: "taraCurious",
  attentive: "taraAttentive",
  encouraging: "taraEncouraging",
  supportive: "taraSupportive",
  pleased: "taraPleased",
  content: "taraContent",
  celebrating: "taraCelebrating",
  welcoming: "taraWelcoming",
};

const STAR_PATH =
  "M50,8 L60,36.3 L90,37 L66.2,55.3 L74.7,84 L50,67 L25.3,84 L33.8,55.3 L10,37 L40,36.3 Z";

const FADE_CLASS =
  "transition-opacity duration-150 motion-reduce:transition-none";

function Eyes({ variant }: { variant: EyeVariant }) {
  // All three variants are always rendered; only opacity toggles, so a state
  // change cross-fades between pre-positioned shapes instead of morphing.
  const variants: EyeVariant[] = ["dot", "arcSoft", "arcUp"];
  return (
    <g aria-hidden="true">
      {variants.map((v) => (
        <g key={v} className={FADE_CLASS} style={{ opacity: v === variant ? 1 : 0 }}>
          {EYE_CX.map((cx) => {
            if (v === "dot") {
              return (
                <circle key={cx} cx={cx} cy={EYE_CY} r={2.6} fill="var(--color-primary)" />
              );
            }
            const d =
              v === "arcSoft"
                ? `M${cx - 3},${EYE_CY} Q${cx},${EYE_CY - 2.5} ${cx + 3},${EYE_CY}`
                : `M${cx - 3.5},${EYE_CY + 1} Q${cx},${EYE_CY - 4} ${cx + 3.5},${EYE_CY + 1}`;
            return (
              <path
                key={cx}
                d={d}
                fill="none"
                stroke="var(--color-primary)"
                strokeWidth={2.4}
                strokeLinecap="round"
              />
            );
          })}
        </g>
      ))}
    </g>
  );
}

function Mouth({ variant }: { variant: MouthVariant }) {
  return (
    <g aria-hidden="true">
      {MOUTH_VARIANTS.map((v) => (
        <path
          key={v}
          d={MOUTH_PATH[v]}
          fill="none"
          stroke="var(--color-primary)"
          strokeWidth={2.4}
          strokeLinecap="round"
          className={FADE_CLASS}
          style={{ opacity: v === variant ? 1 : 0 }}
        />
      ))}
    </g>
  );
}

export default function Tara({
  state,
  emphasized = false,
  size,
  t,
}: {
  state: TaraState;
  emphasized?: boolean;
  size: number;
  t?: (key: string) => string;
}) {
  const mouthVariant =
    state === "supportive" && emphasized ? "gentleUpPlus" : MOUTH_FOR_STATE[state];
  const eyeVariant = EYE_FOR_STATE[state];

  const [pulse, setPulse] = useState(false);
  const [announcement, setAnnouncement] = useState("");
  const prevState = useRef<TaraState | null>(null);

  useEffect(() => {
    if (prevState.current === state) return;
    prevState.current = state;

    if (state === "pleased") {
      setPulse(true);
      const id = window.setTimeout(() => setPulse(false), 400);
      return () => window.clearTimeout(id);
    }

    if (t) setAnnouncement(t(ANNOUNCE_KEY[state]));
  }, [state, t]);

  const glowOn = state === "celebrating" || pulse;

  return (
    <span className="inline-block" style={{ width: size, height: size }}>
      <svg
        viewBox="0 0 100 100"
        width={size}
        height={size}
        aria-hidden="true"
        className="overflow-visible"
      >
        <circle
          cx={50}
          cy={50}
          r={46}
          fill="var(--color-celebration)"
          className="transition-opacity duration-300 motion-reduce:transition-none"
          style={{ opacity: glowOn ? (state === "celebrating" ? 0.35 : 0.5) : 0 }}
        />
        <path
          d={STAR_PATH}
          fill="none"
          stroke="var(--color-primary)"
          strokeWidth={5}
          strokeLinejoin="round"
          strokeLinecap="round"
        />
        <Eyes variant={eyeVariant} />
        <Mouth variant={mouthVariant} />
      </svg>
      {t && (
        <span role="status" aria-live="polite" className="sr-only">
          {announcement}
        </span>
      )}
    </span>
  );
}
