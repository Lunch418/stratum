// Подписка на событие окна, которая всегда вызывает последнюю версию
// обработчика: без неё обработчик клавиш видит состояние на момент
// подписки (например, «копировать» брал бы старые координаты блоков).
import { useEffect, useLayoutEffect, useRef } from 'react';

export function useWindowEvent<K extends keyof WindowEventMap>(type: K, handler: (e: WindowEventMap[K]) => void) {
  const ref = useRef(handler);
  useLayoutEffect(() => { ref.current = handler; });
  useEffect(() => {
    const listener = (e: WindowEventMap[K]) => ref.current(e);
    window.addEventListener(type, listener);
    return () => window.removeEventListener(type, listener);
  }, [type]);
}
