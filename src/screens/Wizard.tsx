import { useState } from "react";
import { completeWizard, type Settings } from "../lib/ipc";
import { useT, type Lang } from "../lib/i18n";

const GRADES = [6, 7, 8, 9, 10, 11, 12];

export default function Wizard({
  onDone,
}: {
  onDone: (s: Settings) => void;
}) {
  const [language, setLanguage] = useState<Lang>("en");
  const [grade, setGrade] = useState(8);
  const [board, setBoard] = useState("CBSE");
  const [error, setError] = useState<string | null>(null);
  const t = useT(language);

  async function start() {
    setError(null);
    try {
      const s = await completeWizard(language, grade, board);
      onDone(s);
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <div className="wizard">
      <h1>{t("appName")}</h1>
      <h2>{t("wizardTitle")}</h2>

      <fieldset>
        <legend>{t("chooseLanguage")}</legend>
        <label>
          <input
            type="radio"
            name="language"
            checked={language === "en"}
            onChange={() => setLanguage("en")}
          />
          English
        </label>
        <label>
          <input
            type="radio"
            name="language"
            checked={language === "hi"}
            onChange={() => setLanguage("hi")}
          />
          हिन्दी
        </label>
      </fieldset>

      <fieldset>
        <legend>{t("chooseGrade")}</legend>
        {GRADES.map((g) => (
          <label key={g}>
            <input
              type="radio"
              name="grade"
              checked={grade === g}
              onChange={() => setGrade(g)}
            />
            {g}
          </label>
        ))}
      </fieldset>

      <fieldset>
        <legend>{t("chooseBoard")}</legend>
        <label>
          <input
            type="radio"
            name="board"
            checked={board === "CBSE"}
            onChange={() => setBoard("CBSE")}
          />
          CBSE
        </label>
      </fieldset>

      {error && <p className="error">{error}</p>}
      <button className="primary" onClick={start}>
        {t("start")}
      </button>
    </div>
  );
}
