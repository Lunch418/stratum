import { useEffect, useRef, useState } from 'react';
import { useWindowEvent } from './hooks';
import { api } from './api';
import { useStore } from './store';
import { SchemeCanvas } from './components/SchemeCanvas';
import { CodeEditor } from './components/CodeEditor';
import { Hierarchy } from './components/Hierarchy';
import { Inspector } from './components/Inspector';
import { ModelView, handleSounds } from './components/ModelView';
import { handleHyper } from './hyper';
import { Messages } from './components/Messages';
import { PathDialog } from './components/PathDialog';
import { Graphs } from './components/Graphs';
import { Graph } from './components/Graph';
import { PictureEditor } from './components/PictureEditor';
import { Debug } from './components/Debug';
import { Help } from './components/Help';
import { Search } from './components/Search';
import { NewProjectDialog, InfoDialog } from './components/ProjectDialogs';
import { OpenDialog } from './components/OpenDialog';
import { Palette, type Command } from './components/Palette';
import { Icon } from './components/Icon';
import { MenuBar } from './components/MenuBar';
import { SheetDialog, ProjectOptionsDialog, EnvOptionsDialog, ClassPropsDialog, CalcOrderDialog, FilePickDialog, AboutDialog, DeleteClassesDialog, loadEnv } from './components/Options';
import { buildMenus, commandsFromMenus } from './menus';
import { PrintDialog } from './components/PrintDialog';
import { ChooseClassDialog } from './components/ChooseClassDialog';

