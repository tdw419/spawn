// ── Spawn Setup Wizard ────────────────────────────────────
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let currentStep = 1;
let selectedProject = null;
let selectedProjectName = null;
let cloudToken = null;
let manualKey = null;
let hardwareProfile = null;

// ── Navigation ────────────────────────────────────────────────

function goStep(n) {
  document.querySelectorAll('.panel').forEach(p => p.classList.remove('active'));
  document.getElementById('panel-' + n).classList.add('active');

  document.querySelectorAll('.step').forEach(s => {
    const sn = parseInt(s.dataset.step);
    s.classList.remove('active', 'done');
    if (sn < n) s.classList.add('done');
    else if (sn === n) s.classList.add('active');
  });

  currentStep = n;
}

function closeWindow() {
  window.__TAURI__.window.getCurrentWindow().close();
}

// ── Step 1: Hardware detect + Cloud Connect ────────────────────

async function autoConnect() {
  // Detect hardware in parallel with cloud connect
  const [hwResult, cloudResult] = await Promise.allSettled([
    invoke('detect_system'),
    invoke('connect_cloud'),
  ]);

  // Store hardware profile
  if (hwResult.status === 'fulfilled') {
    hardwareProfile = hwResult.value;
    renderHardwareInfo(hardwareProfile);
  }

  // Handle cloud
  if (cloudResult.status === 'fulfilled' && cloudResult.value.connected) {
    cloudToken = cloudResult.value.token;
    showConnectOk(cloudResult.value.daily_limit || 50);
  } else {
    showConnectFailed();
  }
}

function renderHardwareInfo(hw) {
  const box = document.getElementById('hardware-box');
  if (!box) return;

  let icon, label, detail, color;

  if (hw.tier === 'gpu-powerful') {
    icon = '🚀';
    label = 'Powerful GPU detected';
    detail = (hw.gpu_name || 'GPU') + ' -- ' + (hw.vram_mb ? (hw.vram_mb / 1024).toFixed(1) + ' GB VRAM' : '');
    color = 'ok';
  } else if (hw.tier === 'gpu-basic') {
    icon = '⚡';
    label = 'GPU detected';
    detail = (hw.gpu_name || 'GPU') + ' -- ' + (hw.vram_mb ? (hw.vram_mb / 1024).toFixed(1) + ' GB VRAM' : '');
    color = 'ok';
  } else if (hw.tier === 'cpu-only') {
    icon = '💻';
    label = 'CPU-only mode';
    detail = (hw.ram_mb / 1024).toFixed(0) + ' GB RAM -- small model will be used';
    color = 'warn';
  } else {
    icon = '☁️';
    label = 'Cloud-only mode';
    detail = 'Not enough resources for local AI. Cloud access will be used.';
    color = 'warn';
  }

  box.innerHTML =
    '<div class="hw-card ' + color + '">' +
      '<span class="hw-icon">' + icon + '</span>' +
      '<div class="hw-info">' +
        '<strong>' + label + '</strong>' +
        '<span class="hw-detail">' + detail + '</span>' +
        '<span class="hw-model">Will install: ' + hw.model + ' (' + hw.model_size + ')</span>' +
      '</div>' +
    '</div>';
  box.classList.remove('hidden');
}

function showConnectOk(limit) {
  document.getElementById('connect-box').classList.add('hidden');
  document.getElementById('connect-result').classList.remove('hidden');
  document.getElementById('connect-success').classList.remove('hidden');
  document.getElementById('quota-text').textContent = limit + ' free AI messages per day as backup. No credit card.';
  document.getElementById('btn-next-1').disabled = false;
  document.getElementById('btn-next-1').textContent = 'Next';
}

function showConnectFailed() {
  document.getElementById('connect-box').classList.add('hidden');
  document.getElementById('connect-result').classList.remove('hidden');
  document.getElementById('connect-failed').classList.remove('hidden');
  document.getElementById('btn-next-1').disabled = false;
  document.getElementById('btn-next-1').textContent = 'Continue';
}

document.getElementById('manual-key').addEventListener('input', (e) => {
  manualKey = e.target.value.trim();
});

document.getElementById('btn-next-1').addEventListener('click', () => {
  goStep(2);
  loadProjects();
});

// ── Step 2: Projects ──────────────────────────────────────────

async function loadProjects() {
  const projects = await invoke('list_projects');
  const list = document.getElementById('project-list');
  list.innerHTML = '';

  for (const p of projects) {
    const div = document.createElement('div');
    div.className = 'project-card';
    div.dataset.id = p.id;
    div.innerHTML =
      '<div class="name">' + p.name + '</div>' +
      '<div class="desc">' + p.description + '</div>';
    div.addEventListener('click', () => selectProject(p.id, p.name));
    list.appendChild(div);
  }
}

function selectProject(id, name) {
  selectedProject = id;
  selectedProjectName = name;
  document.querySelectorAll('.project-card').forEach(c => {
    c.classList.toggle('selected', c.dataset.id === id);
  });
  document.getElementById('btn-next-2').disabled = false;
}

document.getElementById('btn-next-2').addEventListener('click', () => {
  startInstall();
});

// ── Step 3: Install ───────────────────────────────────────────

async function startInstall() {
  goStep(3);
  const list = document.getElementById('progress-list');

  await listen('setup-step', (event) => {
    const { step, success, message } = event.payload;
    updateProgress(list, step, success, message);
  });

  try {
    const skipOllama = hardwareProfile && hardwareProfile.tier === 'cloud-only';
    const results = await invoke('run_setup', {
      projectId: selectedProject,
      cloudToken: cloudToken,
      manualApiKey: manualKey || null,
      skipOllama: skipOllama,
    });

    list.innerHTML = '';
    for (const r of results) {
      addResult(list, r.step, r.success, r.message);
    }

    // Show done section
    document.getElementById('install-title').textContent = selectedProjectName + ' is ready';
    document.getElementById('project-cd').textContent = 'cd ~/zion/projects/' + selectedProject;
    document.getElementById('done-section').classList.remove('hidden');
  } catch (e) {
    addResult(list, 'Error', false, String(e));
  }
}

function updateProgress(list, step, success, message) {
  let item = list.querySelector('[data-step="' + step + '"]');
  if (!item) {
    item = document.createElement('div');
    item.className = 'check-item running';
    item.dataset.step = step;
    list.appendChild(item);
  }

  if (success && !message) {
    item.className = 'check-item running';
    item.innerHTML =
      '<span class="icon"><span class="spinner"></span></span>' +
      '<span class="label">' + step + '</span>';
  } else {
    item.className = 'check-item ' + (success ? 'ok' : 'fail');
    item.innerHTML =
      '<span class="icon">' + (success ? '\u2713' : '\u2717') + '</span>' +
      '<span class="label">' + step + '</span>' +
      (message ? '<span class="detail">' + message + '</span>' : '');
  }
}

function addResult(list, step, success, message) {
  const div = document.createElement('div');
  div.className = 'check-item ' + (success ? 'ok' : 'fail');
  div.innerHTML =
    '<span class="icon">' + (success ? '\u2713' : '\u2717') + '</span>' +
    '<span class="label">' + step + '</span>' +
    (message ? '<span class="detail">' + message + '</span>' : '');
  list.appendChild(div);
}

// ── Boot ──────────────────────────────────────────────────────
autoConnect();
