// звук модели: SndPlaySound и MCI приходят в кадре как команды
const playing = new Map<string, HTMLAudioElement>();
export function handleSounds(sounds: { cmd: string; file: string; loop: boolean }[] | undefined) {
  for (const snd of sounds ?? []) {
    const key = snd.file.toLowerCase();
    if (snd.cmd === 'stop') {
      if (!snd.file) { for (const a of playing.values()) a.pause(); playing.clear(); }
      else { playing.get(key)?.pause(); playing.delete(key); }
      continue;
    }
    playing.get(key)?.pause();
    const a = new Audio('/api/file?name=' + encodeURIComponent(snd.file));
    a.loop = snd.loop;
    a.play().catch(() => { /* MIDI и запрет автозапуска - молча */ });
    playing.set(key, a);
  }
}
