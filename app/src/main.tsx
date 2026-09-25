import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import './index.css';
import App from './App';
import { installModalBehaviour } from './a11y';

// токен сессии уже лёг в cookie — убираем его из адресной строки и истории
const url = new URL(location.href);
if (url.searchParams.has('token')) {
  url.searchParams.delete('token');
  history.replaceState(null, '', url.pathname + url.search + url.hash);
}

installModalBehaviour();
createRoot(document.getElementById('root')!).render(<StrictMode><App /></StrictMode>);
