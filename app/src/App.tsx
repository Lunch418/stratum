import { useEffect, useState } from 'react';
import { api } from './api';
import { useStore } from './store';
import { SchemeCanvas } from './components/SchemeCanvas';
import { CodeEditor } from './components/CodeEditor';
import { Hierarchy } from './components/Hierarchy';
import { Inspector } from './components/Inspector';
import { ModelView } from './components/ModelView';
import { Messages } from './components/Messages';
import { PathDialog } from './components/PathDialog';
import { Graphs } from './components/Graphs';
import { Debug } from './components/Debug';
import { Palette, type Command } from './components/Palette';

export default function App() {
  const s = useStore();
  const frame = s.frame;
  const [dialog, setDialog] = useState<'saveAs' | 'export' | null>(null);
  const [layout, setLayout] = useState<{ left: number; right: number; bottom: number }>(() => {
    const def = { left: 260, right: 300, bottom: 160 };
    try { return { ...def, ...JSON.parse(localStorage.getItem('layout') ?? '{}') as Partial<typeof def> }; }
    catch { return def; }
  });
  const [dragging, setDragging] = useState<'left' | 'right' | 'bottom' | null>(null);
  useEffect(() => { try { localStorage.setItem('layout', JSON.stringify(layout)); } catch { /* приватный режим */ } }, [layout]);
  useEffect(() => {
    if (!dragging) return;
    const ws = document.querySelector('.workspace') as HTMLElement | null;
    const onMove = (e: MouseEvent) => {
      if (!ws) return;
      const r = ws.getBoundingClientRect();
      setLayout(l => dragging === 'left' ? { ...l, left: Math.max(160, Math.min(r.width - l.right - 300, e.clientX - r.left)) }
        : dragging === 'right' ? { ...l, right: Math.max(200, Math.min(r.width - l.left - 300, r.right - e.clientX)) }
        : { ...l, bottom: Math.max(80, Math.min(r.height - 160, r.bottom - e.clientY)) });
    };
    const onUp = () => setDragging(null);
    window.addEventListener('mousemove', onMove); window.addEventListener('mouseup', onUp);
    document.body.style.cursor = dragging === 'bottom' ? 'row-resize' : 'col-resize';
    document.body.style.userSelect = 'none';
    return () => { window.removeEventListener('mousemove', onMove); window.removeEventListener('mouseup', onUp); document.body.style.cursor = ''; document.body.style.userSelect = ''; };
  }, [dragging]);

  // Сохранить: проект в родном формате уже на диске — в его же папку,
  // проект Stratum 2000 — спросить новую папку (исходник не трогаем)
  const save = () => {
    if (!s.project) return;
    if (s.project.native) s.saveProject().catch(e => s.say({ level: 'error', where: 'проект', text: String(e) }));
    else setDialog('saveAs');
  };

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
      else if (e.code === 'F10' && e.shiftKey) { e.preventDefault(); api.event('type=back'); }
      else if (e.code === 'F10') { e.preventDefault(); api.event('type=step'); }
      else if (e.key === 'e' && e.ctrlKey) { e.preventDefault(); s.setTab(s.tab === 'code' ? 'scheme' : 'code'); }
      else if (e.key === 's' && e.ctrlKey) { e.preventDefault(); save(); }
      else if ((e.key.toLowerCase() === 'p' && e.ctrlKey && e.shiftKey) || (e.key.toLowerCase() === 'k' && e.ctrlKey)) { e.preventDefault(); s.setPaletteOpen(true); }
      else if (e.key.toLowerCase() === 'z' && e.ctrlKey && !e.shiftKey) { e.preventDefault(); s.undo(); }
      else if ((e.key.toLowerCase() === 'z' && e.ctrlKey && e.shiftKey) || (e.key.toLowerCase() === 'y' && e.ctrlKey)) { e.preventDefault(); s.redo(); }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [frame?.running, s.tab, s.project?.native]);

  // Ctrl+S внутри Monaco принимает текст имиджа; проект сохраняем по Ctrl+Shift+S;
  // палитра открывается отовсюду
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key.toLowerCase() === 's' && e.ctrlKey && e.shiftKey) { e.preventDefault(); save(); }
      else if ((e.key.toLowerCase() === 'p' && e.ctrlKey && e.shiftKey) || (e.key.toLowerCase() === 'k' && e.ctrlKey)) { e.preventDefault(); s.setPaletteOpen(true); }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [s.project?.native]);

  useEffect(() => {
    const guard = (e: BeforeUnloadEvent) => { if (useStore.getState().unsaved) e.preventDefault(); };
    window.addEventListener('beforeunload', guard);
    return () => window.removeEventListener('beforeunload', guard);
  }, []);

  const running = !!frame?.running;

  const commands: Command[] = [
    { id: 'run', title: running ? 'Пауза' : 'Пуск', hint: 'F5', group: 'модель', run: () => api.event(running ? 'type=pause' : 'type=run') },
    { id: 'step', title: 'Шаг', hint: 'F10', group: 'модель', run: () => api.event('type=step') },
    { id: 'back', title: 'Такт назад', hint: 'Shift+F10', group: 'модель', run: () => api.event('type=back') },
    { id: 'bottom-debug', title: 'Панель: Отладка', group: 'вид', run: () => s.setBottomTab('debug') },
    { id: 'reset', title: 'Сброс', hint: 'Shift+F5', group: 'модель', run: () => api.event('type=reset').then(() => s.refreshInstances()) },
    { id: 'save', title: 'Сохранить проект', hint: 'Ctrl+S', group: 'проект', run: save },
    { id: 'saveas', title: 'Сохранить проект как…', group: 'проект', run: () => setDialog('saveAs') },
    { id: 'export', title: 'Экспорт в Stratum 2000…', group: 'проект', run: () => setDialog('export') },
    { id: 'undo', title: 'Отменить', hint: 'Ctrl+Z', group: 'правка', run: s.undo },
    { id: 'redo', title: 'Повторить', hint: 'Ctrl+Shift+Z', group: 'правка', run: s.redo },
    { id: 'newclass', title: 'Новый имидж', group: 'правка', run: async () => { let n = 1; while (s.project?.classes.some(c => c.name.toLowerCase() === `имидж${n}`)) n++; await api.newClass(`Имидж${n}`); await s.reload(); s.select(`Имидж${n}`); s.setTab('code'); } },
    { id: 'tab-scheme', title: 'Вкладка: Схема', group: 'вид', run: () => s.setTab('scheme') },
    { id: 'tab-code', title: 'Вкладка: Код', hint: 'Ctrl+E', group: 'вид', run: () => s.setTab('code') },
    { id: 'tab-model', title: 'Вкладка: Окно модели', group: 'вид', run: () => s.setTab('model') },
    { id: 'bottom-graphs', title: 'Панель: Графики', group: 'вид', run: () => s.setBottomTab('graphs') },
    { id: 'bottom-messages', title: 'Панель: Сообщения', group: 'вид', run: () => s.setBottomTab('messages') },
    { id: 'theme', title: s.theme === 'light' ? 'Тёмная тема' : 'Светлая тема', group: 'вид', run: s.toggleTheme },
    { id: 'root', title: 'Схема: к корню', group: 'схема', run: () => { s.goToScheme(0); s.setTab('scheme'); } },
    ...(s.project?.classes ?? []).filter(c => !c.library).map(c => ({ id: 'code:' + c.name, title: `Код: ${c.name}`, group: 'имидж', run: () => { s.select(c.name); s.setTab('code'); } })),
    ...(s.project?.classes ?? []).filter(c => !c.library && (c.children.length || c.hasScheme)).map(c => ({ id: 'scheme:' + c.name, title: `Схема: ${c.name}`, group: 'имидж', run: () => { s.enterScheme(c.name); } })),
    ...(s.project?.classes ?? []).filter(c => c.library).map(c => ({ id: 'lib:' + c.name, title: `Библиотека: ${c.name}`, hint: c.description.slice(0, 40), group: 'библиотека', run: () => { s.select(c.name); s.setTab('code'); } })),
  ];

  return (
    <div className="ide">
      <header className="topbar">
        <span className="brand">Stratum Modern</span>
        <button className={`primary${running ? ' paused' : ''}`} onClick={() => api.event(running ? 'type=pause' : 'type=run')} title="F5">{running ? 'Пауза' : 'Пуск'}</button>
        <button onClick={() => api.event('type=step')} title="F10">Шаг</button>
        <button onClick={() => api.event('type=back')} title="Shift+F10 — такт назад по истории" disabled={!frame?.canBack || running}>Назад</button>
        <button onClick={() => api.event('type=reset').then(() => s.refreshInstances())} title="Shift+F5">Сброс</button>
        <label className="muted" style={{ display: 'flex', alignItems: 'center', gap: 6 }}>Скорость
          <input type="range" min={1} max={200} defaultValue={30} onChange={e => api.event('type=speed&fps=' + e.target.value)} />
        </label>
        <span className="counter mono">такт {frame?.tick ?? 0}{frame?.stopped ? ' · остановлено' : ''}</span>
        <span className="sep" />
        <button className="ghost" onClick={s.undo} disabled={!s.project?.canUndo} title="Ctrl+Z">Отменить</button>
        <button className="ghost" onClick={s.redo} disabled={!s.project?.canRedo} title="Ctrl+Shift+Z">Повторить</button>
        <button onClick={save} title="Ctrl+S — сохранить проект в текстовом формате" className={s.unsaved ? 'attention' : ''}>Сохранить{s.unsaved ? ' •' : ''}</button>
        <button className="ghost" onClick={() => setDialog('saveAs')} title="Сохранить копию проекта в другую папку">Сохранить как…</button>
        <button className="ghost" onClick={() => setDialog('export')} title="Записать project.spj и .cls для Stratum 2000">Экспорт…</button>
        <button className="ghost" onClick={() => s.setPaletteOpen(true)} title="Ctrl+Shift+P">Команды</button>
        <button className="ghost" onClick={s.toggleTheme} title="Тема">{s.theme === 'light' ? 'Тёмная' : 'Светлая'}</button>
      </header>
      {s.paletteOpen && <Palette commands={commands} onClose={() => s.setPaletteOpen(false)} />}
      {dialog === 'saveAs' && s.project && (
        <PathDialog title="Сохранить проект как" action="Сохранить" initial={s.project.native ? s.project.dir : s.project.dir.replace(/[\\/]+$/, '') + '-modern'}
          hint="Папка получит project.json и classes/ — текстовый формат Stratum Modern."
          onClose={() => setDialog(null)}
          onSubmit={dir => { setDialog(null); s.saveProject(dir).catch(e => s.say({ level: 'error', where: 'проект', text: String(e) })); }} />
      )}
      {dialog === 'export' && s.project && (
        <PathDialog title="Экспорт в Stratum 2000" action="Экспортировать" initial={s.project.dir.replace(/[\\/]+$/, '') + '-export'}
          hint="В папку будут записаны project.spj, _preload.stt и файлы .cls."
          onClose={() => setDialog(null)}
          onSubmit={dir => { setDialog(null); api.exportProject(dir).then(r => { s.showToast('Экспортировано'); s.say({ level: 'info', where: 'проект', text: `экспорт: ${r.classes} имиджей → ${r.dir}` }); }).catch(e => s.say({ level: 'error', where: 'экспорт', text: String(e) })); }} />
      )}
      <div className="workspace" style={{ '--left-w': layout.left + 'px', '--right-w': layout.right + 'px', '--bottom-h': layout.bottom + 'px' } as React.CSSProperties}>
        <div className={`splitter v${dragging === 'left' ? ' active' : ''}`} style={{ left: layout.left }} onMouseDown={() => setDragging('left')} onDoubleClick={() => setLayout(l => ({ ...l, left: 260 }))} />
        <div className={`splitter v${dragging === 'right' ? ' active' : ''}`} style={{ right: layout.right, marginLeft: 0, marginRight: -3 }} onMouseDown={() => setDragging('right')} onDoubleClick={() => setLayout(l => ({ ...l, right: 300 }))} />
        <div className={`splitter h${dragging === 'bottom' ? ' active' : ''}`} style={{ bottom: layout.bottom }} onMouseDown={() => setDragging('bottom')} onDoubleClick={() => setLayout(l => ({ ...l, bottom: 160 }))} />
        <aside className="left"><Hierarchy /></aside>
        <section className="center">
          <div className="tabs">
            <button className={s.tab === 'scheme' ? 'active' : ''} onClick={() => s.setTab('scheme')}>Схема</button>
            <button className={s.tab === 'code' ? 'active' : ''} onClick={() => s.setTab('code')}>Код{s.selectedClass ? ` · ${s.selectedClass}` : ''}</button>
            <button className={s.tab === 'model' ? 'active' : ''} onClick={() => s.setTab('model')}>Окно модели{frame?.windows.length ? ` (${frame.windows.length})` : ''}</button>
          </div>
          {frame?.halt && (
            <div className={`halt ${frame.halt.kind}`}>
              <span>{frame.halt.kind === 'error' ? 'Ошибка' : 'Остановлено'}: {frame.halt.message}{frame.halt.line ? ` (строка ${frame.halt.line})` : ''}</span>
              <span className="spacer" />
              {frame.halt.class && <button className="small" onClick={() => { s.select(frame.halt!.class, frame.halt!.instance); s.setTab('code'); }}>К коду</button>}
              {frame.halt.kind === 'error' && <button className="small" onClick={() => api.event('type=back')} disabled={!frame.canBack}>Такт назад</button>}
            </div>
          )}
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
        <section className="bottom">
          <div className="tabs small-tabs">
            <button className={s.bottomTab === 'messages' ? 'active' : ''} onClick={() => s.setBottomTab('messages')}>Сообщения</button>
            <button className={s.bottomTab === 'graphs' ? 'active' : ''} onClick={() => s.setBottomTab('graphs')}>Графики{s.traceCount ? ` (${s.traceCount})` : ''}</button>
            <button className={s.bottomTab === 'debug' ? 'active' : ''} onClick={() => s.setBottomTab('debug')}>Отладка</button>
          </div>
          <div className="tab-body" style={{ display: 'flex', flexDirection: 'column' }}>
            {s.bottomTab === 'messages' ? <Messages /> : s.bottomTab === 'graphs' ? <Graphs /> : <Debug />}
          </div>
        </section>
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
