import en from './en.json';
import fa from './fa.json';
export type Key = keyof typeof en;
export type Translate = (key: Key) => string;
const checked: Record<Key, string> = fa;
export function translator(language: 'en' | 'fa'): Translate { const dictionary = language === 'fa' ? checked : en; return key => dictionary[key]; }
export function errorKey(value: unknown): Key { const code = String(value); return code in en ? code as Key : 'unknown_error'; }
