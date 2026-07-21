export const sprint0Sections = [
  "Dashboard",
  "Import canonico",
  "Canon Review",
  "Database Studio",
  "Cloud & Sync",
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
