// Редактор текста имиджа: Monaco с грамматикой языка Stratum, подсказками
// по 960 функциям и константам, ошибками разбора от ядра.
import Editor, { loader, type Monaco, type OnMount } from '@monaco-editor/react';
import * as monacoLib from 'monaco-editor';
import type { editor as MonacoEditor, Position } from 'monaco-editor';
import EditorWorker from 'monaco-editor/editor/editor.worker.js?worker';

// Monaco из сборки, а не с CDN: IDE должна работать без сети
self.MonacoEnvironment = { getWorker: () => new EditorWorker() };
loader.config({ monaco: monacoLib });
import { useEffect, useRef, useState } from 'react';
import { api } from '../api';
import { classByName, useStore } from '../store';
import langData from '../lang-data.json';

const LANG = 'stratum';
let registered = false;

function registerLanguage(monaco: Monaco) {
  if (registered) return;
  registered = true;
  monaco.languages.register({ id: LANG });
  const functions = langData.functions.map(f => f.name);
  const constants = langData.constants.map(c => c.name);
  monaco.languages.setMonarchTokensProvider(LANG, {
    ignoreCase: true,
    keywords: ['if', 'else', 'endif', 'while', 'endwhile', 'do', 'until', 'switch', 'case', 'default', 'endswitch', 'break', 'function', 'return', 'local', 'parameter', 'nosave'],
    types: ['FLOAT', 'STRING', 'HANDLE', 'COLORREF', 'INTEGER', 'WORD', 'BYTE', 'POINTER'],
    functions,
    constants,
    tokenizer: {
      root: [
        [/\/\/.*$/, 'comment'],
        [/\/\*/, 'comment', '@comment'],
        [/"([^"]|"")*"/, 'string'],
        [/'[^']*'/, 'string'],
        [/#\d+/, 'number.handle'],
        [/\d+(\.\d+)?([eE][+-]?\d+)?/, 'number'],
        [/~/, 'operator.tilde'],
        [/[A-Za-zА-Яа-яЁё_][\wА-Яа-яЁё]*/, {
          cases: { '@keywords': 'keyword', '@types': 'type', '@functions': 'support.function', '@constants': 'constant', '@default': 'identifier' },
        }],
        [/::=|:=|==|!=|<=|>=|&&|\|\||<<|>>|[-+*/%^=<>&|!()?,]/, 'operator'],
      ],
      comment: [[/\*\//, 'comment', '@pop'], [/./, 'comment']],
    },
  });
  monaco.languages.setLanguageConfiguration(LANG, {
    comments: { lineComment: '//', blockComment: ['/*', '*/'] },
    brackets: [['(', ')']],
    autoClosingPairs: [{ open: '(', close: ')' }, { open: '"', close: '"' }],
    indentationRules: {
      increaseIndentPattern: /^\s*(if|while|do|switch|case|default|else)\b/i,
      decreaseIndentPattern: /^\s*(endif|endwhile|until|endswitch|else|case|default)\b/i,
    },
  });
  const items = [
    ...langData.functions.map(f => ({ label: f.name, kind: 1, detail: f.sig, documentation: f.desc, insertText: f.name + '(' })),
    ...langData.constants.map(c => ({ label: c.name, kind: 14, detail: String(c.value), insertText: c.name })),
  ];
  monaco.languages.registerCompletionItemProvider(LANG, {
    provideCompletionItems: (model: MonacoEditor.ITextModel, position: Position) => {
      const word = model.getWordUntilPosition(position);
      const range = { startLineNumber: position.lineNumber, endLineNumber: position.lineNumber, startColumn: word.startColumn, endColumn: word.endColumn };
      // переменные имиджа из текста
      const vars = new Set<string>();
      for (const m of model.getValue().matchAll(/\b(FLOAT|STRING|HANDLE|COLORREF)\s+(?:local\s+|parameter\s+|nosave\s+)*([\wА-Яа-яЁё, ]+)/gi)) {
        for (const v of m[2].split(',')) vars.add(v.trim());
      }
      return {
        suggestions: [
          ...items.map(i => ({ ...i, range, kind: i.kind as never })),
          ...[...vars].filter(Boolean).map(v => ({ label: v, kind: 4 as never, insertText: v, range })),
        ],
      };
    },
  });
  monaco.languages.registerHoverProvider(LANG, {
    provideHover: (model: MonacoEditor.ITextModel, position: Position) => {
      const w = model.getWordAtPosition(position);
      if (!w) return null;
      const f = langData.functions.find(f => f.name.toLowerCase() === w.word.toLowerCase());
      if (f) return { contents: [{ value: '```\n' + f.sig + '\n```' }, { value: f.desc }] };
      const c = langData.constants.find(c => c.name.toLowerCase() === w.word.toLowerCase());
      if (c) return { contents: [{ value: `**${c.name}** = ${c.value}` }] };
      return null;
    },
  });
  monaco.editor.defineTheme('stratum-light', {
    base: 'vs', inherit: true, colors: { 'editor.background': '#ffffff' },
    rules: [
      { token: 'keyword', foreground: '2f6fdb', fontStyle: 'bold' }, { token: 'type', foreground: '8a63d2' },
      { token: 'support.function', foreground: '1f7a8c' }, { token: 'constant', foreground: 'e07b39' },
      { token: 'operator.tilde', foreground: 'c93b3b', fontStyle: 'bold' }, { token: 'number.handle', foreground: '1f9e9e' },
      { token: 'comment', foreground: '6b7280', fontStyle: 'italic' }, { token: 'string', foreground: '1f9d55' },
    ],
  });
  monaco.editor.defineTheme('stratum-dark', {
    base: 'vs-dark', inherit: true, colors: { 'editor.background': '#1b1e26' },
    rules: [
      { token: 'keyword', foreground: '6fa0ff', fontStyle: 'bold' }, { token: 'type', foreground: 'b39ddb' },
      { token: 'support.function', foreground: '7fd1dc' }, { token: 'constant', foreground: 'f0a066' },
      { token: 'operator.tilde', foreground: 'ff6b6b', fontStyle: 'bold' }, { token: 'number.handle', foreground: '5fd0d0' },
      { token: 'comment', foreground: '8b92a3', fontStyle: 'italic' }, { token: 'string', foreground: '3dcb7a' },
    ],
  });
}

export function CodeEditor() {
  const project = useStore(s => s.project);
  const selected = useStore(s => s.selectedClass);
  const theme = useStore(s => s.theme);
  const updateClass = useStore(s => s.updateClass);
  const say = useStore(s => s.say);
  const showToast = useStore(s => s.showToast);
  const klass = classByName(project, selected);
  const [text, setText] = useState('');
  const [dirty, setDirty] = useState(false);
  const editorRef = useRef<Parameters<OnMount>[0] | null>(null);
  const monacoRef = useRef<Monaco | null>(null);
  const halt = useStore(s => s.frame?.halt);
  const gotoLine = useStore(s => s.gotoLine);
  const setGotoLine = useStore(s => s.setGotoLine);
  useEffect(() => {
    const editor = editorRef.current;
    if (!editor || !gotoLine || !klass || gotoLine.class.toLowerCase() !== klass.name.toLowerCase()) return;
    editor.revealLineInCenter(gotoLine.line);
    editor.setPosition({ lineNumber: gotoLine.line, column: 1 });
    editor.focus();
    setGotoLine(null);
  }, [gotoLine, klass?.name, text]);
  const decorations = useRef<string[]>([]);

  // подсветка строки ошибки времени выполнения в тексте этого имиджа
  useEffect(() => {
    const editor = editorRef.current, monaco = monacoRef.current;
    if (!editor || !monaco) return;
    const mine = halt && halt.line > 0 && klass && halt.class.toLowerCase() === klass.name.toLowerCase();
    decorations.current = editor.deltaDecorations(decorations.current, mine ? [{
      range: new monaco.Range(halt.line, 1, halt.line, 1),
      options: { isWholeLine: true, className: halt.kind === 'error' ? 'line-error' : 'line-halt', glyphMarginClassName: 'glyph-halt' },
    }] : []);
    if (mine) editor.revealLineInCenter(halt.line);
  }, [halt?.message, halt?.line, klass?.name]);

  useEffect(() => { setText(klass?.text ?? ''); setDirty(false); }, [klass?.name]);

  async function save() {
    if (!klass) return;
    const r = await api.setText(klass.name, text);
    updateClass({ ...klass, text });
    useStore.getState().markUnsaved();
    setDirty(false);
    const monaco = monacoRef.current, editor = editorRef.current;
    if (monaco && editor) {
      const model = editor.getModel();
      if (model) monaco.editor.setModelMarkers(model, 'stratum', r.ok || !r.error ? [] : [{
        severity: 8, message: r.error.message, startLineNumber: r.error.line, startColumn: r.error.column, endLineNumber: r.error.line, endColumn: r.error.column + 1,
      }]);
    }
    if (r.ok) { showToast(r.live ? 'Применено на ходу' : 'Текст принят'); say({ level: 'info', where: klass.name, text: r.live ? 'текст применён в работающей модели' : 'текст принят' }); }
    else if (r.error) say({ level: 'error', where: `${klass.name}, строка ${r.error.line}`, text: r.error.message });
  }

  const onMount: OnMount = (editor, monaco) => {
    editorRef.current = editor; monacoRef.current = monaco;
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => void save());
    // F1 — справка по слову под курсором
    editor.addCommand(monaco.KeyCode.F1, () => {
      const pos = editor.getPosition();
      const word = pos && editor.getModel()?.getWordAtPosition(pos)?.word;
      if (word) useStore.getState().setHelpTopic(word);
    });
  };

  if (!klass) return <div className="muted" style={{ padding: 16 }}>Выберите имидж в иерархии.</div>;
  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div className="panel-title">
        <span>{klass.name}{klass.library ? ' · библиотека' : ''}{dirty ? ' · изменён' : ''}</span>
        <span className="spacer" />
        <button className="small" onClick={save} disabled={!dirty} title="Ctrl+S">Сохранить</button>
      </div>
      <div style={{ flex: 1, minHeight: 0 }}>
        <Editor
          language={LANG}
          theme={theme === 'dark' ? 'stratum-dark' : 'stratum-light'}
          value={text}
          beforeMount={registerLanguage}
          onMount={onMount}
          onChange={v => { setText(v ?? ''); setDirty(true); }}
          options={{ fontFamily: 'JetBrains Mono, ui-monospace, monospace', fontSize: 13, minimap: { enabled: false }, tabSize: 2, scrollBeyondLastLine: false, wordBasedSuggestions: 'off', readOnly: klass.library }}
        />
      </div>
    </div>
  );
}
