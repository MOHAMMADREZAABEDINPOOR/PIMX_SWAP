import { useCallback, useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { api, native } from './lib/api';
import { defaults, type Analysis, type Config, type Layout } from './lib/types';
import { errorKey, translator, type Key } from './i18n';
import { Icon, Logo } from './components/Icon';
import { LayoutSelect } from './components/LayoutSelect';
import { Candidates } from './components/Candidates';
import { Onboarding } from './components/Onboarding';
import { Settings } from './pages/Settings';
import { About } from './pages/About';
import { useHistory } from './hooks/useHistory';

type Page = 'converter' | 'settings' | 'about';
export function App() {
  const [config, setConfig] = useState<Config>(defaults); const [layouts, setLayouts] = useState<Layout[]>([]);
  const [page, setPage] = useState<Page>(new URLSearchParams(location.search).get('page') === 'settings' ? 'settings' : 'converter');
  const [mode, setMode] = useState<'auto' | 'direct'>('auto'); const [source, setSource] = useState(''); const [target, setTarget] = useState('');
  const [output, setOutput] = useState(''); const [analysis, setAnalysis] = useState<Analysis | null>(null); const [global, setGlobal] = useState(false);
  const [busy, setBusy] = useState(false); const [loaded, setLoaded] = useState(false); const [paused, setPaused] = useState(false);
  const [errors, setErrors] = useState<string[]>([]); const [version, setVersion] = useState('1.0.0'); const [notice, setNotice] = useState<{ key: Key; error: boolean } | null>(null);
  const [compact, setCompact] = useState(false);
  useEffect(() => { if (!notice) return; const timer = setTimeout(() => setNotice(null), 5000); return () => clearTimeout(timer); }, [notice]);
  const toggleCompact = async () => { try { if (native) await api.compact(!compact); setCompact(!compact); setPage('converter'); } catch (e) { showError(e); } };
  const history = useHistory(); const t = translator(config.uiLanguage);
  useEffect(() => { window.scrollTo({ top: 0 }); }, [page]);
  const showError = useCallback((e: unknown) => setNotice({ key: errorKey(e), error: true }), []);
  const show = (key: Key) => setNotice({ key, error: false });
  const sync = useCallback(async () => {
    try { const b = await api.bootstrap(); setConfig(b.config); setSource(b.config.source); setTarget(b.config.target); setLayouts(b.layouts); setPaused(b.paused); setErrors(b.errors); setVersion(b.version);
      if (!b.config.onboarded && b.systemLanguage === 'fa') setConfig({ ...b.config, uiLanguage: 'fa' });
      if (b.errors.length) showError(b.errors[0]);
    } catch (e) { showError(e); } finally { setLoaded(true); }
  }, [showError]);
  const loadPending = useCallback(async () => { try { const p = await api.pending(); if (p) { setAnalysis(p); setGlobal(true); setPage('converter'); } } catch (e) { showError(e); } }, [showError]);
  useEffect(() => { void sync(); if (native) void loadPending(); }, [sync, loadPending]);
  useEffect(() => {
    document.documentElement.lang = config.uiLanguage; document.documentElement.dir = config.uiLanguage === 'fa' ? 'rtl' : 'ltr';
    const media = matchMedia('(prefers-color-scheme: dark)');
    const update = () => { document.documentElement.dataset.theme = config.theme === 'system' ? media.matches ? 'dark' : 'light' : config.theme; };
    update(); media.addEventListener('change', update); return () => media.removeEventListener('change', update);
  }, [config.uiLanguage, config.theme]);
  useEffect(() => {
    if (!native) return;
    let disposed = false; const unlisteners: (() => void)[] = [];
    const add = async <T,>(name: string, handler: (payload: T) => void) => { const off = await listen<T>(name, e => handler(e.payload)); if (disposed) off(); else unlisteners.push(off); };
    void add<string>('navigate', p => { if (p === 'settings') { void api.compact(false).then(() => { setCompact(false); setPage('settings'); }).catch(showError); } else setPage('converter'); });
    void add('settings-changed', () => { void sync(); }); void add('selection-ready', () => { void loadPending(); });
    void add<string>('operation-status', code => setNotice({ key: errorKey(code), error: !['converted', 'unchanged'].includes(code) }));
    return () => { disposed = true; unlisteners.forEach(off => off()); };
  }, [sync, loadPending]);
  const save = async (c: Config) => { try { const saved = await api.save(c); setConfig(saved); setSource(saved.source); setTarget(saved.target); show('saved'); } catch (e) { showError(e); throw e; } };
  const refresh = async () => { try { setLayouts(await api.refresh()); show('layoutsRefreshed'); } catch (e) { showError(e); } };
  const language = async () => { const c = { ...config, uiLanguage: config.uiLanguage === 'en' ? 'fa' as const : 'en' as const }; if (!native || !config.onboarded) setConfig(c); else { try { await save(c); } catch { /* The localized error is displayed by save. */ } } };
  const convert = async () => {
    if (busy) return; if (!history.value.trim()) { showError('empty_text'); return; }
    setBusy(true); setAnalysis(null); setGlobal(false);
    try {
      if (mode === 'direct') { const c = await api.convert(history.value, source, target); setOutput(c.text); show('converted'); }
      else { const a = await api.analyze(history.value); if (a.action === 'apply') { setOutput(a.candidates[0].text); show('converted'); } else if (a.action === 'unchanged') { setOutput(history.value); show('unchanged'); } else { setAnalysis(a); setOutput(''); } }
    } catch (e) { showError(e); } finally { setBusy(false); }
  };
  const choose = async (index: number) => { if (!analysis) return; if (!global) { setOutput(analysis.candidates[index].text); setAnalysis(null); show('converted'); return; } setBusy(true); try { await api.apply(index); setAnalysis(null); setGlobal(false); show('converted'); } catch (e) { showError(e); setAnalysis(null); setGlobal(false); } finally { setBusy(false); } };
  const cancel = async () => { if (global) { try { await api.cancel(); } catch (e) { showError(e); } } setGlobal(false); setAnalysis(null); };
  const button = (name: 'paste' | 'clear' | 'undo' | 'redo', action: () => void, disabled = false) => <button className="icon-button" title={t(name)} aria-label={t(name)} onClick={action} disabled={disabled}><Icon name={name} /></button>;
  return <><div className="titlebar" dir="ltr"><div className="titlebar-brand" onPointerDown={e => { if (native && e.button === 0) void api.window('drag').catch(showError); }} onDoubleClick={() => { if (native) void api.window('maximize').catch(showError); }}><Logo size={19} /><span>PIMXSWAP</span><span className="titlebar-mode">{compact ? t('compact') : t(page)}</span></div><div className="window-controls"><button title={t(compact ? 'expand' : 'compact')} aria-label={t(compact ? 'expand' : 'compact')} onClick={() => void toggleCompact()}><Icon name={compact ? 'expand' : 'compact'} size={15} /></button>{(['minimize', 'maximize', 'close'] as const).map(action => <button key={action} className={action === 'close' ? 'window-close' : ''} disabled={!native} title={t(action === 'close' ? 'closeWindow' : action)} aria-label={t(action === 'close' ? 'closeWindow' : action)} onClick={() => void api.window(action).catch(showError)}><Icon name={action} size={15} /></button>)}</div></div><div className={`app ${compact ? 'compact-workspace' : ''} ${config.reduceMotion ? 'reduce-motion' : ''} effects-${config.animationIntensity}`}>
    <aside className="sidebar"><div className="brand"><Logo /><span>PIMX<span className="brand-weight">SWAP</span></span></div><span className="sidebar-label">{t('keyboardUtility')}</span><nav aria-label="PIMXSWAP">{(['converter', 'settings', 'about'] as const).map((p, i) => <button key={p} aria-label={t(p)} title={t(p)} className={page === p ? 'nav-item active' : 'nav-item'} onClick={() => setPage(p)}><Icon name={(['swap', 'settings', 'info'] as const)[i]} /><span>{t(p)}</span>{page === p && <span className="nav-dot" />}</button>)}</nav><div className="sidebar-bottom"><div className="privacy-icon"><Icon name="shield" /></div><p>{t('privacy')}</p><span className="core-label">{t('brandTagline')}</span></div></aside>
    <div className="main-shell"><header className="topbar"><div className="breadcrumb"><span>PIMXSWAP</span><span>/</span><strong>{t(page)}</strong></div><div className="topbar-actions"><span className="offline"><span className="status-dot" />{t('offline')}</span><button className="language-button" onClick={() => void language()} aria-label={t('appLanguage')}>{config.uiLanguage === 'en' ? t('persian') : t('english')}</button><button className="icon-button" aria-label={t(config.theme === 'dark' ? 'light' : 'dark')} onClick={() => { const c = { ...config, theme: config.theme === 'dark' ? 'light' as const : 'dark' as const }; if (native) void save(c).catch(() => {}); else setConfig(c); }}><Icon name={config.theme === 'dark' ? 'sun' : 'moon'} /></button></div></header>
      <main key={page}>{page === 'converter' && <div className="page-content workspace"><div className="hero"><span className="eyebrow"><span className="mini-line" />{t('eyebrow')}</span><h1>{t('headline')}<br /><span>{t('headlineAccent')}</span></h1><p>{t('description')}</p></div><div className="workspace-toolbar"><div className="segmented" aria-label={t('manual')}>{(['auto', 'direct'] as const).map(m => <button key={m} className={mode === m ? 'selected' : ''} aria-pressed={mode === m} onClick={() => { setMode(m); setAnalysis(null); }}><Icon name={m === 'auto' ? 'spark' : 'swap'} />{t(m)}</button>)}</div><span className="toolbar-note">{mode === 'auto' ? t('autoDetect') : t('brandTagline')}</span></div>
        <div className="editor-grid"><section className="editor-card input-card" onPointerMove={e => { if (config.reduceMotion || !config.animationIntensity) return; const r = e.currentTarget.getBoundingClientRect(); e.currentTarget.style.setProperty('--pointer-x', `${e.clientX - r.left}px`); e.currentTarget.style.setProperty('--pointer-y', `${e.clientY - r.top}px`); }}><div className="editor-heading"><div><span className="editor-dot" /><label htmlFor="input-text">{t('input')}</label></div><div className="editor-tools">{button('paste', () => { void api.read().then(history.set).catch(showError); })}{button('undo', history.undo, !history.canUndo)}{button('redo', history.redo, !history.canRedo)}{button('clear', () => { history.set(''); setOutput(''); setAnalysis(null); }, !history.value)}</div></div><textarea id="input-text" dir="auto" value={history.value} maxLength={100000} placeholder={t('inputPlaceholder')} spellCheck={false} onChange={e => { history.set(e.target.value); setAnalysis(null); }} onKeyDown={e => { if (e.ctrlKey && e.key === 'Enter') { e.preventDefault(); void convert(); } if (e.ctrlKey && e.key.toLowerCase() === 'z') { e.preventDefault(); if (e.shiftKey) history.redo(); else history.undo(); } if (e.ctrlKey && e.key.toLowerCase() === 'y') { e.preventDefault(); history.redo(); } }} /><div className="editor-footer"><span><bdi>{Array.from(history.value).length}</bdi> {t('characters')} <span>·</span> <bdi>{history.value.trim() ? history.value.trim().split(/\s+/u).length : 0}</bdi> {t('words')}</span><span className="layout-code">{mode === 'auto' ? <Icon name="spark" size={14} /> : layouts.find(l => l.id === source)?.language.toUpperCase()}</span></div></section>
          <div className="editor-connector"><Icon name="arrow" /></div><section className="editor-card output-card"><div className="editor-heading"><div><span className="editor-dot mint" /><label htmlFor="output-text">{t('output')}</label></div><button className="icon-button" disabled={!output} title={t('copy')} aria-label={t('copy')} onClick={() => { void api.write(output).then(() => show('copied')).catch(showError); }}><Icon name="copy" /></button></div><textarea id="output-text" dir="auto" value={output} readOnly placeholder={t('outputPlaceholder')} /><div className="editor-footer"><span>{output ? <><Icon name="check" size={13} />{t('converted')}</> : t('privacy')}</span><span className="layout-code">{mode === 'direct' ? layouts.find(l => l.id === target)?.language.toUpperCase() : ''}</span></div></section></div>
        {mode === 'direct' && <div className="layout-bar"><LayoutSelect label={t('source')} value={source} onChange={setSource} layouts={layouts} t={t} /><button className="icon-button swap-button" aria-label={t('swap')} title={t('swap')} onClick={() => { setSource(target); setTarget(source); if (output) { history.set(output); setOutput(history.value); } }}><Icon name="swap" /></button><LayoutSelect label={t('target')} value={target} onChange={setTarget} layouts={layouts} t={t} /></div>}
        <div className="convert-row"><span className="hint" dir="auto">{t('convertHint')}</span><button className="primary convert-button" disabled={busy || !history.value.trim() || !loaded} onClick={() => void convert()}><Icon name={mode === 'auto' ? 'spark' : 'swap'} />{t(busy ? 'working' : mode === 'auto' ? 'auto' : 'convert')}<span className="button-arrow"><Icon name="arrow" /></span></button></div>
        {analysis && <Candidates analysis={analysis} global={global} busy={busy} onChoose={i => void choose(i)} onCancel={() => void cancel()} t={t} />}
        <div className="shortcut-card"><div className="shortcut-icon"><Icon name="keyboard" size={25} /></div><div><strong>{t('shortcutHint')}</strong><p>{t('shortcutDescription')}</p></div><kbd>{config.autoHotkey.split('+').join(' + ')}</kbd></div></div>}
        {page === 'settings' && <Settings config={config} layouts={layouts} t={t} onSave={async c => { try { await save(c); } catch { /* Keep the draft and localized notice on failure. */ } }} onRefresh={() => void refresh()} />}
        {page === 'about' && <About t={t} version={version} errors={errors} diagnostics={config.debugLogs} />}
      </main><footer className="app-footer"><button className="status-control" onClick={() => { void api.pause(!paused).then(() => setPaused(!paused)).catch(showError); }} aria-label={t(paused ? 'resume' : 'pause')}><span className={`status-dot ${paused ? 'paused' : ''}`} />{t(paused ? 'paused' : 'ready')}</button><span>PIMXSWAP <bdi>{version}</bdi></span></footer></div>
    {notice && <div className={`toast ${notice.error ? 'error-toast' : ''}`} role={notice.error ? 'alert' : 'status'}><Icon name={notice.error ? 'info' : 'check'} /><span>{t(notice.key)}</span><button className="icon-button" aria-label={t('dismiss')} onClick={() => setNotice(null)}><Icon name="close" size={16} /></button></div>}
    {native && loaded && !config.onboarded && <Onboarding config={config} layouts={layouts} t={t} onLanguage={uiLanguage => setConfig(c => ({ ...c, uiLanguage }))} onSave={async c => { try { await save(c); } catch { /* Preserve onboarding and display the error. */ } }} />}
  </div></>;
}
