export type Lang = "en" | "hi";

type Dict = Record<string, string>;

const en: Dict = {
  appName: "Sarvo Shala",
  wizardTitle: "Welcome to Sarvo Shala",
  chooseLanguage: "Choose language",
  chooseGrade: "Choose class",
  chooseBoard: "Choose board",
  start: "Start",
  browse: "Browse",
  tutor: "Tutor",
  quiz: "Quiz",
  progress: "Progress",
  askPlaceholder: "Ask the tutor…",
  tutorPlaceholder: "Type your answer…",
  you: "You",
  send: "Send",
  hint: "Hint",
  startQuiz: "Start quiz",
  submit: "Submit",
  next: "Next",
  finish: "Finish",
  score: "Score",
  mastery: "Mastery",
  dueForReview: "Due for review",
  streakDays: "Day streak",
  sampleContentNotice: "Sample content — not NCERT text.",
  loading: "Loading…",
  errorGeneric: "Something went wrong.",
  weight: "weight",
  noDue: "Nothing due for review. Nicely done!",
  stateProbe: "Probe",
  stateClarify: "Clarify",
  stateConfront: "Confront",
  stateScaffold: "Scaffold",
  stateConfirm: "Confirm",
};

const hi: Dict = {
  appName: "सर्वो शाला",
  wizardTitle: "सर्वो शाला में आपका स्वागत है",
  chooseLanguage: "भाषा चुनें",
  chooseGrade: "कक्षा चुनें",
  chooseBoard: "बोर्ड चुनें",
  start: "शुरू करें",
  browse: "अध्याय",
  tutor: "ट्यूटर",
  quiz: "प्रश्नोत्तरी",
  progress: "प्रगति",
  askPlaceholder: "ट्यूटर से पूछें…",
  tutorPlaceholder: "अपना उत्तर लिखें…",
  you: "आप",
  send: "भेजें",
  hint: "संकेत",
  startQuiz: "प्रश्नोत्तरी शुरू करें",
  submit: "जमा करें",
  next: "आगे",
  finish: "समाप्त",
  score: "अंक",
  mastery: "प्रवीणता",
  dueForReview: "समीक्षा के लिए",
  streakDays: "दिनों की शृंखला",
  sampleContentNotice: "नमूना सामग्री — एनसीईआरटी पाठ्य नहीं।",
  loading: "लोड हो रहा है…",
  errorGeneric: "कुछ गलत हुआ।",
  weight: "भार",
  noDue: "समीक्षा के लिए कुछ नहीं। बहुत बढ़िया!",
  stateProbe: "प्रश्न",
  stateClarify: "स्पष्ट",
  stateConfront: "सामना",
  stateScaffold: "सहायता",
  stateConfirm: "पुष्टि",
};

export function t(key: string, lang: Lang): string {
  const dict = lang === "hi" ? hi : en;
  return dict[key] ?? key;
}

export function useT(lang: Lang): (key: string) => string {
  return (key: string) => t(key, lang);
}
