const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let cfg = { autostart: true, devices: [] };
let states = [];
let models = [];
let sel = 0;
let interacting = false; // suppress live re-render while the user edits

const $ = (id) => document.getElementById(id);
const el = (tag, cls, txt) => {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (txt != null) e.textContent = txt;
  return e;
};
const pretty = (id) =>
  id.replace(/([A-Z])/g, " $1").replace(/^./, (c) => c.toUpperCase()).trim();

function toast(msg, isErr = false) {
  const t = $("toast");
  t.textContent = msg;
  t.className = "toast" + (isErr ? " err" : "");
  setTimeout(() => t.classList.add("hidden"), 2200);
}

// ---- load + poll ----
async function init() {
  models = await invoke("get_models").catch(() => []);
  await loadConfig();
  document.addEventListener("pointerdown", () => (interacting = true));
  document.addEventListener("pointerup", () => setTimeout(() => (interacting = false), 200));
  $("autostart").addEventListener("change", onAutostart);
  $("close-btn").addEventListener("click", () => invoke("hide_window"));
  $("save-btn").addEventListener("click", onSave);
  $("apply-btn").addEventListener("click", onApply);
  $("add-first").addEventListener("click", addDevice);
  await listen("tray-apply", onApply);
  poll();
  setInterval(poll, 800);
}

async function loadConfig() {
  cfg = await invoke("get_config");
  if (!cfg.devices) cfg.devices = [];
  $("autostart").checked = !!cfg.autostart;
  if (sel >= cfg.devices.length) sel = Math.max(0, cfg.devices.length - 1);
}

async function poll() {
  try { states = await invoke("get_states"); } catch { states = []; }
  renderStatus();
  renderTabs();
  const focused = ["INPUT", "SELECT"].includes(document.activeElement?.tagName);
  if (!interacting && !focused) renderPanel();
}

const curState = () => states[sel] || null;
const curDevice = () => cfg.devices[sel] || null;

function renderStatus() {
  const st = curState();
  const pill = $("status-pill"), txt = $("status-text");
  if (st && st.connected) {
    pill.classList.add("on");
    txt.textContent = st.message || "connected";
  } else {
    pill.classList.remove("on");
    txt.textContent = st ? (st.message || "waiting") : "no device";
  }
}

function renderTabs() {
  const tabs = $("device-tabs");
  tabs.innerHTML = "";
  cfg.devices.forEach((d, i) => {
    const t = el("button", "tab" + (i === sel ? " active" : ""), d.name || d.mac_address || "device");
    t.title = d.name || d.mac_address;
    t.onclick = () => { sel = i; renderPanel(); renderTabs(); renderStatus(); };
    tabs.appendChild(t);
  });
  const add = el("button", "tab-add", "+");
  add.title = "Add device";
  add.onclick = addDevice;
  tabs.appendChild(add);
}

function renderPanel() {
  const panel = $("device-panel");
  const empty = $("empty");
  if (cfg.devices.length === 0) {
    panel.classList.add("hidden");
    empty.classList.remove("hidden");
    return;
  }
  empty.classList.add("hidden");
  panel.classList.remove("hidden");
  panel.innerHTML = "";
  const d = curDevice();
  const st = curState();

  panel.appendChild(deviceCard(d));
  if (st && st.connected && st.categories.length) {
    st.categories.forEach((c) => panel.appendChild(categoryCard(d, st, c)));
  } else {
    panel.appendChild(profileFallbackCard(d));
  }
}

// ---- device fields card ----
function deviceCard(d) {
  const card = el("div", "card");
  card.appendChild(el("h2", null, "Device"));

  card.appendChild(textField("Name", d.name, (v) => (d.name = v)));
  card.appendChild(textField("MAC", d.mac_address, (v) => (d.mac_address = v)));

  // model select
  const f = el("div", "field");
  f.appendChild(el("label", null, "Model"));
  const select = el("select");
  models.forEach((m) => {
    const o = el("option", null, m);
    o.value = m;
    if (m === d.model) o.selected = true;
    select.appendChild(o);
  });
  select.onchange = () => (d.model = select.value);
  f.appendChild(select);
  card.appendChild(f);

  card.appendChild(numField("Poll (s)", d.poll_seconds, 1, 120, (v) => (d.poll_seconds = v)));
  card.appendChild(numField("Delay (s)", d.apply_delay_seconds, 0, 30, (v) => (d.apply_delay_seconds = v)));

  const actions = el("div", "row-actions");
  const scanBtn = el("button", "btn small", "Scan");
  scanBtn.onclick = () => doScan(d, card);
  const remBtn = el("button", "btn small danger", "Remove");
  remBtn.onclick = () => removeDevice();
  actions.appendChild(scanBtn);
  actions.appendChild(remBtn);
  card.appendChild(actions);
  return card;
}

