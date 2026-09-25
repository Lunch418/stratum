// Гиперпереходы из кадра ядра (закладка «Гипербаза» объектов рисунка).
import type { Frame } from './api';
import { useStore } from './store';

/// Гиперпереходы из кадра: эффект смены страницы, запуск приложения
/// (в браузере недоступен — сообщение), загрузка проекта (после подтверждения).
export function handleHyper(events: Frame['hyper']) {
  for (const h of events ?? []) {
    const s = useStore.getState();
    if (h.mode === 0) {
      const e = h.effect.toLowerCase();
      const fx = /лев|left/.test(e) ? 'fx-left' : /прав|right/.test(e) ? 'fx-right' : /верх|up|top/.test(e) ? 'fx-up' : /низ|down|bottom/.test(e) ? 'fx-down' : 'fx-fade';
      requestAnimationFrame(() => {
        const div = [...document.querySelectorAll<HTMLElement>('.model .win')].find(d => d.dataset.name === h.window);
        if (!div) return;
        div.classList.remove('fx-left', 'fx-right', 'fx-up', 'fx-down', 'fx-fade');
        void div.offsetWidth;
        div.classList.add(fx);
      });
    } else if (h.mode === 1) {
      s.say({ level: 'info', where: 'гипербаза', text: `ссылка запускает приложение «${h.target}» — из среды в браузере приложения не запускаются` });
      s.showToast('Запуск приложений из гиперссылки недоступен');
    } else if (h.mode === 2) {
      const dir = (s.project?.dir ?? '').replace(/[\\/]+$/, '');
      const path = h.target.replace(/^["']|["']$/g, '');
      s.setHyperProject(/^([a-zA-Z]:|[\\/])/.test(path) || !dir ? path : dir + '/' + path);
    }
  }
}

