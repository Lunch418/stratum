import { useEffect } from 'react';
import { api } from './api';
import { useStore } from './store';
import { SchemeCanvas } from './components/SchemeCanvas';
import { CodeEditor } from './components/CodeEditor';
import { Hierarchy } from './components/Hierarchy';
import { Inspector } from './components/Inspector';
import { ModelView } from './components/ModelView';
import { Messages } from './components/Messages';

export default function App() {
  const s = useStore();
  const frame = s.frame;

  useEffect(() => {
    s.load().then(() => {
      // #code/Имя, #model, #scheme — прямые ссылки на вкладку и имидж
      const [tab, name] = location.hash.slice(1).split('/');
      if (tab === 'code' || tab === 'model' || tab === 'scheme') s.setTab(tab);
      if (name) s.select(decodeURIComponent(name));
    }).catch(e => s.say({ level: 'error', where: 'ядро', text: String(e) }));
  }, []);
  useEffect(() => { document.documentElement.dataset.theme = s.theme; }, [s.theme]);

  // опрос кадра
  useEffect(() => {
    let alive = true;
    const poll = async () => {
      try { const f = await api.frame(); if (alive) s.setFrame(f); } catch { /* ядро недоступно */ }
      if (alive) setTimeout(poll, s.tab === 'model' ? 40 : 250);
    };
    poll();
    return () => { alive = false; };
  }, [s.tab]);

  // горячие клавиши транспорта и переключения вкладок
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.target as HTMLElement).closest('.monaco-editor, input, textarea')) return;
      if (e.code === 'F5' && e.shiftKey) { e.preventDefault(); api.event('type=reset').then(() => s.refreshInstances()); }
      else if (e.code === 'F5') { e.preventDefault(); api.event(frame?.running ? 'type=pause' : 'type=run'); }
      else if (e.code === 'F10') { e.preventDefault(); api.event('type=step'); }
      else if (e.key === 'e' && e.ctrlKey) { e.preventDefault(); s.setTab(s.tab === 'code' ? 'scheme' : 'code'); }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [frame?.running, s.tab]);

  const running = !!frame?.running;
  return (
    <div className="ide">
      <header className="topbar">
        <span className="brand">Stratum Modern</span>
        <button className={`primary${running ? ' paused' : ''}`} onClick={() => api.event(running ? 'type=pause' : 'type=run')} title="F5">{running ? 'Пауза' : 'Пуск'}</button>
        <button onClick={() => api.event('type=step')} title="F10">Шаг</button>
        <button onClick={() => api.event('type=reset').then(() => s.refreshInstances())} title="Shift+F5">Сброс</button>
        <label className="muted" style={{ display: 'flex', alignItems: 'center', gap: 6 }}>Скорость
          <input type="range" min={1} max={200} defaultValue={30} onChange={e => api.event('type=speed&fps=' + e.target.value)} />
        </label>
        <span className="counter mono">такт {frame?.tick ?? 0}{frame?.stopped ? ' · остановлено' : ''}</span>
        <button className="ghost" onClick={s.toggleTheme} title="Тема">{s.theme === 'light' ? 'Тёмная' : 'Светлая'}</button>
      </header>
      <div className="workspace">
        <aside className="left"><Hierarchy /></aside>
        <section className="center">
          <div className="tabs">
            <button className={s.tab === 'scheme' ? 'active' : ''} onClick={() => s.setTab('scheme')}>Схема</button>
            <button className={s.tab === 'code' ? 'active' : ''} onClick={() => s.setTab('code')}>Код{s.selectedClass ? ` · ${s.selectedClass}` : ''}</button>
            <button className={s.tab === 'model' ? 'active' : ''} onClick={() => s.setTab('model')}>Окно модели{frame?.windows.length ? ` (${frame.windows.length})` : ''}</button>
          </div>
          {s.tab === 'scheme' && (
            <div className="breadcrumbs">
              {s.schemePath.map((p, i) => <span key={i}>{i > 0 && ' › '}<button onClick={() => s.goToScheme(i)}>{p}</button></span>)}
            </div>
          )}
          <div className="tab-body">
            {s.tab === 'scheme' && <SchemeCanvas />}
            {s.tab === 'code' && <CodeEditor />}
            {s.tab === 'model' && <ModelView />}
          </div>
        </section>
        <aside className="right"><Inspector /></aside>
        <section className="bottom"><Messages /></section>
      </div>
      <footer className="statusbar">
        <span className={`mode ${running ? 'run' : 'pause'}`}>{running ? 'Выполнение' : 'Пауза'}</span>
        <span className="mono">такт {frame?.tick ?? 0}</span>
        <span>{s.selectedClass ?? ''}</span>
        <span className="spacer" style={{ flex: 1 }} />
        <span>{s.project?.dir ?? ''}</span>
      </footer>
      {s.toast && <div className="toast">{s.toast}</div>}
    </div>
  );
}
