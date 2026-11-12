import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile, readdir } from 'node:fs/promises';
import vm from 'node:vm';

// Run with: node --experimental-vm-modules --test apps/tauri/tests/frontend.test.mjs
// VM modules let the browser's .js modules remain usable without package.json.
const ui = new URL('../ui/', import.meta.url);
async function modules(globals = {}) {
  const context = vm.createContext({ console, setTimeout, clearTimeout, ...globals });
  const cache = new Map();
  async function load(url) {
    if (cache.has(url.href)) return cache.get(url.href);
    const mod = new vm.SourceTextModule(await readFile(url, 'utf8'), { context, identifier: url.href });
    cache.set(url.href, mod);
    await mod.link((specifier, parent) => load(new URL(specifier, parent.identifier)));
    return mod;
  }
  return async (name) => {
    const mod = await load(new URL(name, ui));
    if (mod.status === 'linked') await mod.evaluate();
    return mod.namespace;
  };
}

function harness(api) {
  const calls = [], timers = [], cleared = [], errors = [], renders = [], states = [];
  let settle;
  const client = api.createIpcClient({
    invoke(command, args) {
      calls.push({ command, args });
      return new Promise((resolve, reject) => { settle = { resolve, reject }; });
    },
    render(status, actionErrors) { renders.push({ status, errors: { ...actionErrors } }); },
    controls(state) { states.push({ ...state }); },
    success() {},
    failure(service, message) { errors.push({ service, message }); },
    setTimer(fn, delay) { timers.push({ fn, delay }); return timers.length; },
    clearTimer(id) { cleared.push(id); },
  });
  return { ...client, calls, timers, cleared, errors, renders, states, get settle() { return settle; } };
}
const snapshot = (volumeBusy = false, bluetoothBusy = false) => ({ volume: { busy: volumeBusy }, bluetooth: { busy: bluetoothBusy } });

test('poll priority: failure 2000, volume 50, Bluetooth 250, idle 750', async () => {
  const api = await (await modules())('ipc.js');
  for (const connected of [false, true]) {
    for (const volumeBusy of [false, true]) {
      for (const bluetoothBusy of [false, true]) {
        assert.equal(api.pollDelay({ connected, volumeBusy, bluetoothBusy }), !connected ? 2000 : volumeBusy ? 50 : bluetoothBusy ? 250 : 750);
      }
    }
  }
});

test('IPC is serialized, drops concurrent actions, and polls only after completion', async () => {
  const h = harness(await (await modules())('ipc.js'));
  const first = h.request('dashboard_status');
  await h.request('volume_action', { action: 'up' });
  assert.equal(h.calls.length, 1);
  assert.equal(h.timers.length, 0);
  assert.equal(h.states.at(-1).inFlight, true);
  h.settle.resolve(snapshot(true, true));
  await first;
  assert.equal(h.timers.at(-1).delay, 50);
  const action = h.request('volume_action', { action: 'up' });
  assert.equal(h.cleared.at(-1), 1);
  assert.deepEqual(h.calls.at(-1), { command: 'volume_action', args: { action: 'up' } });
  h.settle.resolve(snapshot(false, true));
  await action;
  assert.equal(h.timers.at(-1).delay, 250);
  h.timers.at(-1).fn();
  assert.equal(h.calls.at(-1).command, 'dashboard_status');
  h.settle.resolve(snapshot());
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(h.timers.at(-1).delay, 750);
  assert.equal(h.states.at(-1).inFlight, false);
});

test('action errors survive polls, clear on service success; IPC failures retry and recover', async () => {
  const h = harness(await (await modules())('ipc.js'));
  let pending = h.request('dashboard_status');
  h.settle.reject('offline');
  await pending;
  assert.equal(h.timers.at(-1).delay, 2000);
  assert.deepEqual(h.errors.at(-1), { service: null, message: 'offline' });
  pending = h.request('dashboard_status');
  h.settle.resolve(snapshot());
  await pending;
  pending = h.request('bluetooth_action');
  h.settle.reject('denied');
  await pending;
  assert.deepEqual(h.errors.at(-1), { service: 'bluetooth', message: 'Action failed: denied' });
  assert.equal(h.timers.at(-1).delay, 750);
  pending = h.request('dashboard_status');
  h.settle.resolve(snapshot());
  await pending;
  assert.equal(h.renders.at(-1).errors.bluetooth, 'Action failed: denied');
  pending = h.request('bluetooth_action');
  h.settle.resolve(snapshot());
  await pending;
  assert.equal(h.renders.at(-1).errors.bluetooth, '');
});

