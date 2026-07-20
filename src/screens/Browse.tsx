import { useEffect, useState } from "react";
import {
  listSubjects,
  listChapters,
  listConcepts,
  type Subject,
  type Chapter,
  type Concept,
} from "../lib/ipc";

export default function Browse({
  grade,
  t,
  onAskTutor,
  onStartQuiz,
}: {
  grade: number;
  t: (k: string) => string;
  onAskTutor: (c: Chapter) => void;
  onStartQuiz: (c: Chapter) => void;
}) {
  const [subjects, setSubjects] = useState<Subject[]>([]);
  const [subjectId, setSubjectId] = useState<number | null>(null);
  const [chapters, setChapters] = useState<Chapter[]>([]);
  const [chapterId, setChapterId] = useState<number | null>(null);
  const [concepts, setConcepts] = useState<Concept[]>([]);

  useEffect(() => {
    listSubjects(grade)
      .then((subs) => {
        setSubjects(subs);
        if (subs.length > 0) setSubjectId(subs[0].id);
      })
      .catch(() => setSubjects([]));
  }, [grade]);

  useEffect(() => {
    if (subjectId == null) return;
    listChapters(subjectId)
      .then((chs) => {
        setChapters(chs);
        setChapterId(chs.length > 0 ? chs[0].id : null);
      })
      .catch(() => setChapters([]));
  }, [subjectId]);

  useEffect(() => {
    if (chapterId == null) return;
    listConcepts(chapterId)
      .then(setConcepts)
      .catch(() => setConcepts([]));
  }, [chapterId]);

  const activeChapter = chapters.find((c) => c.id === chapterId) ?? null;

  return (
    <div className="browse">
      <div className="subject-row">
        {subjects.map((s) => (
          <button
            key={s.id}
            className={s.id === subjectId ? "chip active" : "chip"}
            onClick={() => setSubjectId(s.id)}
          >
            {s.name}
          </button>
        ))}
      </div>

      <div className="browse-body">
        <ul className="chapter-list">
          {chapters.map((c) => (
            <li key={c.id}>
              <button
                className={c.id === chapterId ? "chapter active" : "chapter"}
                onClick={() => setChapterId(c.id)}
              >
                <strong>{c.title}</strong>
                <span>{c.summary}</span>
              </button>
            </li>
          ))}
        </ul>

        {activeChapter && (
          <div className="concept-detail">
            <h3>{activeChapter.title}</h3>
            <div className="concept-cards">
              {concepts.map((cn) => (
                <div key={cn.id} className="concept-card">
                  <div className="concept-name">{cn.name}</div>
                  {cn.keyFormula && (
                    <span className="formula">{cn.keyFormula}</span>
                  )}
                  <div className="weight">
                    {"●".repeat(cn.examWeight)}
                    <span className="weight-label"> weight</span>
                  </div>
                  {cn.commonError && (
                    <div className="common-error">⚠ {cn.commonError}</div>
                  )}
                </div>
              ))}
            </div>
            <div className="chapter-actions">
              <button
                className="primary"
                onClick={() => onAskTutor(activeChapter)}
              >
                {t("tutor")}
              </button>
              <button
                className="primary"
                onClick={() => onStartQuiz(activeChapter)}
              >
                {t("startQuiz")}
              </button>
            </div>
          </div>
        )}
      </div>

      <footer className="notice">{t("sampleContentNotice")}</footer>
    </div>
  );
}
