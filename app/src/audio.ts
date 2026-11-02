// Звук модели (Audio*): воспроизведение через Web Audio API. Состояние
// дескрипторов ведёт ядро, сюда приходят снимки команд. Браузер запрещает
// звук без жеста пользователя, поэтому контекст создаётся и возобновляется
// по первому щелчку на кнопке запуска (unlockAudio).
export interface AudioCmd { cmd: 'open' | 'play' | 'stop' | 'update' | 'seek'; id: number; file: string; volume: number; balance: number; tone: number; repeat: boolean; pos: number }

interface Voice {
  buffer?: AudioBuffer;
  loading?: Promise<void>;
  source?: AudioBufferSourceNode;
  gain?: GainNode;
  pan?: StereoPannerNode;
  wantPlay: boolean;
  last: AudioCmd;
}

let ctx: AudioContext | null = null;
const voices = new Map<number, Voice>();
const cache = new Map<string, Promise<AudioBuffer | undefined>>();

export function unlockAudio() {
  try {
    ctx ??= new AudioContext();
    if (ctx.state === 'suspended') void ctx.resume();
    // звуки, запрошенные до жеста, начинаются сейчас
    for (const v of voices.values()) {
      v.loading ??= load(v.last.file).then(b => { v.buffer = b; if (v.wantPlay && b) start(v); });
      if (v.buffer && v.wantPlay && !v.source) start(v);
    }
  } catch { /* нет Web Audio */ }
}

function load(file: string): Promise<AudioBuffer | undefined> {
  let p = cache.get(file);
  if (!p) {
    p = (async () => {
      try {
        const r = await fetch('/api/file?name=' + encodeURIComponent(file));
        if (!r.ok || !ctx) return undefined;
        return await ctx.decodeAudioData(await r.arrayBuffer());
      } catch { return undefined; /* формат не поддержан WebView */ }
    })();
    cache.set(file, p);
  }
  return p;
}

function stopSource(v: Voice) {
  if (v.source) { v.source.onended = null; try { v.source.stop(); } catch { /* уже остановлен */ } v.source.disconnect(); v.source = undefined; }
}

function start(v: Voice) {
  if (!ctx || !v.buffer) return;
  stopSource(v);
  const c = v.last;
  const src = ctx.createBufferSource();
  src.buffer = v.buffer;
  src.loop = c.repeat;
  src.playbackRate.value = c.tone;
  v.gain ??= ctx.createGain();
  v.pan ??= ctx.createStereoPanner();
  src.connect(v.gain).connect(v.pan).connect(ctx.destination);
  const off = Math.min(Math.max(0, c.pos), Math.max(0, v.buffer.duration - 0.001));
  src.start(0, off);
  src.onended = () => { if (v.source === src) v.source = undefined; };
  v.source = src;
}

function applyParams(v: Voice) {
  const c = v.last;
  if (v.gain) v.gain.gain.value = c.volume;
  if (v.pan) v.pan.pan.value = c.balance;
  if (v.source) { v.source.playbackRate.value = c.tone; v.source.loop = c.repeat; }
}

export function handleAudio(cmds: AudioCmd[] | undefined) {
  for (const c of cmds ?? []) {
    let v = voices.get(c.id);
    if (!v || c.cmd === 'open') {
      v = { wantPlay: false, last: c };
      voices.set(c.id, v);
    }
    v.last = c;
    const voice = v;
    if (!ctx) { if (c.cmd === 'play') voice.wantPlay = true; continue; }
    voice.loading ??= load(c.file).then(b => { voice.buffer = b; if (voice.wantPlay && b) start(voice); });
    if (c.cmd === 'play') {
      voice.wantPlay = true;
      if (voice.buffer) start(voice);
    } else if (c.cmd === 'stop') {
      voice.wantPlay = false;
      stopSource(voice);
    } else if (c.cmd === 'seek' && voice.source) {
      start(voice);
    }
    applyParams(voice);
  }
}

/// Остановка всех голосов (остановка модели).
export function stopAllAudio() {
  for (const v of voices.values()) { v.wantPlay = false; stopSource(v); }
  voices.clear();
}
