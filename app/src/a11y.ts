// Общее поведение модальных окон, чтобы каждый диалог не повторял его сам:
// роль и заголовок для экранного диктора, фокус внутрь при открытии и назад
// при закрытии, Tab не уходит за пределы окна, Escape закрывает верхнее окно
// так же, как щелчок по подложке (окна модели подложку не закрывают — им
// Escape тоже ничего не сделает).
const FOCUSABLE = 'input:not([disabled]):not([type="hidden"]), select:not([disabled]), textarea:not([disabled]), button:not([disabled]), [href], [tabindex]:not([tabindex="-1"])';

let seq = 0;
const returnTo = new WeakMap<Element, HTMLElement | null>();

function topBackdrop(): HTMLElement | null {
  const all = document.querySelectorAll<HTMLElement>('.modal-backdrop');
  return all.length ? all[all.length - 1] : null;
}

const visible = (el: HTMLElement) => el.offsetParent !== null || el.getClientRects().length > 0;
const focusables = (root: HTMLElement) => [...root.querySelectorAll<HTMLElement>(FOCUSABLE)].filter(visible);

function prepare(backdrop: HTMLElement) {
  const modal = backdrop.querySelector<HTMLElement>('.modal') ?? backdrop;
  if (modal.dataset.a11y) return;
  modal.dataset.a11y = '1';
  modal.setAttribute('role', 'dialog');
  modal.setAttribute('aria-modal', 'true');
  const title = modal.querySelector<HTMLElement>(':scope > .panel-title');
  if (title) {
    title.id ||= `dlg-title-${++seq}`;
    modal.setAttribute('aria-labelledby', title.id);
  }
  returnTo.set(backdrop, document.activeElement instanceof HTMLElement ? document.activeElement : null);
  // autoFocus диалога уже сработал — не перебиваем его
  if (modal.contains(document.activeElement)) return;
  const list = focusables(modal);
  // первое поле ввода, если диалог с него начинается; иначе — главная кнопка
  const first = list.find(el => !el.closest('.panel-title, .tabs'));
  const field = first?.matches('input[type="text"], input[type="number"], input[type="search"], input:not([type]), textarea') ? first : undefined;
  const target = field ??modal.querySelector<HTMLElement>('.modal-actions button[type="submit"], .modal-actions .primary') ?? [...modal.querySelectorAll<HTMLElement>('.modal-actions button')].pop() ?? list[0];
  target?.focus({ preventScroll: true });
}

// Деревья и списки строк (иерархия, библиотеки, справка, папки): в каждую
// панель входят одной остановкой Tab, дальше ↑/↓, Home/End — по строкам,
// пробел — выбрать (как щелчок); Enter оставлен за «Свойствами имиджа».
function rovingTrees() {
  for (const row of document.querySelectorAll<HTMLElement>('.tree-row:not([tabindex])')) row.tabIndex = -1;
  for (const panel of document.querySelectorAll<HTMLElement>('.left, .right-bottom, .tree.browse')) {
    const rows = panel.querySelectorAll<HTMLElement>('.tree-row');
    if (rows.length && ![...rows].some(r => r.tabIndex === 0)) rows[0].tabIndex = 0;
  }
}

function treeKeys(e: KeyboardEvent) {
  const row = (e.target as HTMLElement | null)?.closest?.('.tree-row') as HTMLElement | null;
  if (!row || row !== e.target) return;
  const panel = row.closest('.left, .right-bottom, .tree.browse, .modal') ?? document.body;
  const rows = [...panel.querySelectorAll<HTMLElement>('.tree-row')].filter(visible);
  const i = rows.indexOf(row);
  const go = (j: number) => {
    const next = rows[Math.max(0, Math.min(rows.length - 1, j))];
    if (!next || next === row) return;
    row.tabIndex = -1; next.tabIndex = 0; next.focus(); next.scrollIntoView({ block: 'nearest' });
  };
  if (e.key === 'ArrowDown') { e.preventDefault(); go(i + 1); }
  else if (e.key === 'ArrowUp') { e.preventDefault(); go(i - 1); }
  else if (e.key === 'Home') { e.preventDefault(); go(0); }
  else if (e.key === 'End') { e.preventDefault(); go(rows.length - 1); }
  else if (e.key === ' ') { e.preventDefault(); row.click(); }
}

export function installModalBehaviour() {
  const seen = new Set<HTMLElement>();
  const sync = () => {
    const now = new Set(document.querySelectorAll<HTMLElement>('.modal-backdrop'));
    for (const b of now) if (!seen.has(b)) { seen.add(b); prepare(b); }
    for (const b of [...seen]) if (!now.has(b)) {
      seen.delete(b);
      const back = returnTo.get(b);
      if (back && back.isConnected && !topBackdrop()) back.focus({ preventScroll: true });
    }
  };
  new MutationObserver(() => { sync(); rovingTrees(); }).observe(document.body, { childList: true, subtree: true });
  rovingTrees();
  window.addEventListener('keydown', treeKeys);

  window.addEventListener('keydown', e => {
    const top = topBackdrop();
    if (!top) return;
    if (e.key === 'Escape' && !e.defaultPrevented) {
      // та же реакция, что на щелчок по подложке
      top.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true }));
      return;
    }
    if (e.key !== 'Tab') return;
    const list = focusables(top);
    if (!list.length) return;
    const first = list[0], last = list[list.length - 1], cur = document.activeElement;
    if (!top.contains(cur)) { e.preventDefault(); first.focus(); }
    else if (e.shiftKey && cur === first) { e.preventDefault(); last.focus(); }
    else if (!e.shiftKey && cur === last) { e.preventDefault(); first.focus(); }
  });
}
