export interface Config {
  uiLanguage: 'en' | 'fa'; theme: 'dark' | 'light' | 'system';
  autoHotkey: string; reverseHotkey: string; openHotkey: string;
  preferredLayouts: string[]; source: string; target: string; threshold: number;
  launchAtStartup: boolean; minimizeToTray: boolean; startMinimized: boolean; notifications: boolean;
  reduceMotion: boolean; animationIntensity: number; restoreClipboard: boolean; debugLogs: boolean; onboarded: boolean;
}
export interface Layout { id: string; name: string; language: string; active: boolean }
export interface Conversion { text: string; mapped: number; unmapped: number }
export interface Candidate { source: string; target: string; language: string; name: string; text: string; confidence: number; unmapped: number; reason: string }
export interface Analysis { candidates: Candidate[]; action: 'apply' | 'choose' | 'unchanged'; originalConfidence: number }
export interface Bootstrap { config: Config; layouts: Layout[]; errors: string[]; paused: boolean; version: string; systemLanguage: string }
export const defaults: Config = {
  uiLanguage: 'en', theme: 'dark', autoHotkey: 'Ctrl+Shift+Space', reverseHotkey: 'Ctrl+Alt+Shift+Space', openHotkey: 'Ctrl+Alt+P',
  preferredLayouts: [], source: '', target: '', threshold: .86,
  launchAtStartup: false, minimizeToTray: true, startMinimized: false, notifications: true,
  reduceMotion: false, animationIntensity: 1, restoreClipboard: true, debugLogs: false, onboarded: false,
};
