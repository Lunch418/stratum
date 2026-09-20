// Главное меню в структуре оригинала (ресурс MENU 100 из r1251s32.dll,
// см. docs/original/menus.md). Каждый пункт оригинала либо действует, либо
// помечен причиной, почему в новой среде он не нужен. Из тех же пунктов
// собирается палитра команд.
import { api } from './api';
import type { Menu, MenuItem } from './components/MenuBar';
import type { Command } from './components/Palette';
import { useStore, type DialogId } from './store';

export interface MenuContext {
  running: boolean;
  canBack: boolean;
  save: () => void;
  open: (d: DialogId) => void;
  newClass: () => void;
}

const MDI = 'В новой среде вместо окон MDI — панели с разделителями';

export function buildMenus(ctx: MenuContext): Menu[] {
  const s = useStore.getState();
  const cls = s.selectedClass;
  const lib = !!s.project?.classes.find(c => c.name === cls)?.library;
  const noProject = !s.project || s.project.empty;
  const st = (action: 'default' | 'keep') => api.stateAction(action).then(() => { if (action === 'keep') s.markUnsaved(); s.showToast(action === 'default' ? 'Переменные — по умолчанию' : 'Текущее состояние стало стартовым'); });
  const tab = (t: typeof s.tab, label: string): MenuItem => ({ label, checked: s.tab === t, run: () => s.setTab(t) });
  const bottom = (t: typeof s.bottomTab, label: string): MenuItem => ({ label, checked: s.bottomTab === t, run: () => s.setBottomTab(t) });
  const drawTool = (label: string, hint?: string): MenuItem => ({ label, hint, run: () => { s.setTab('picture'); s.showToast('Инструмент — на панели над листом'); } });

  return [
    { title: 'Файл', items: [
      { label: 'Новый', sub: [
        { label: 'Проект…', run: () => ctx.open('new') },
        { label: 'Имидж…', run: ctx.newClass, disabled: noProject },
        { label: 'Векторный рисунок', run: () => s.setTab('picture'), disabled: noProject },
      ] },
      { label: 'Открыть…', hint: 'Ctrl+O', run: () => ctx.open('open') },
      { sep: true },
      { label: 'Сохранить всё', hint: 'Ctrl+S', run: ctx.save, disabled: noProject },
      { label: 'Сохранить проект как…', run: () => ctx.open('saveAs'), disabled: noProject },
      { label: 'Экспорт в Stratum 2000…', run: () => ctx.open('export'), disabled: noProject },
      { label: 'Закрыть проект', run: () => api.newProject().then(() => s.load()), disabled: noProject },
      { sep: true },
      { label: 'Печать…', hint: 'Ctrl+P', run: () => window.print(), disabled: noProject },
      { sep: true },
      { label: 'Выход', run: () => window.close(), why: 'В браузере вкладка закрывается сама' },
    ] },
    { title: 'Правка', items: [
      { label: 'Отмена', hint: 'Ctrl+Z', run: s.undo, disabled: !s.project?.canUndo },
      { label: 'Повтор', hint: 'Ctrl+Shift+Z', run: s.redo, disabled: !s.project?.canRedo },
      { sep: true },
      { label: 'Вырезать', hint: 'Ctrl+X', run: () => document.execCommand('cut'), disabled: s.tab !== 'scheme' && s.tab !== 'code', why: 'Действует на схеме и в коде' },
      { label: 'Копировать', hint: 'Ctrl+C', run: () => document.execCommand('copy') },
      { label: 'Вставить', hint: 'Ctrl+V', run: () => document.execCommand('paste') },
      { label: 'Дублировать', hint: 'Ctrl+D', run: () => s.showToast('Выберите блок на схеме или объект рисунка и нажмите Ctrl+D') },
      { sep: true },
      { label: 'Удалить', hint: 'Delete', run: () => s.showToast('Выберите блок, связь или объект и нажмите Delete') },
      { sep: true },
      { label: 'Поиск', hint: 'Ctrl+F', run: () => { s.setTab('code'); s.showToast('Ctrl+F в редакторе кода'); } },
      { label: 'Замена', hint: 'Ctrl+H', run: () => { s.setTab('code'); s.showToast('Ctrl+H в редакторе кода'); } },
      { label: 'Поиск по проекту', hint: 'Ctrl+Shift+F', run: () => s.setBottomTab('search') },
    ] },
    { title: 'Вид', items: [
      { label: 'Панель инструментов', disabled: true, why: 'Панели закреплены; их ширину меняют разделители' },
      { label: 'Строка состояния', checked: true, disabled: true, why: 'Строка состояния всегда внизу' },
      { sep: true },
      { label: 'Библиотеки', checked: true, run: () => s.showToast('Библиотеки — раздел в иерархии слева') },
      bottom('messages', 'Сообщения'),
      { sep: true },
      { label: 'Главная схема', run: () => { s.goToScheme(0); s.setTab('scheme'); } },
      tab('code', 'Исходный текст'),
      tab('model', 'Окно модели'),
      tab('picture', 'Рисунок имиджа'),
      tab('icon', 'Иконка имиджа'),
      tab('graph', 'Граф зависимостей'),
      { sep: true },
      { label: 'Иерархия проекта', checked: true, disabled: true, why: 'Иерархия всегда слева' },
      { label: 'Просмотр переменных', run: () => s.showToast('Переменные — в инспекторе справа; графики — внизу') },
      bottom('graphs', 'Графики'),
      bottom('debug', 'Отладка'),
      { sep: true },
      { label: 'Информация…', run: () => ctx.open('info'), disabled: noProject },
      { label: s.theme === 'light' ? 'Тёмная тема' : 'Светлая тема', run: s.toggleTheme },
    ] },
    { title: 'Вставка', items: [
      { label: 'Имидж…', hint: 'перетащить из иерархии', run: () => { s.setTab('scheme'); s.showToast('Перетащите имидж из иерархии на схему'); } },
      { label: 'Связь', run: () => { s.setTab('scheme'); s.showToast('Потяните от порта блока к другому блоку'); } },
      { label: 'Создать и вставить новый имидж…', run: ctx.newClass, disabled: noProject },
      { label: 'Контактная площадка', disabled: true, why: 'Не реализовано: контактные площадки схемы' },
      { sep: true },
      { label: 'Новый двухмерный объект', sub: [
        drawTool('Линия'), drawTool('Полилиния', 'Ctrl+P'), drawTool('Прямоугольник', 'Ctrl+B'), drawTool('Скруглённый прямоугольник', 'Ctrl+U'), drawTool('Эллипс', 'Ctrl+E'),
        { sep: true },
        drawTool('Текст', 'Ctrl+T'),
        { label: 'Битовая карта', run: () => { s.setTab('picture'); s.showToast('Кнопка «Вставить из файла» на панели рисования'); } },
        { label: 'Двойная битовая карта', disabled: true, why: 'Растры с маской вставляются функцией CreateDoubleBitmap2d' },
        { label: 'Проекция 3d пространства', disabled: true, why: 'Создаётся функцией CreateView3d' },
        drawTool('Группа', 'кнопка «Группа»'),
      ] },
      { label: 'Формы', sub: [
        { label: 'Строка ввода (Edit)', disabled: true, why: 'Контролы создаются в коде: CreateControl2d' },
        { label: 'Комбо-бокс', disabled: true, why: 'CreateControl2d' },
        { label: 'Чек-бокс', disabled: true, why: 'CreateControl2d' },
        { label: 'Радиокнопка', disabled: true, why: 'CreateControl2d' },
        { label: 'Кнопка', disabled: true, why: 'CreateControl2d' },
        { label: 'Список', disabled: true, why: 'CreateControl2d' },
      ] },
      { label: '3d', sub: [
        { label: 'Создать новую камеру', disabled: true, why: 'CreateCamera3d в коде' },
        { label: 'Дублировать камеру', disabled: true, why: 'CreateCamera3d в коде' },
      ] },
      { sep: true },
      { label: 'OLE объект', disabled: true, why: 'OLE — технология Windows 98' },
      { label: 'Из файла…', run: () => ctx.open('insertFile'), disabled: noProject || lib },
    ] },
    { title: 'Формат', items: [
      { label: 'Свойства имиджа…', hint: 'Enter', run: () => ctx.open('classProps'), disabled: !cls },
      { label: 'Актуальные размеры', hint: 'Ctrl+A', run: () => s.showToast('Масштаб 100 % — кнопка процентов на панели рисования') },
      { sep: true },
      { label: 'Добавить в группу', run: () => { s.setTab('picture'); s.showToast('Выделите объекты и нажмите «Группа»'); } },
      { label: 'Разгруппировать', run: () => { s.setTab('picture'); s.showToast('Выделите группу и нажмите «Разгруппировать»'); } },
      { sep: true },
      { label: 'Редактировать схему…', run: () => { if (cls) s.enterScheme(cls); }, disabled: !cls },
      { label: 'Редактировать текст…', run: () => s.setTab('code'), disabled: !cls },
      { label: 'Редактировать изображение…', run: () => s.setTab('picture'), disabled: !cls },
      { sep: true },
      { label: 'Z-порядок', sub: [
        { label: 'На верх', hint: 'Ctrl+PgUp', run: () => window.dispatchEvent(new CustomEvent('zorder', { detail: 'top' })) },
        { label: 'Поместить назад', hint: 'Ctrl+PgDn', run: () => window.dispatchEvent(new CustomEvent('zorder', { detail: 'bottom' })) },
        { label: 'Приблизить на одну', hint: 'PgUp', run: () => window.dispatchEvent(new CustomEvent('zorder', { detail: 'up' })) },
        { label: 'Отодвинуть на одну', hint: 'PgDn', run: () => window.dispatchEvent(new CustomEvent('zorder', { detail: 'down' })) },
      ] },
      { label: 'Слои', sub: [
        { label: 'Сетка', hint: 'Alt+1', checked: s.layers.grid, run: () => s.toggleLayer('grid') },
        { label: 'Имиджи', hint: 'Alt+2', checked: s.layers.images, run: () => s.toggleLayer('images') },
        { label: 'Связи', hint: 'Alt+3', checked: s.layers.links, run: () => s.toggleLayer('links') },
        { label: 'Другие 2d объекты', hint: 'Alt+4', checked: s.layers.graphics, run: () => s.toggleLayer('graphics') },
        { sep: true },
        { label: 'Произвольно…', run: () => ctx.open('sheet'), disabled: !cls },
      ] },
      { sep: true },
      { label: 'Параметры листа…', run: () => ctx.open('sheet'), disabled: !cls || lib },
      { label: 'Редактировать порядок вычислений…', run: () => ctx.open('calcOrder'), disabled: !cls || lib },
    ] },
    { title: 'Моделирование', items: [
      { label: ctx.running ? 'Пауза' : 'Запуск', hint: 'F5 · Ctrl+F9', run: () => api.event(ctx.running ? 'type=pause' : 'type=run') },
      { label: 'Один шаг', hint: 'F10 · F7', run: () => api.event('type=step') },
      { label: 'Такт назад', hint: 'Shift+F10', run: () => api.event('type=back'), disabled: !ctx.canBack || ctx.running },
      { label: 'Стоп (сброс)', hint: 'Ctrl+F2 · Shift+F5', run: () => api.event('type=reset').then(() => s.refreshInstances()) },
      { sep: true },
      { label: 'Очистить', sub: [
        { label: 'Всё очистить и остановить', hint: 'Ctrl+F2', run: () => api.event('type=reset').then(() => s.refreshInstances()) },
        { sep: true },
        { label: 'Обнулить переменные', run: () => st('default') },
        { label: 'Закрыть потоки', disabled: true, why: 'Потоки закрываются при сбросе' },
        { label: 'Закрыть окна', disabled: true, why: 'Окна модели закрываются при сбросе' },
        { label: 'Закрыть все базы', disabled: true, why: 'Базы данных не поддерживаются' },
      ] },
      { label: 'По умолчанию', run: () => st('default') },
      { label: 'Установить переменные…', run: () => s.showToast('Значения правятся в инспекторе справа на паузе') },
      { sep: true },
      { label: 'Сохранить переменные…', hint: 'F2', run: () => ctx.open('stateSave'), disabled: noProject },
      { label: 'Загрузить переменные…', hint: 'F3', run: () => ctx.open('stateLoad'), disabled: noProject },
      { label: 'Запомнить как стартовое состояние', hint: 'Alt+F2', run: () => st('keep'), disabled: noProject },
      { label: 'Вернуть стартовое состояние', hint: 'Alt+F3', run: () => api.event('type=reset').then(() => s.refreshInstances()), disabled: noProject },
      { sep: true },
      { label: 'Сохранить переменные имиджа…', run: () => ctx.open('imageSave'), disabled: !cls },
      { label: 'Прочитать переменные имиджа…', run: () => ctx.open('imageLoad'), disabled: !cls },
      { label: 'Установить по умолчанию', run: () => st('keep'), disabled: noProject },
    ] },
    { title: 'Параметры', items: [
      { label: 'Параметры среды…', run: () => ctx.open('envOptions') },
      { label: 'Параметры проекта…', run: () => ctx.open('projectOptions'), disabled: noProject },
      { sep: true },
      { label: 'Дополнительные модули…', disabled: true, why: 'Плагины Ogre3D, видео и БД не переносятся' },
      { label: 'Database Explorer…', disabled: true, why: 'Базы данных DBF не поддерживаются' },
    ] },
    { title: 'Окна', items: [
      { label: 'Каскадом', disabled: true, why: MDI },
      { label: 'Черепица горизонтально', disabled: true, why: MDI },
      { label: 'Черепица вертикально', disabled: true, why: MDI },
      { label: 'Упорядочить иконки', disabled: true, why: MDI },
      { sep: true },
      { label: 'Раскладка панелей по умолчанию', run: () => { try { localStorage.removeItem('layout'); } catch { /* */ } location.reload(); } },
      { label: 'Палитра команд…', hint: 'Ctrl+Shift+P', run: () => s.setPaletteOpen(true) },
    ] },
    { title: 'Помощь', items: [
      { label: 'Содержание', hint: 'F1', run: () => s.setHelpTopic('') },
      { label: 'Клавиатура', run: () => s.setHelpTopic('клавиатура') },
      { label: 'Функция под курсором', hint: 'F1 в коде', run: () => s.setTab('code') },
      { sep: true },
      { label: 'Системные библиотеки', run: () => s.setHelpTopic('библиотеки') },
      { label: 'Исходный код на GitHub', run: () => window.open('https://github.com/Lunch418/stratum', '_blank') },
      { sep: true },
      { label: 'О программе…', run: () => ctx.open('about') },
    ] },
  ];
}

/// Палитра команд — плоский список тех же пунктов.
export function commandsFromMenus(menus: Menu[]): Command[] {
  const out: Command[] = [];
  const walk = (items: MenuItem[], group: string, prefix: string) => {
    for (const it of items) {
      if (it.sep || !it.label) continue;
      if (it.sub) { walk(it.sub, group, prefix + it.label + ' › '); continue; }
      if (it.run && !it.disabled) out.push({ id: group + ':' + prefix + it.label, title: prefix + it.label, hint: it.hint, group, run: it.run });
    }
  };
  for (const m of menus) walk(m.items, m.title.toLowerCase(), '');
  return out;
}
