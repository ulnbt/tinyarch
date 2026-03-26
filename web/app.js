// ── CodeMirror 6 from CDN (no bundler required) ──────────────────────────
// All imports from the same codemirror meta-package to avoid duplicate instances.
import { EditorView, basicSetup, EditorState } from 'https://esm.sh/codemirror@6';

// ── TinyArch WASM module ─────────────────────────────────────────────────
import init, { TinyArchWasm } from './pkg/tinyarch_wasm.js';

// ── Constants ─────────────────────────────────────────────────────────────
const RAM_WORDS    = 128 * 1024;          // 128k words of RAM
const MAX_CYCLES   = 50_000_000n;         // cycle budget (BigInt for u64)
const STEPS_PER_FRAME = [100, 1_000, 10_000, 100_000, 1_000_000];

const DEFAULT_PROGRAM = `; TinyArch v2 — Hello World
@term_out -> r1
@hw_len -> r2
hw_dat -> r3
0 -> r4

print_loop:
    if r4 == r2 jmp finish
    r3[r4] -> r5
    r1 + 1 -> r6
    r5 -> r6[r4]
    r4 + 1 -> r4
    jmp print_loop

finish:
    r2 -> @r1
    HALT

term_out: DAT 0x10000000
hw_len:   DAT 13
hw_dat:   STRING "Hello World!"
hw_nl:    DAT 10
`;

// ── State ─────────────────────────────────────────────────────────────────
let machine   = null;   // TinyArchWasm | null
let running   = false;
let assembled = false;  // true if current editor content has been assembled
let rafHandle = null;
let editorView = null;
let prevRegs  = new Uint32Array(16);

// ── Boot ──────────────────────────────────────────────────────────────────
async function boot() {
  await init();
  initEditor();
  bindEvents();
  updateSpeedLabel();
  renderCpuEmpty();
}
boot();

// ── Editor setup ─────────────────────────────────────────────────────────
function initEditor() {
  editorView = new EditorView({
    parent: document.getElementById('editor'),
    state: EditorState.create({
      doc: DEFAULT_PROGRAM,
      extensions: [
        basicSetup,
        EditorView.updateListener.of(update => {
          if (update.docChanged) assembled = false;
        }),
      ],
    }),
  });
}

// ── Event bindings ────────────────────────────────────────────────────────
function bindEvents() {
  document.getElementById('btn-run').addEventListener('click', onRun);
  document.getElementById('btn-step').addEventListener('click', onStep);
  document.getElementById('btn-pause').addEventListener('click', onPause);
  document.getElementById('btn-reset').addEventListener('click', onReset);
  document.getElementById('btn-clear-term').addEventListener('click', () => {
    document.getElementById('terminal').textContent = '';
  });
  document.getElementById('btn-mem-go').addEventListener('click', updateMemoryView);
  document.getElementById('mem-addr').addEventListener('keydown', e => {
    if (e.key === 'Enter') updateMemoryView();
  });
  document.getElementById('speed-slider').addEventListener('input', updateSpeedLabel);
  document.getElementById('example-select').addEventListener('change', onExampleLoad);
}

// ── Run / Step / Pause / Reset ────────────────────────────────────────────
function onRun() {
  if (running) return;
  // Reassemble only if: never assembled yet, or machine halted (restart from top)
  if (!assembled || !machine) {
    if (!tryAssemble()) return;
  } else if (machine.is_halted()) {
    document.getElementById('terminal').textContent = '';
    if (!tryAssemble()) return;
  }
  running = true;
  document.getElementById('btn-pause').disabled = false;
  rafHandle = requestAnimationFrame(tick);
}

function onStep() {
  if (running) return;
  // Assemble on first step only; subsequent presses advance from current state
  if (!assembled || !machine) {
    if (!tryAssemble()) return;
  }
  if (machine.is_halted()) return;
  try {
    machine.step_n(1);
    appendTerminal(machine.drain_output());
    updateCpuState();
    updateMemoryView();
    if (machine.is_halted()) setStatus('Halted after step');
  } catch (e) {
    showError(`Runtime panic: ${e}`);
  }
}

function onPause() {
  if (!running) return;
  running = false;
  if (rafHandle) cancelAnimationFrame(rafHandle);
  document.getElementById('btn-pause').disabled = true;
  setStatus('Paused');
}

function onReset() {
  stopRunning();
  document.getElementById('terminal').textContent = '';
  hideError();
  tryAssemble();
}

// ── Animation frame tick ─────────────────────────────────────────────────
function tick() {
  if (!running || !machine) return;
  const speed = parseInt(document.getElementById('speed-slider').value);
  const n = STEPS_PER_FRAME[speed - 1];
  try {
    const stillRunning = machine.step_n(n);
    const text = machine.drain_output();
    if (text) appendTerminal(text);
    updateCpuState();
    updateMemoryView();
    if (!stillRunning) {
      running = false;
      document.getElementById('btn-pause').disabled = true;
      setStatus(machine.is_halted() ? 'Halted' : 'Cycle limit reached');
      return;
    }
  } catch (e) {
    running = false;
    document.getElementById('btn-pause').disabled = true;
    showError(`Runtime panic: ${e}`);
    return;
  }
  rafHandle = requestAnimationFrame(tick);
}