async function doScan(d, card) {
  card.querySelector(".scan-list")?.remove();
  const list = el("div", "scan-list");
  list.appendChild(el("div", "muted", "scanning…"));
  card.appendChild(list);
  try { await invoke("scan", { model: d.model }); } catch (e) { toast(String(e), true); }
  // poll scan results a few times
  for (let i = 0; i < 12; i++) {
    await new Promise((r) => setTimeout(r, 500));
    const s = await invoke("get_scan").catch(() => null);
    if (!s) continue;
    if (!s.scanning) {
      list.innerHTML = "";
      if (s.results.length === 0) { list.appendChild(el("div", "muted", "no connected devices found")); break; }
      s.results.forEach((r) => {
        const b = el("button", "scan-item");
        b.innerHTML = `${r.name} <small>${r.mac_address}</small>`;
        b.onclick = () => { d.mac_address = r.mac_address; renderPanel(); };
        list.appendChild(b);
      });
      break;
    }
  }
}

// ---- live settings ----
function categoryCard(d, st, c) {
  const card = el("div", "card");
  card.appendChild(el("h2", null, pretty(c.id)));
  c.settings.forEach((s) => card.appendChild(settingRow(d, st, s)));
  return card;
}

function settingRow(d, st, s) {
  const row = el("div", "setting");

  const starred = d.profile.some((e) => e.id === s.id);
  const star = el("button", "star" + (starred ? " on" : ""), "★");
  star.title = "Apply on connect";
  star.onclick = () => {
    const idx = d.profile.findIndex((e) => e.id === s.id);
    if (idx >= 0) d.profile.splice(idx, 1);
    else d.profile.push({ id: s.id, value: rawOf(s) });
    star.classList.toggle("on");
  };
  row.appendChild(star);

  const label = el("div", "label");
  label.appendChild(document.createTextNode(pretty(s.id)));
  row.appendChild(label);

  const ctrl = el("div", "ctrl");
  buildControl(ctrl, d, s);
  row.appendChild(ctrl);
  return row;
}

function rawOf(s) {
  switch (s.type) {
    case "toggle": return String(s.value);
    case "i32Range": return String(s.value);
    case "select": return s.value ?? "";
    case "optionalSelect":
    case "modifiableSelect": return s.value ?? "";
    case "multiSelect": return (s.values || []).join(",");
    case "equalizer": return (s.value || []).join(",");
    case "information": return s.value ?? "";
    default: return "";
  }
}

function send(d, id, raw) {
  invoke("set_setting", { mac: d.mac_address, id, raw }).catch((e) => toast(String(e), true));
  const entry = d.profile.find((e) => e.id === id);
  if (entry) entry.value = raw; // keep starred value fresh
}

function buildControl(ctrl, d, s) {
  switch (s.type) {
    case "toggle": {
      const lab = el("label", "switch");
      const inp = el("input"); inp.type = "checkbox"; inp.checked = !!s.value;
      const track = el("span", "track");
      inp.onchange = () => send(d, s.id, String(inp.checked));
      lab.appendChild(inp); lab.appendChild(track); ctrl.appendChild(lab);
      break;
    }
    case "i32Range": {
      const r = s.setting.range;
      const range = el("input"); range.type = "range";
      range.min = r.start; range.max = r.end; range.step = s.setting.step || 1; range.value = s.value;
      const out = el("output", null, String(s.value));
      range.oninput = () => (out.textContent = range.value);
      range.onchange = () => send(d, s.id, range.value);
      ctrl.appendChild(range); ctrl.appendChild(out);
      break;
    }
    case "select":
    case "optionalSelect":
    case "modifiableSelect": {
      const select = el("select");
      if (s.type !== "select") {
        const o = el("option", null, "(none)"); o.value = ""; if (s.value == null) o.selected = true;
        select.appendChild(o);
      }
      const opts = s.setting.options, locs = s.setting.localizedOptions || opts;
      opts.forEach((opt, i) => {
        const o = el("option", null, locs[i] || opt); o.value = opt;
        if (opt === s.value) o.selected = true;
        select.appendChild(o);
      });
      select.onchange = () => send(d, s.id, select.value);
      ctrl.appendChild(select);
      break;
    }
    case "multiSelect": {
      const wrap = el("div", "chips");
      const sel = new Set(s.values || []);
      const opts = s.setting.options, locs = s.setting.localizedOptions || opts;
      opts.forEach((opt, i) => {
        const chip = el("button", "chip" + (sel.has(opt) ? " on" : ""), locs[i] || opt);
        chip.onclick = () => {
          if (sel.has(opt)) sel.delete(opt); else sel.add(opt);
          chip.classList.toggle("on");
          send(d, s.id, [...sel].join(","));
        };
        wrap.appendChild(chip);
      });
      ctrl.appendChild(wrap);
      break;
    }
    case "equalizer": {
      const fd = s.setting.fractionDigits || 0;
      const div = el("div", "eq");
      const values = (s.value || []).slice();
      s.setting.bandHz.forEach((hz, i) => {
        const band = el("div", "band");
        band.appendChild(el("span", null, hz >= 1000 ? (hz / 1000) + "k" : hz + "Hz"));
        const range = el("input"); range.type = "range";
        range.min = s.setting.min; range.max = s.setting.max; range.step = 1; range.value = values[i] ?? 0;
        const out = el("output", null, (range.value / Math.pow(10, fd)).toFixed(fd));
        range.oninput = () => (out.textContent = (range.value / Math.pow(10, fd)).toFixed(fd));
        range.onchange = () => { values[i] = parseInt(range.value, 10); send(d, s.id, values.join(",")); };
        band.appendChild(range); band.appendChild(out);
        div.appendChild(band);
      });
      ctrl.appendChild(div);
      ctrl.style.flex = "1";
      break;
    }
    case "information": {
      ctrl.appendChild(el("span", "muted", s.translatedValue || s.value || "—"));
      break;
    }
    default:
      ctrl.appendChild(el("span", "muted", "—"));
  }
}

