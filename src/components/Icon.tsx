export type IconName = 'swap' | 'spark' | 'settings' | 'info' | 'copy' | 'paste' | 'clear' | 'undo' | 'redo' | 'arrow' | 'shield' | 'keyboard' | 'check' | 'refresh' | 'close' | 'chevron' | 'sun' | 'moon' | 'compact' | 'expand' | 'minimize' | 'maximize';
const paths: Record<IconName, string> = {
  compact: 'M4 4h16v16H4V4ZM4 9h16M9 9v11', expand: 'M8 3H3v5M16 3h5v5M3 16v5h5M21 16v5h-5', minimize: 'M5 12h14', maximize: 'M5 5h14v14H5V5Z',
  swap: 'M4 7h15m-4-4 4 4-4 4M20 17H5m4 4-4-4 4-4',
  spark: 'm12 3 2.5 6.5L21 12l-6.5 2.5L12 21l-2.5-6.5L3 12l6.5-2.5L12 3Z',
  settings: 'M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8ZM9 3h6l1 3 3 1 2 5-2 5-3 1-1 3H9l-1-3-3-1-2-5 2-5 3-1 1-3Z',
  info: 'M12 8h.01M12 11v6M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18Z',
  copy: 'M9 9h11v11H9V9ZM15 9V4H4v11h5',
  paste: 'M9 4H5v17h14V4h-4M9 2h6v5H9V2ZM8 11h8M8 15h6',
  clear: 'M4 6h16M9 6V3h6v3M7 6l1 15h8l1-15M10 10v7M14 10v7',
  undo: 'M8 5 3 10l5 5M3 10h11a6 6 0 0 1 0 12',
  redo: 'm16 5 5 5-5 5M21 10H10a6 6 0 0 0 0 12',
  arrow: 'M5 12h14m-6-6 6 6-6 6',
  shield: 'm12 3 8 3v6c0 5-8 9-8 9s-8-4-8-9V6l8-3Zm-4 9 3 3 5-6',
  keyboard: 'M3 5h18v14H3V5ZM6 9h.01M10 9h.01M14 9h.01M18 9h.01M6 12h.01M10 12h.01M14 12h.01M18 12h.01M8 16h8',
  check: 'm5 12 4 4L19 6', refresh: 'M20 7v5h-5M4 17v-5h5M6 7a7 7 0 0 1 12-2l2 2M4 17l2 2a7 7 0 0 0 12-2',
  close: 'm6 6 12 12M18 6 6 18', chevron: 'm8 5 7 7-7 7',
  sun: 'M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8ZM12 2v2M12 20v2M2 12h2M20 12h2M5 5l1 1M18 18l1 1M19 5l-1 1M6 18l-1 1',
  moon: 'M20 15A9 9 0 0 1 9 4a9 9 0 1 0 11 11Z',
};
export function Icon({ name, size = 18 }: { name: IconName; size?: number }) { return <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d={paths[name]} /></svg>; }
export function Logo({ size = 32 }: { size?: number }) { return <svg width={size} height={size} viewBox="0 0 64 64" fill="none" aria-hidden="true"><path d="M12 12h31l11 11-11 11v-8H24l-8 8H4l16-16h23l-6-6H12Z" fill="currentColor"/><path d="M52 52H21L10 41l11-11v8h19l8-8h12L44 46H21l6 6h25Z" fill="currentColor" opacity=".55"/></svg>; }