class Element {
  constructor() {
    this.children = []; this.dataset = {}; this.attributes = {}; this.textContent = '';
    this.classList = { toggle() {}, add() {} };
  }
  append(...nodes) { for (const node of nodes) { node.parent = this; this.children.push(node); } }
  replaceChildren(...nodes) { this.children = []; this.append(...nodes); }
  remove() { this.parent.children = this.parent.children.filter(node => node !== this); }
  setAttribute(key, value) { this.attributes[key] = value; }
  removeAttribute(key) { delete this.attributes[key]; }
  addEventListener(event, callback) { this[event] = callback; }
  querySelectorAll() { return this.children.flatMap(node => node.dataset.bluetooth ? [node] : node.querySelectorAll()); }
}
async function fakeDocument() {
  const html = await readFile(new URL('index.html', ui), 'utf8');
  const elements = new Map([...html.matchAll(/\bid="([^"]+)"/g)].map(match => [match[1], new Element()]));
  return { html, elements, getElementById: id => elements.get(id), querySelectorAll: () => [], createElement: () => new Element() };
}

test('safe telemetry and Bluetooth rendering keeps unchanged nodes and current device confirmation', async () => {
  const document = await fakeDocument();
  const confirmations = [], calls = [];
  const load = await modules({ document, window: { confirm(message) { confirmations.push(message); return true; } } });
  const { renderComputer } = await load('screens/computer.js');
  const { createBluetoothScreen } = await load('screens/bluetooth.js');
  const renderBluetooth = createBluetoothScreen((...args) => calls.push(args));
  const computer = { connected: true, url: '<img>', fields: { '<script>': '<img onerror=bad>' } };
  renderComputer({ computer });
  const fields = document.elements.get('computer-fields');
  const term = fields.children[0];
  renderComputer({ computer });
  assert.equal(fields.children[0], term);
  assert.equal(term.textContent, '<script>');
  assert.equal(fields.children[1].textContent, '<img onerror=bad>');
  const device = { address: 'AA:BB', name: '<img>', paired: true, trusted: false, connected: false };
  const bluetooth = { devices: [device], selected_address: 'aa:bb', busy: false };
  renderBluetooth({ bluetooth }, {});
  const list = document.elements.get('bluetooth-devices');
  const row = list.children[0];
  assert.equal(row.children[0].textContent, '<img>');
  device.name = 'New name';
  renderBluetooth({ bluetooth }, {});
  assert.equal(list.children[0], row);
  const buttons = row.querySelectorAll();
  assert.equal(buttons[0].attributes['aria-pressed'], 'true');
  assert.equal(buttons[1].dataset.unavailable, 'true');
  assert.equal(buttons[4].dataset.unavailable, 'true');
  buttons[5].click();
  assert.match(confirmations[0], /New name/);
  assert.equal(calls[0][0], 'bluetooth_action');
  renderBluetooth({ bluetooth: { ...bluetooth, devices: [] } }, {});
  assert.equal(list.children.length, 0);
});

test('bootstrap module graph links, renders a full snapshot, and HTML supplies literal DOM IDs', async () => {
  const document = await fakeDocument();
  const timers = [], calls = [];
  const status = {
    ...snapshot(), system: {}, computer: { connected: false, fields: {}, url: 'ws://local' },
    spotify: { running: false }, bluetooth: { busy: false, devices: [] },
  };
  const load = await modules({ document, window: { __TAURI__: { core: { invoke(command) { calls.push(command); return Promise.resolve(status); } } } }, setTimeout(fn, delay) { timers.push(delay); return 1; }, clearTimeout() {} });
  await load('app.js');
  await new Promise(resolve => setImmediate(resolve));
  assert.deepEqual(calls, ['dashboard_status']);
  assert.deepEqual(timers, [750]);
  assert.equal(document.elements.get('connection').textContent, '● CONNECTED');
  assert.match(document.html, /<script type="module" src="app.js"><\/script>/);
  const files = [...(await readdir(ui)).filter(name => name.endsWith('.js')), ...(await readdir(new URL('screens/', ui))).map(name => `screens/${name}`)];
  for (const file of files) {
    const source = await readFile(new URL(file, ui), 'utf8');
    for (const match of source.matchAll(/(?:element|text|meter|showError)\("([^"]+)"/g)) {
      assert.ok(document.elements.has(match[1]), `${file}: missing #${match[1]}`);
    }
    assert.doesNotMatch(source, /innerHTML/);
  }
});