// fallback when offline: editable id/value rows
function profileFallbackCard(d) {
  const card = el("div", "card");
  card.appendChild(el("h2", null, "Profile (applied on connect)"));
  card.appendChild(el("div", "muted", "Connect the device to edit settings with live controls."));
  d.profile.forEach((entry, i) => {
    const f = el("div", "field");
    const idIn = el("input"); idIn.type = "text"; idIn.value = entry.id; idIn.placeholder = "settingId";
    idIn.oninput = () => (entry.id = idIn.value);
    const valIn = el("input"); valIn.type = "text"; valIn.value = entry.value; valIn.placeholder = "value";
    valIn.oninput = () => (entry.value = valIn.value);
    const rm = el("button", "icon-btn", "✕");
    rm.onclick = () => { d.profile.splice(i, 1); renderPanel(); };
    f.appendChild(idIn); f.appendChild(valIn); f.appendChild(rm);
    card.appendChild(f);
  });
  const add = el("button", "btn small", "+ Add setting");
  add.onclick = () => { d.profile.push({ id: "", value: "" }); renderPanel(); };
  card.appendChild(add);
  return card;
}

// ---- helpers ----
function textField(label, value, onset) {
  const f = el("div", "field");
  f.appendChild(el("label", null, label));
  const inp = el("input"); inp.type = "text"; inp.value = value || "";
  inp.oninput = () => onset(inp.value);
  f.appendChild(inp);
  return f;
}
function numField(label, value, min, max, onset) {
  const f = el("div", "field");
  f.appendChild(el("label", null, label));
  const inp = el("input"); inp.type = "number"; inp.min = min; inp.max = max; inp.value = value;
  inp.oninput = () => onset(parseInt(inp.value || min, 10));
  f.appendChild(inp);
  return f;
}

// ---- actions ----
function addDevice() {
  cfg.devices.push({
    name: "New device", mac_address: "", model: models[0] || "SoundcoreA3959",
    poll_seconds: 5, apply_delay_seconds: 2, profile: [],
  });
  sel = cfg.devices.length - 1;
  renderTabs(); renderPanel();
}
function removeDevice() {
  if (cfg.devices.length === 0) return;
  cfg.devices.splice(sel, 1);
  if (sel >= cfg.devices.length) sel = Math.max(0, cfg.devices.length - 1);
  onSave();
  renderTabs(); renderPanel();
}
async function onSave() {
  // Refresh starred values from live snapshots so the saved profile reflects current settings.
  cfg.devices.forEach((d, i) => {
    const st = states[i];
    if (!st) return;
    const live = {};
    st.categories.forEach((c) => c.settings.forEach((s) => (live[s.id] = rawOf(s))));
    d.profile.forEach((e) => { if (live[e.id] != null) e.value = live[e.id]; });
  });
  cfg.autostart = $("autostart").checked;
  try { await invoke("save_config", { newConfig: cfg }); toast("Saved"); }
  catch (e) { toast(String(e), true); }
}
async function onApply() {
  const d = curDevice();
  if (d && d.mac_address) invoke("apply_now", { mac: d.mac_address }).catch((e) => toast(String(e), true));
}
async function onAutostart() {
  cfg.autostart = $("autostart").checked;
  onSave();
}

init();
