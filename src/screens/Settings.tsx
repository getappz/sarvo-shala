import type { Lang } from "../lib/i18n";

export default function Settings({
  language,
  onLanguageChange,
  t,
}: {
  language: Lang;
  onLanguageChange: (l: Lang) => void;
  t: (k: string) => string;
}) {
  return (
    <div className="max-w-md">
      <h2 className="font-display text-lg mb-3">{t("settings")}</h2>
      <fieldset className="border border-border rounded-lg p-4">
        <legend className="text-sm font-semibold px-1">{t("language")}</legend>
        <label className="mr-4 inline-flex items-center gap-2">
          <input
            type="radio"
            name="settings-language"
            checked={language === "en"}
            onChange={() => onLanguageChange("en")}
          />
          English
        </label>
        <label className="inline-flex items-center gap-2">
          <input
            type="radio"
            name="settings-language"
            checked={language === "hi"}
            onChange={() => onLanguageChange("hi")}
          />
          हिन्दी
        </label>
      </fieldset>
    </div>
  );
}