// ── Assembly ──────────────────────────────────────────────────────────────
function tryAssemble() {
  hideError();
  if (machine) { machine.free(); machine = null; }
  prevRegs = new Uint32Array(16);

  machine = new TinyArchWasm(RAM_WORDS, MAX_CYCLES);
  const source = editorView.state.doc.toString();
  try {
    machine.assemble(source);
  } catch (e) {
    showError(`Assembly error: ${e}`);
    machine.free();
    machine = null;
    assembled = false;
    return false;
  }
  assembled = true;
  updateCpuState();
  updateMemoryView();
  setStatus('Ready');
  return true;
}

// ── Example loader ────────────────────────────────────────────────────────
async function onExampleLoad(e) {
  const url = e.target.value;
  e.target.value = '';
  if (!url) return;
  // Stop any running execution before the fetch so nothing runs while we wait.
  stopRunning();
  try {
    const resp = await fetch(url);
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    const src = await resp.text();
    editorView.dispatch({
      changes: { from: 0, to: editorView.state.doc.length, insert: src },
    });
    assembled = false;
    document.getElementById('terminal').textContent = '';
    hideError();
    renderCpuEmpty();
  } catch (err) {
    showError(`Could not load example: ${err}`);
  }
}

// ── UI helpers ────────────────────────────────────────────────────────────
function stopRunning() {
  if (running) {
    running = false;
    if (rafHandle) cancelAnimationFrame(rafHandle);
  }
  document.getElementById('btn-pause').disabled = true;
}

function appendTerminal(text) {
  if (!text) return;
  const term = document.getElementById('terminal');
  term.textContent += text;
  term.scrollTop = term.scrollHeight;
}

function setStatus(msg) {
  document.getElementById('cpu-status').textContent = msg;
}

function showError(msg) {
  const box = document.getElementById('asm-error');
  box.textContent = msg;
  box.classList.remove('hidden');
}

function hideError() {
  document.getElementById('asm-error').classList.add('hidden');
}

function updateSpeedLabel() {
  const speed = parseInt(document.getElementById('speed-slider').value);
  const n = STEPS_PER_FRAME[speed - 1];
  const label = n >= 1_000_000 ? `${n/1_000_000}M/frame`
              : n >= 1_000     ? `${n/1_000}k/frame`
              :                  `${n}/frame`;
  document.getElementById('speed-label').textContent = label;
}

// ── CPU state renderer ────────────────────────────────────────────────────
function updateCpuState() {
  if (!machine) return;
  const regs   = machine.registers();      // Uint32Array(16)
  const pc     = machine.pc();
  const flags  = machine.flags();
  const stack  = machine.stack_ptr();
  const cycles = machine.cycle_count();    // BigInt

  document.getElementById('cpu-status').innerHTML =
    `PC: <b>0x${pc.toString(16).padStart(4,'0')}</b> &nbsp; ` +
    `Flags: <b>0x${flags.toString(16).padStart(8,'0')}</b> &nbsp; ` +
    `Stack: <b>0x${stack.toString(16).padStart(8,'0')}</b> &nbsp; ` +
    `Cycles: <b>${cycles.toLocaleString()}</b>`;

  const regNames = ['r0','r1','r2','r3','r4','r5','r6','r7',
                    'r8','r9','rA','rB','rC','rD','rE','rF'];
  let html = '<div class="reg-grid">';
  for (let i = 0; i < 16; i++) {
    const changed = regs[i] !== prevRegs[i];
    html += `<span class="reg-cell${changed ? ' changed' : ''}">` +
            `<span class="reg-name">${regNames[i]}</span>` +
            `0x${regs[i].toString(16).padStart(8,'0')}` +
            `</span>`;
  }
  html += '</div>';
  document.getElementById('cpu-state').innerHTML = html;
  prevRegs = regs;
}

function renderCpuEmpty() {
  document.getElementById('cpu-status').textContent = '—';
  const regNames = ['r0','r1','r2','r3','r4','r5','r6','r7',
                    'r8','r9','rA','rB','rC','rD','rE','rF'];
  let html = '<div class="reg-grid">';
  for (const n of regNames) {
    html += `<span class="reg-cell"><span class="reg-name">${n}</span>—</span>`;
  }
  html += '</div>';
  document.getElementById('cpu-state').innerHTML = html;
}

// ── Memory viewer ─────────────────────────────────────────────────────────
function updateMemoryView() {
  if (!machine) return;
  const addrInput = document.getElementById('mem-addr').value.trim();
  const startAddr = parseInt(addrInput, 16) || 0;
  const words = machine.memory_range(startAddr, 64);
  let html = '<table>';
  for (let i = 0; i < words.length; i += 4) {
    const addr = (startAddr + i).toString(16).padStart(4, '0');
    let row = `<tr><th>0x${addr}</th>`;
    for (let j = 0; j < 4 && i + j < words.length; j++) {
      const v = words[i + j];
      const hex = v.toString(16).padStart(8, '0');
      row += `<td class="${v === 0 ? 'zero' : ''}">0x${hex}</td>`;
    }
    row += '</tr>';
    html += row;
  }
  html += '</table>';
  document.getElementById('memory-view').innerHTML = html;
}
