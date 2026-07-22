export type MasteryLevel = "not_started" | "learning" | "practiced" | "mastered";

export function masteryLevel(attempts: number, pKnown: number): MasteryLevel {
  if (attempts === 0) return "not_started";
  if (pKnown < 0.5) return "learning";
  if (pKnown < 0.85) return "practiced";
  return "mastered";
}
