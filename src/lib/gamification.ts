export function badgeLabel(key: string, t: (k: string) => string): string {
  if (key.startsWith("chapter_mastered:")) return t("badgeChapterMastered");
  if (key === "streak_7") return t("badgeStreak7");
  if (key === "streak_30") return t("badgeStreak30");
  return key;
}
