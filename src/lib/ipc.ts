import { invoke } from "@tauri-apps/api/core";

export interface Settings {
  language: string;
  grade: number;
  board: string;
  onboarded: boolean;
}

export function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export function completeWizard(
  language: string,
  grade: number,
  board: string
): Promise<Settings> {
  return invoke<Settings>("complete_wizard", { language, grade, board });
}

export interface Subject {
  id: number;
  grade: number;
  name: string;
  nameHi: string;
}

export interface Chapter {
  id: number;
  subjectId: number;
  ord: number;
  title: string;
  titleHi: string;
  summary: string;
}

export interface Concept {
  id: number;
  chapterId: number;
  ord: number;
  name: string;
  keyFormula: string;
  commonError: string;
  examWeight: number;
}

export function listSubjects(grade: number): Promise<Subject[]> {
  return invoke<Subject[]>("list_subjects", { grade });
}

export function listChapters(subjectId: number): Promise<Chapter[]> {
  return invoke<Chapter[]>("list_chapters", { subjectId });
}

export function listConcepts(chapterId: number): Promise<Concept[]> {
  return invoke<Concept[]>("list_concepts", { chapterId });
}
