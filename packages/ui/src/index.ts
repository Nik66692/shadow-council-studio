export const sprint0Sections = [
  "Dashboard",
  "Import canonico",
  "Codex",
  "Carte",
  "Agende e Direttive",
  "RFC",
  "Decisioni",
  "Playtest",
  "Asset",
  "Release",
  "Impostazioni",
] as const;
export type Sprint0Section = (typeof sprint0Sections)[number];
