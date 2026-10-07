import { useEffect, useLayoutEffect, useId, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import type { Layout } from '../lib/types';
import type { Translate } from '../i18n';
import { Icon } from './Icon';
export function LayoutSelect({ label, value, onChange, layouts, t }: { label: string; value: string; onChange: (id: string) => void; layouts: Layout[]; t: Translate }) {
  const [open, setOpen] = useState(false), [query, setQuery] = useState(''), [cursor, setCursor] = useState(0);
  const ref = useRef<HTMLDivElement>(null), search = useRef<HTMLInputElement>(null), id = useId();
  const popup = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState({left:0,top:0,width:0,height:260});
  useLayoutEffect(() => {
    if (!open) return;
    const place = () => {
      const box=ref.current?.querySelector('.layout-trigger')?.getBoundingClientRect();
      if (!box) return;
      const below=innerHeight-box.bottom-12, above=box.top-12;
      const down=below>=Math.min(260,above);
      const height=Math.min(260, Math.max(100, down ? below : above));
      setPosition({left:Math.max(8,Math.min(box.left,innerWidth-box.width-8)),top:down?box.bottom+6:box.top-height-6,width:box.width,height});
    };
    place(); window.addEventListener('resize',place); window.addEventListener('scroll',place,true);
    return () => {window.removeEventListener('resize',place);window.removeEventListener('scroll',place,true);};
  },[open]);
  const contains = (target: EventTarget | null) => target instanceof Node && (ref.current?.contains(target) || popup.current?.contains(target));
  const selected = layouts.find(l => l.id === value);
  const filtered = layouts.filter(l => `${l.name} ${l.language} ${l.id}`.toLocaleLowerCase().includes(query.toLocaleLowerCase()));
  useEffect(() => {
    if (!open) return;
    search.current?.focus();
    const outside = (e: PointerEvent) => { if (!contains(e.target)) setOpen(false); };
    document.addEventListener('pointerdown', outside);
    return () => document.removeEventListener('pointerdown', outside);
  }, [open]);
  useEffect(() => { popup.current?.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: 'nearest' }); }, [cursor]);
  const choose = (layout: Layout) => { onChange(layout.id); setOpen(false); ref.current?.querySelector<HTMLButtonElement>('.layout-trigger')?.focus(); };
  return <div className="layout-select" ref={ref} onBlur={e => { if (!contains(e.relatedTarget)) setOpen(false); }} onKeyDown={e => {
    if (e.key === 'Escape') { e.stopPropagation(); setOpen(false); ref.current?.querySelector<HTMLButtonElement>('.layout-trigger')?.focus(); }
    if (open && ['ArrowDown', 'ArrowUp'].includes(e.key)) { e.preventDefault(); setCursor(i => Math.max(0, Math.min(filtered.length - 1, i + (e.key === 'ArrowDown' ? 1 : -1)))); }
    if (open && e.key === 'Enter' && filtered[cursor]) { e.preventDefault(); choose(filtered[cursor]); }
  }}><span id={`${id}-label`}>{label}</span><button type="button" className={`layout-trigger ${open ? 'is-open' : ''}`} aria-label={label} aria-haspopup="listbox" aria-expanded={open} aria-controls={`${id}-list`} onClick={() => { setOpen(!open); setQuery(''); setCursor(0); }}><span>{selected?.name ?? t('selectLayout')}</span><Icon name="chevron" size={15} /></button>
    {open && createPortal(<div ref={popup} data-layout-popup className="layout-popup" style={{left:position.left,top:position.top,width:position.width,height:position.height}}><input ref={search} aria-label={t('searchLayouts')} placeholder={t('searchLayouts')} value={query} onChange={e => { setQuery(e.target.value); setCursor(0); }} role="combobox" aria-expanded="true" aria-controls={`${id}-list`} aria-activedescendant={filtered[cursor] ? `${id}-${cursor}` : undefined} /><div className="layout-options" role="listbox" id={`${id}-list`} aria-labelledby={`${id}-label`}>{filtered.map((l, i) => <button type="button" role="option" id={`${id}-${i}`} aria-selected={i === cursor} className={`layout-option ${l.id === value ? 'chosen' : ''}`} key={l.id} onFocus={() => setCursor(i)} onClick={() => choose(l)}><span className="language-badge">{l.language.toUpperCase()}</span><span>{l.name}<small>{l.id.slice(-8)}</small></span>{l.id === value && <Icon name="check" size={15} />}</button>)}{!filtered.length && <p className="hint">{t('noLayouts')}</p>}</div><div className="layout-count">{filtered.length} / {layouts.length}</div></div>, document.body)}
  </div>;
}
