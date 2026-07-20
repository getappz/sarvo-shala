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