export default function App() {
  const s = useStore();
  const frame = s.frame;
  const dialog = s.dialog;
  const setDialog = s.setDialog;
  const env = loadEnv();
  // «Вид → Панель инструментов / Строка состояния»
  const [view, setView] = useState<{ toolbar: boolean; statusbar: boolean }>(() => {
    try { return { toolbar: true, statusbar: true, ...JSON.parse(localStorage.getItem('view') ?? '{}') as object }; } catch { return { toolbar: true, statusbar: true }; }
  });
  const toggleView = (k: 'toolbar' | 'statusbar') => setView(v => { const n = { ...v, [k]: !v[k] }; try { localStorage.setItem('view', JSON.stringify(n)); } catch { /* приватный режим */ } return n; });
  // индикатор производительности: тактов в секунду по кадрам ядра
  const perf = useRef<{ t: number; tick: number }>({ t: 0, tick: 0 });
  const [tps, setTps] = useState(0);
  useEffect(() => {
    const id = setInterval(() => {
      const f = useStore.getState().frame, now = performance.now(), tick = f?.tick ?? 0, p = perf.current;
      setTps(f?.running && p.t ? Math.max(0, Math.round((tick - p.tick) * 1000 / (now - p.t))) : 0);
      perf.current = { t: now, tick };
    }, 1000);
    return () => clearInterval(id);
  }, []);
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
      if (tab === 'code' || tab === 'model' || tab === 'scheme' || tab === 'graph' || tab === 'picture' || tab === 'icon') s.setTab(tab);
      if (name) s.select(decodeURIComponent(name));
      if (useStore.getState().project?.empty) setDialog('open');
    }).catch(e => s.say({ level: 'error', where: 'ядро', text: String(e) }));
  }, []);
  useEffect(() => { document.documentElement.dataset.theme = s.theme; }, [s.theme]);

  // опрос кадра
  useEffect(() => {
    let alive = true;
    const poll = async () => {
      try { const f = await api.frame(); if (alive) { s.setFrame(f); handleSounds(f.sounds); handleHyper(f.hyper); } } catch { /* ядро недоступно */ }
      if (alive) setTimeout(poll, s.tab === 'model' ? env.pollMs : 250);
    };
    poll();
    return () => { alive = false; };
  }, [s.tab]);

  // горячие клавиши транспорта и переключения вкладок
  useWindowEvent('keydown', (e: KeyboardEvent) => {
      if ((e.target as HTMLElement).closest('.monaco-editor, input, textarea')) return;
      if ((e.code === 'F5' && e.shiftKey) || (e.code === 'F2' && e.ctrlKey)) { e.preventDefault(); api.event('type=reset').then(() => s.refreshInstances()); }
      else if (e.code === 'F5' || (e.code === 'F9' && e.ctrlKey)) { e.preventDefault(); api.event(frame?.running ? 'type=pause' : 'type=run'); }
      else if (e.code === 'F10' && e.shiftKey) { e.preventDefault(); api.event('type=back'); }
      else if (e.code === 'F10' || e.code === 'F7') { e.preventDefault(); api.event('type=step'); }
      else if (e.code === 'F2' && e.altKey) { e.preventDefault(); api.stateAction('keep').then(() => s.markUnsaved()); }
      else if (e.code === 'F2') { e.preventDefault(); setDialog('stateSave'); }
      else if (e.code === 'F3') { e.preventDefault(); setDialog('stateLoad'); }
      else if (e.altKey && ['Digit1', 'Digit2', 'Digit3', 'Digit4'].includes(e.code)) { e.preventDefault(); s.toggleLayer((['grid', 'images', 'links', 'graphics'] as const)[Number(e.code[5]) - 1]); }
      else if (e.key === 'Enter' && s.selectedClass && !dialog && (e.target as HTMLElement).tagName !== 'BUTTON') { e.preventDefault(); setDialog('classProps'); }
      else if (e.key.toLowerCase() === 'p' && e.ctrlKey && !e.shiftKey) { e.preventDefault(); if (s.project && !s.project.empty) setDialog('print'); }
      else if (e.key.toLowerCase() === 'o' && e.ctrlKey) { e.preventDefault(); setDialog('open'); }
      else if (e.key === 'e' && e.ctrlKey) { e.preventDefault(); s.setTab(s.tab === 'code' ? 'scheme' : 'code'); }
      else if (e.key === 's' && e.ctrlKey) { e.preventDefault(); save(); }
      else if ((e.key.toLowerCase() === 'p' && e.ctrlKey && e.shiftKey) || (e.key.toLowerCase() === 'k' && e.ctrlKey)) { e.preventDefault(); s.setPaletteOpen(true); }
      else if (e.key.toLowerCase() === 'z' && e.ctrlKey && !e.shiftKey) { e.preventDefault(); s.undo(); }
      else if ((e.key.toLowerCase() === 'z' && e.ctrlKey && e.shiftKey) || (e.key.toLowerCase() === 'y' && e.ctrlKey)) { e.preventDefault(); s.redo(); }
  });

  // Ctrl+S внутри Monaco принимает текст имиджа; проект сохраняем по Ctrl+Shift+S;
  // палитра открывается отовсюду
  useWindowEvent('keydown', (e: KeyboardEvent) => {
      if (e.key.toLowerCase() === 's' && e.ctrlKey && e.shiftKey) { e.preventDefault(); save(); }
      else if ((e.key.toLowerCase() === 'p' && e.ctrlKey && e.shiftKey) || (e.key.toLowerCase() === 'k' && e.ctrlKey)) { e.preventDefault(); s.setPaletteOpen(true); }
      else if (e.key.toLowerCase() === 'f' && e.ctrlKey && e.shiftKey) { e.preventDefault(); s.setBottomTab('search'); }
  });

  // автосохранение: проект в родном формате пишется сам через 30 с после правки
  useEffect(() => {
    if (!env.autosave) return;
    const id = setInterval(() => {
      const st = useStore.getState();
      if (st.unsaved && st.project?.native) st.saveProject().catch(() => {});
    }, env.autosave * 1000);
    return () => clearInterval(id);
  }, [env.autosave]);
  useEffect(() => { document.documentElement.style.setProperty('--font-size', env.fontSize + 'px'); }, []);

  useEffect(() => {
    const guard = (e: BeforeUnloadEvent) => { if (useStore.getState().unsaved && loadEnv().confirmClose) e.preventDefault(); };
    window.addEventListener('beforeunload', guard);
    return () => window.removeEventListener('beforeunload', guard);
  }, []);

  const running = !!frame?.running;

  const newClass = async () => { let n = 1; while (s.project?.classes.some(c => c.name.toLowerCase() === `имидж${n}`)) n++; await api.newClass(`Имидж${n}`); await s.reload(); s.select(`Имидж${n}`); s.setTab('code'); };
  const newClassOnScheme = async () => {
    const scheme = s.schemePath[s.schemePath.length - 1] ?? s.project?.root;
    if (!scheme) return;
    let n = 1; while (s.project?.classes.some(c => c.name.toLowerCase() === `имидж${n}`)) n++;
    const name = `Имидж${n}`;
    await api.newClass(name);
    const parent = s.project?.classes.find(c => c.name === scheme);
    const x = parent ? Math.max(0, ...parent.children.map(c => c.x + 140)) : 0;
    await api.addChild(scheme, name, x, 0);
    s.markUnsaved(); await s.reload(); s.select(name); s.setTab('scheme'); s.showToast(`${name} создан и поставлен на схему ${scheme}`);
  };
  const menus = buildMenus({ running, canBack: !!frame?.canBack, save, open: setDialog, newClass, newClassOnScheme, view, toggleView });
  const commands: Command[] = [
    ...commandsFromMenus(menus),
    ...(s.project?.classes ?? []).filter(c => !c.library).map(c => ({ id: 'code:' + c.name, title: `Код: ${c.name}`, group: 'имидж', run: () => { s.select(c.name); s.setTab('code'); } })),
    ...(s.project?.classes ?? []).filter(c => !c.library && (c.children.length || c.hasScheme)).map(c => ({ id: 'scheme:' + c.name, title: `Схема: ${c.name}`, group: 'имидж', run: () => { s.enterScheme(c.name); } })),
    ...(s.project?.classes ?? []).filter(c => c.library).map(c => ({ id: 'lib:' + c.name, title: `Библиотека: ${c.name}`, hint: c.description.slice(0, 40), group: 'библиотека', run: () => { s.select(c.name); s.setTab('code'); } })),
  ];

  return (
    <div className={`ide${view.toolbar ? '' : ' no-toolbar'}${view.statusbar ? '' : ' no-statusbar'}`}>
      <header className="topbar">
        <span className="brand">Stratum<span className="brand-accent">Modern</span></span>
        <MenuBar menus={menus} />
        <span className="sep" />
        <div className="transport">
        <button className={`primary icon-text${running ? ' paused' : ''}`} onClick={() => api.event(running ? 'type=pause' : 'type=run')} title="Запуск / пауза (F5)"><Icon name={running ? 'pause' : 'play'} />{running ? 'Пауза' : 'Пуск'}</button>
        <button className="icon-only" onClick={() => api.event('type=step')} title="Один шаг (F10)"><Icon name="step" /></button>
        <button className="icon-only" onClick={() => api.event('type=back')} title="Такт назад (Shift+F10)" disabled={!frame?.canBack || running}><Icon name="back" /></button>
        <button className="icon-only" onClick={() => api.event('type=reset').then(() => s.refreshInstances())} title="Стоп и сброс (Ctrl+F2)"><Icon name="stop" /></button>
        <label className="muted speed" title="Тактов в секунду">
          <input type="range" min={1} max={200} defaultValue={30} onChange={e => api.event('type=speed&fps=' + e.target.value)} />
        </label>
        <span className="counter mono">такт {frame?.tick ?? 0}{frame?.stopped ? ' · стоп' : ''}</span>
        </div>
        <span className="spacer" />
        <button className="ghost icon-only" onClick={() => setDialog('open')} title="Открыть проект (Ctrl+O)"><Icon name="open" /></button>
        <button onClick={save} title="Сохранить всё (Ctrl+S)" className={`icon-only${s.unsaved ? ' attention' : ' ghost'}`}><Icon name="save" /></button>
        <button className="ghost icon-only" onClick={s.undo} disabled={!s.project?.canUndo} title="Отмена (Ctrl+Z)"><Icon name="undo" /></button>
        <button className="ghost icon-only" onClick={s.redo} disabled={!s.project?.canRedo} title="Повтор (Ctrl+Shift+Z)"><Icon name="redo" /></button>
        <button className="ghost icon-only" onClick={() => setDialog('info')} disabled={!s.project || s.project.empty} title="Информация о проекте"><Icon name="info" /></button>
        <button className="ghost icon-only" onClick={() => s.setPaletteOpen(true)} title="Палитра команд (Ctrl+Shift+P)"><Icon name="search" /></button>
        <button className="ghost icon-only" onClick={s.toggleTheme} title={s.theme === 'light' ? 'Тёмная тема' : 'Светлая тема'}><Icon name={s.theme === 'light' ? 'moon' : 'sun'} /></button>
      </header>
      {dialog === 'open' && <OpenDialog required={!!s.project?.empty} onClose={() => setDialog(null)} />}
      {s.paletteOpen && <Palette commands={commands} onClose={() => s.setPaletteOpen(false)} />}
      {dialog === 'saveAs' && s.project && (
        <PathDialog title="Сохранить проект как" action="Сохранить" initial={s.project.native ? s.project.dir : s.project.dir.replace(/[\\/]+$/, '') + '-modern'}
          hint="Папка получит project.json и classes/ — текстовый формат Stratum Modern."
          onClose={() => setDialog(null)}
          onSubmit={dir => { setDialog(null); s.saveProject(dir).catch(e => s.say({ level: 'error', where: 'проект', text: String(e) })); }} />
      )}
      {dialog === 'new' && <NewProjectDialog onClose={() => setDialog(null)} />}
      {dialog === 'info' && <InfoDialog onClose={() => setDialog(null)} />}
      {dialog === 'sheet' && <SheetDialog onClose={() => setDialog(null)} />}
      {dialog === 'projectOptions' && <ProjectOptionsDialog onClose={() => setDialog(null)} />}
      {dialog === 'envOptions' && <EnvOptionsDialog onClose={() => setDialog(null)} />}
      {dialog === 'classProps' && <ClassPropsDialog onClose={() => setDialog(null)} />}
      {dialog === 'calcOrder' && <CalcOrderDialog onClose={() => setDialog(null)} />}
      {dialog === 'about' && <AboutDialog onClose={() => setDialog(null)} />}
      {dialog === 'print' && <PrintDialog onClose={() => setDialog(null)} />}
      {dialog === 'chooseClass' && <ChooseClassDialog onClose={() => setDialog(null)} />}
      {s.hyperProject && (
        <div className="modal-backdrop" onMouseDown={() => s.setHyperProject(null)}>
          <form className="modal" style={{ width: 460 }} onMouseDown={e => e.stopPropagation()} onSubmit={e => {
            e.preventDefault(); const p = s.hyperProject!; s.setHyperProject(null);
            s.openProject(p).catch(err => s.say({ level: 'error', where: 'гипербаза', text: String(err) }));
          }}>
            <div className="panel-title">Гиперссылка: загрузить проект</div>
            <div className="modal-body">
              <div className="mono small" style={{ wordBreak: 'break-all' }}>{s.hyperProject}</div>
              <div className="muted small" style={{ marginTop: 8 }}>{s.unsaved ? 'В текущем проекте есть несохранённые правки — они пропадут. ' : ''}Текущий проект будет закрыт.</div>
            </div>
            <div className="modal-actions">
              <button type="button" className="ghost" onClick={() => s.setHyperProject(null)}>Отмена</button>
              <button type="submit" className="primary">Открыть проект</button>
            </div>
          </form>
        </div>
      )}
      {dialog === 'exportVdr' && s.project && s.selectedClass && (
        <FilePickDialog title={`Экспорт рисунка имиджа ${s.selectedClass} в VDR`} ext="vdr" onClose={() => setDialog(null)}
          save={(s.project.dir || '.').replace(/[\\/]+$/, '') + `/${s.selectedClass.replace(/[<>:"/\\|?*]/g, '_')}.vdr`}
          onPick={async p => {
            const cls = s.selectedClass!; setDialog(null);
            const r = await fetch(`/api/vdr/save?class=${encodeURIComponent(cls)}&kind=${s.tab === 'icon' ? 'icon' : 'image'}&path=${encodeURIComponent(p)}`, { method: 'POST' });
            if (r.ok) s.showToast('Рисунок записан в ' + p); else s.say({ level: 'error', where: 'VDR', text: (await r.json()).error ?? 'ошибка записи' });
          }} />
      )}
      {dialog === 'deleteClasses' && <DeleteClassesDialog onClose={() => setDialog(null)} />}
      {dialog === 'insertFile' && s.selectedClass && (
        <FilePickDialog title="Вставить из файла в рисунок имиджа" ext="vdr,bmp" onClose={() => setDialog(null)}
          onPick={async f => {
            setDialog(null);
            const r = await fetch(`/api/picture/${encodeURIComponent(s.selectedClass!)}?kind=image`, { method: 'POST', body: JSON.stringify({ op: 'insert', file: f, x: 0, y: 0 }) });
            if (r.ok) { s.markUnsaved(); s.setTab('picture'); s.showToast('Вставлено'); } else s.say({ level: 'error', where: 'вставка', text: (await r.json()).error ?? 'ошибка' });
          }} />
      )}
      {(dialog === 'imageSave' || dialog === 'imageLoad') && s.project && s.selectedClass && (
        <FilePickDialog title={`${dialog === 'imageSave' ? 'Сохранить' : 'Прочитать'} переменные имиджа ${s.selectedClass}`} ext="stt" onClose={() => setDialog(null)}
          save={dialog === 'imageSave' ? (s.project.dir || '.').replace(/[\\/]+$/, '') + `/${s.selectedClass}.stt` : undefined}
          onPick={p => { const cls = s.selectedClass!; setDialog(null); api.stateAction(dialog === 'imageSave' ? 'save' : 'load', p, cls).then(r => s.showToast(`${dialog === 'imageSave' ? 'Сохранено' : 'Загружено'}: ${r.images} экземпляров`)).catch(e => s.say({ level: 'error', where: 'состояние', text: String(e) })); }} />
      )}
      {(dialog === 'stateSave' || dialog === 'stateLoad') && s.project && (
        <PathDialog title={dialog === 'stateSave' ? 'Сохранить состояние' : 'Загрузить состояние'} action={dialog === 'stateSave' ? 'Сохранить' : 'Загрузить'}
          initial={(s.project.dir || '.').replace(/[\\/]+$/, '') + '/state.stt'}
          hint="Файл .stt в новой редакции (переменные по именам), как _preload.stt."
          onClose={() => setDialog(null)}
          onSubmit={p => { setDialog(null); api.stateAction(dialog === 'stateSave' ? 'save' : 'load', p).then(r => s.showToast(`${dialog === 'stateSave' ? 'Сохранено' : 'Загружено'}: ${r.images} имиджей`)).catch(e => s.say({ level: 'error', where: 'состояние', text: String(e) })); }} />
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
            <button className={s.tab === 'picture' ? 'active' : ''} onClick={() => s.setTab('picture')}>Рисунок{s.selectedClass ? ` · ${s.selectedClass}` : ''}</button>
            <button className={s.tab === 'icon' ? 'active' : ''} onClick={() => s.setTab('icon')}>Иконка</button>
            <button className={s.tab === 'graph' ? 'active' : ''} onClick={() => s.setTab('graph')}>Граф</button>
          </div>
          {frame?.halt && (
            <div className={`halt ${frame.halt.kind}`}>
              <span>{frame.halt.kind === 'error' ? 'Ошибка' : frame.halt.kind === 'warning' ? 'Предупреждение' : frame.halt.kind === 'math' ? 'Математическая ошибка' : 'Остановлено'}: {frame.halt.message}{frame.halt.line ? ` (строка ${frame.halt.line})` : ''}</span>
              <span className="spacer" />
              {frame.halt.class && <button className="small" onClick={() => { s.select(frame.halt!.class, frame.halt!.instance); s.setTab('code'); }}>К коду</button>}
              {frame.halt.kind === 'error' && <button className="small" onClick={() => api.event('type=back')} disabled={!frame.canBack}>Такт назад</button>}
              {frame.halt.kind === 'math' && <>
                <button className="small" onClick={() => api.event('type=run')} title="Модель продолжит с подставленным значением, как «Пропустить» в оригинале">Продолжить</button>
                <button className="small" onClick={() => api.event('type=ignoremath')}>Больше не замечать</button>
              </>}
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
            {s.tab === 'graph' && <Graph />}
            {s.tab === 'picture' && <PictureEditor kind="image" />}
            {s.tab === 'icon' && <PictureEditor kind="icon" />}
          </div>
        </section>
        <aside className="right">
          <div className="right-top"><Inspector /></div>
          <div className="right-bottom"><Help /></div>
        </aside>
        <section className="bottom">
          <div className="tabs small-tabs">
            <button className={s.bottomTab === 'messages' ? 'active' : ''} onClick={() => s.setBottomTab('messages')}>Сообщения</button>
            <button className={s.bottomTab === 'graphs' ? 'active' : ''} onClick={() => s.setBottomTab('graphs')}>Графики{s.traceCount ? ` (${s.traceCount})` : ''}</button>
            <button className={s.bottomTab === 'debug' ? 'active' : ''} onClick={() => s.setBottomTab('debug')}>Отладка</button>
            <button className={s.bottomTab === 'search' ? 'active' : ''} onClick={() => s.setBottomTab('search')}>Поиск</button>
          </div>
          <div className="tab-body" style={{ display: 'flex', flexDirection: 'column' }}>
            {s.bottomTab === 'messages' ? <Messages /> : s.bottomTab === 'graphs' ? <Graphs /> : s.bottomTab === 'debug' ? <Debug /> : <Search />}
          </div>
        </section>
      </div>
      <footer className="statusbar">
        <span className={`mode ${running ? 'run' : 'pause'}`}>{running ? 'Выполнение' : 'Пауза'}</span>
        {env.statusTicks && <span className="mono">такт {frame?.tick ?? 0}</span>}
        {env.statusPerf && <span className="mono" title="Тактов в секунду">{tps} т/с</span>}
        <span>{s.selectedClass ?? ''}</span>
        <span className="spacer" style={{ flex: 1 }} />
        <span>{s.project?.dir ?? ''}</span>
      </footer>
      {s.toast && <div className="toast">{s.toast}</div>}
    </div>
  );
}
