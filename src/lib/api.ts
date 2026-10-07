import { invoke, isTauri } from '@tauri-apps/api/core';
import type { Analysis, Bootstrap, Config, Conversion, Layout } from './types';

export const native = isTauri();
function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!native) return Promise.reject('native_required');
  return invoke<T>(command, args);
}
export const api = {
  window: (action: 'minimize' | 'maximize' | 'close' | 'drag') => call<void>('window_action', { action }),
  compact: (compact: boolean) => call<void>('set_compact', { compact }),
  bootstrap: () => call<Bootstrap>('bootstrap'),
  refresh: () => call<Layout[]>('refresh_layouts'),
  convert: (text: string, source: string, target: string) => call<Conversion>('convert_text', { text, source, target }),
  analyze: (text: string) => call<Analysis>('analyze_text', { text }),
  save: (config: Config) => call<Config>('save_settings', { config }),
  read: () => call<string>('read_clipboard'),
  write: (text: string) => call<void>('write_clipboard', { text }),
  pause: (paused: boolean) => call<void>('set_paused', { paused }),
  pending: () => call<Analysis | null>('pending_selection'),
  apply: (index: number) => call<void>('apply_selection', { index }),
  cancel: () => call<void>('cancel_selection'),
};
