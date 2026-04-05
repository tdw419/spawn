use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tauri::{Emitter, Window};

// ── Config ─────────────────────────────────────────────────────────────

const CLOUD_API_BASE: &str = "https://spawn-cloud.tdw419.workers.dev";
const CLOUD_API_FALLBACK: &str = "http://localhost:8787";
const HERMES_INSTALL_URL: &str = "https://raw.githubusercontent.com/NousResearch/hermes-agent/main/scripts/install.sh";

// ── Types ──────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone)]
struct Project {
    id: String,
    name: String,
    description: String,
    company_id: String,
    plugins: Vec<String>,
    repo_url: String,
}

#[derive(Serialize, Clone)]
struct StepResult {
    step: String,
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct PrereqCheck {
    name: String,
    installed: bool,
    version: Option<String>,
    required: String,
    install_cmd: Option<String>,
}

#[derive(Serialize)]
struct CloudStatus {
    connected: bool,
    token: Option<String>,
    daily_limit: Option<u32>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct RegisterResponse {
    token: String,
    proxy_url: String,
    daily_limit: u32,
}

#[derive(Serialize, Clone)]
struct HardwareProfile {
    tier: String,           // "gpu-powerful", "gpu-basic", "cpu-only", "cloud-only"
    backend: String,        // "ollama" or "cloud"
    model: String,          // model name to pull
    model_size: String,     // human-readable size
    vram_mb: Option<u64>,   // VRAM in MB if GPU detected
    ram_mb: u64,            // system RAM in MB
    gpu_name: Option<String>,
    has_cuda: bool,
    has_metal: bool,
}

// ── Available projects ─────────────────────────────────────────────────

fn get_projects() -> Vec<Project> {
    vec![
        Project {
            id: "geometry-os".into(),
            name: "Geometry OS".into(),
            description: "GPU-native OS in Rust. Compute shader VM, Hilbert curve memory.".into(),
            company_id: "41e9e9c7-38b4-45a8-b2cc-c34206d7d86d".into(),
            plugins: vec!["paperclip.geometry-os".into()],
            repo_url: "https://github.com/jericho/geometry-os".into(),
        },
        Project {
            id: "ascii-world".into(),
            name: "ASCII World".into(),
            description: "ASCII-driven interfaces for autonomous agents.".into(),
            company_id: "523529a2-a20e-4cdd-86f3-9407155422c2".into(),
            plugins: vec!["paperclip.ascii-world".into()],
            repo_url: "https://github.com/jericho/ascii-world".into(),
        },
        Project {
            id: "aipm".into(),
            name: "AIPM".into(),
            description: "Autonomous AI project management loop.".into(),
            company_id: "2b005468-996b-4b71-88eb-41970af8d63a".into(),
            plugins: vec!["paperclip.aipm-outcome".into()],
            repo_url: "https://github.com/jericho/aipm".into(),
        },
    ]
}

// ── Hardware detection ─────────────────────────────────────────────────

fn detect_hardware() -> HardwareProfile {
    let (ram_mb, has_cuda, vram_mb, gpu_name) = detect_gpu_and_ram();
    let has_metal = detect_metal();

    let (tier, backend, model, model_size) = if has_cuda && vram_mb.unwrap_or(0) >= 4096 {
        ("gpu-powerful".into(), "ollama".into(), "qwen2.5-coder:7b".into(), "4.7 GB")
    } else if has_cuda && vram_mb.unwrap_or(0) >= 2048 {
        ("gpu-basic".into(), "ollama".into(), "qwen2.5-coder:3b".into(), "1.9 GB")
    } else if has_metal {
        // Apple Silicon -- unified memory, assume decent
        ("gpu-powerful".into(), "ollama".into(), "qwen2.5-coder:7b".into(), "4.7 GB")
    } else if ram_mb >= 8192 {
        ("cpu-only".into(), "ollama".into(), "qwen2.5-coder:1.5b".into(), "1.1 GB")
    } else {
        ("cloud-only".into(), "cloud".into(), "glm-4.5-air".into(), "0 (cloud)")
    };

    HardwareProfile {
        tier,
        backend,
        model,
        model_size: model_size.into(),
        vram_mb,
        ram_mb,
        gpu_name,
        has_cuda,
        has_metal,
    }
}

#[cfg(target_os = "linux")]
fn detect_gpu_and_ram() -> (u64, bool, Option<u64>, Option<String>) {
    let mut ram_mb: u64 = 4096; // default
    let mut has_cuda = false;
    let mut vram_mb: Option<u64> = None;
    let mut gpu_name: Option<String> = None;

    // Get RAM from /proc/meminfo
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                let kb: u64 = line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(4096 * 1024);
                ram_mb = kb / 1024;
                break;
            }
        }
    }

    // Check for NVIDIA GPU via nvidia-smi
    let (ok, out) = run_cmd("nvidia-smi", &["--query-gpu=memory.total,name", "--format=csv,noheader,nounits"]);
    if ok {
        has_cuda = true;
        // Parse "24576, NVIDIA RTX 5090" or similar
        let parts: Vec<&str> = out.split(',').collect();
        if let Some(vram_str) = parts.first() {
            vram_mb = vram_str.trim().parse().ok();
        }
        if let Some(name) = parts.get(1) {
            gpu_name = Some(name.trim().to_string());
        }
    }

    (ram_mb, has_cuda, vram_mb, gpu_name)
}

#[cfg(target_os = "macos")]
fn detect_gpu_and_ram() -> (u64, bool, Option<u64>, Option<String>) {
    let mut ram_mb: u64 = 8192;

    // macOS sysctl for RAM
    let (_, out) = run_cmd("sysctl", &["-n", "hw.memsize"]);
    if let Ok(bytes) = out.trim().parse::<u64>() {
        ram_mb = bytes / (1024 * 1024);
    }

    // macOS doesn't have CUDA, but Metal is checked separately
    // GPU name from system_profiler
    let (_, out) = run_cmd("system_profiler", &["SPDisplaysDataType"]);
    let gpu_name = out.lines()
        .find(|l| l.contains("Chipset Model") || l.contains("Chip:"))
        .map(|l| l.split(':').last().unwrap_or("Apple GPU").trim().to_string());

    (ram_mb, false, None, gpu_name)
}

#[cfg(target_os = "windows")]
fn detect_gpu_and_ram() -> (u64, bool, Option<u64>, Option<String>) {
    // On Windows, use nvidia-smi if available, otherwise basic detection
    let mut ram_mb: u64 = 8192;
    let mut has_cuda = false;
    let mut vram_mb: Option<u64> = None;
    let mut gpu_name: Option<String> = None;

    // Get RAM via wmic
    let (_, out) = run_cmd("wmic", &["OS", "get", "TotalVisibleMemorySize", "/value"]);
    for line in out.lines() {
        if line.starts_with("TotalVisibleMemorySize=") {
            let kb: u64 = line.split('=').nth(1).unwrap_or("0").trim().parse().unwrap_or(0);
            ram_mb = kb / 1024;
        }
    }

    // Check NVIDIA
    let (ok, out) = run_cmd("nvidia-smi", &["--query-gpu=memory.total,name", "--format=csv,noheader,nounits"]);
    if ok {
        has_cuda = true;
        let parts: Vec<&str> = out.split(',').collect();
        if let Some(vram_str) = parts.first() {
            vram_mb = vram_str.trim().parse().ok();
        }
        if let Some(name) = parts.get(1) {
            gpu_name = Some(name.trim().to_string());
        }
    }

    (ram_mb, has_cuda, vram_mb, gpu_name)
}

fn detect_metal() -> bool {
    #[cfg(target_os = "macos")]
    {
        let (ok, _) = run_cmd("system_profiler", &["SPDisplaysDataType"]);
        if ok {
            // Metal is always available on modern macOS with Apple Silicon
            return true;
        }
    }
    false
}

// ── Helpers ────────────────────────────────────────────────────────────

fn run_cmd(program: &str, args: &[&str]) -> (bool, String) {
    match std::process::Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let combined = if stdout.is_empty() { stderr } else { stdout };
            (output.status.success(), combined.trim().to_string())
        }
        Err(e) => (false, e.to_string()),
    }
}

fn get_version(program: &str, version_flag: &str) -> Option<String> {
    let (ok, out) = run_cmd(program, &[version_flag]);
    if ok {
        let line = out.lines().next().unwrap_or("");
        let cleaned = line.replace(program, "").trim().trim_start_matches('v').to_string();
        if cleaned.is_empty() { None } else { Some(cleaned) }
    } else {
        None
    }
}

fn which_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn resolve_resource(relative_path: &str) -> Option<String> {
    let candidates = vec![
        format!("../{}", relative_path),
        format!("./{}", relative_path),
    ];

    for candidate in &candidates {
        if std::path::Path::new(candidate).exists() {
            return Some(std::fs::canonicalize(candidate).ok()?.to_string_lossy().to_string());
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let installed = format!("{}/.local/share/spawn/{}", home, relative_path);
        if std::path::Path::new(&installed).exists() {
            return Some(installed);
        }
    }

    None
}

fn emit_step(window: &Window, step: &str, success: bool, message: &str) {
    let payload = StepResult {
        step: step.into(),
        success,
        message: message.into(),
    };
    let _ = window.emit("setup-step", &payload);
}

async fn try_cloud_register(base_url: &str) -> Result<RegisterResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/register", base_url))
        .json(&serde_json::json!({
            "device_id": uuid::Uuid::new_v4().to_string(),
            "product": "spawn"
        }))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Server returned {}: {}", status, body));
    }

    resp.json::<RegisterResponse>()
        .await
        .map_err(|e| format!("Bad response: {}", e))
}

fn open_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/C", "start", url])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ── Commands ───────────────────────────────────────────────────────────

#[tauri::command]
fn list_projects() -> Vec<Project> {
    get_projects()
}

#[tauri::command]
fn check_prerequisites() -> Vec<PrereqCheck> {
    vec![
        PrereqCheck {
            name: "git".into(),
            installed: which_exists("git"),
            version: get_version("git", "--version"),
            required: "any".into(),
            install_cmd: Some("sudo apt install git".into()),
        },
        PrereqCheck {
            name: "curl".into(),
            installed: which_exists("curl"),
            version: get_version("curl", "--version"),
            required: "any".into(),
            install_cmd: Some("sudo apt install curl".into()),
        },
    ]
}

#[tauri::command]
fn detect_system() -> HardwareProfile {
    detect_hardware()
}

#[tauri::command]
async fn connect_cloud() -> Result<CloudStatus, String> {
    let result = match try_cloud_register(CLOUD_API_BASE).await {
        Ok(r) => Ok(r),
        Err(_) => try_cloud_register(CLOUD_API_FALLBACK).await,
    };

    match result {
        Ok(reg) => Ok(CloudStatus {
            connected: true,
            token: Some(reg.token),
            daily_limit: Some(reg.daily_limit),
            error: None,
        }),
        Err(e) => Ok(CloudStatus {
            connected: false,
            token: None,
            daily_limit: None,
            error: Some(e),
        }),
    }
}

#[tauri::command]
async fn run_setup(
    window: Window,
    project_id: String,
    cloud_token: Option<String>,
    manual_api_key: Option<String>,
    skip_ollama: Option<bool>,
) -> Result<Vec<StepResult>, String> {
    let mut results = Vec::new();
    let projects = get_projects();
    let project = projects
        .iter()
        .find(|p| p.id == project_id)
        .ok_or("Unknown project")?;

    let hw = detect_hardware();
    let use_ollama = !skip_ollama.unwrap_or(false) && hw.backend == "ollama";
    let has_cloud = cloud_token.is_some();

    // Determine the API key string
    let api_key = if let Some(ref token) = cloud_token {
        format!("cloud:{}", token)
    } else if let Some(ref key) = manual_api_key {
        key.clone()
    } else {
        String::new()
    };

    // ── Step 1: Install Hermes ───────────────────────────────────
    emit_step(&window, "Installing Hermes Agent...", true, "");
    if which_exists("hermes") {
        let ver = get_version("hermes", "--version").unwrap_or_default();
        results.push(StepResult {
            step: "Hermes Agent".into(),
            success: true,
            message: format!("Already installed ({})", ver),
        });
    } else {
        let (ok, out) = run_cmd("bash", &[
            "-c",
            &format!("curl -fsSL {} | bash -s -- --skip-setup", HERMES_INSTALL_URL),
        ]);
        if ok {
            results.push(StepResult {
                step: "Hermes Agent".into(),
                success: true,
                message: "Installed successfully".into(),
            });
        } else {
            let msg = format!("Install failed: {}", out);
            emit_step(&window, "Hermes Agent", false, &msg);
            results.push(StepResult { step: "Hermes Agent".into(), success: false, message: msg });
            return Ok(results);
        }
    }

    // ── Step 2: Install Ollama + pull model ──────────────────────
    if use_ollama {
        // Install Ollama if not present
        emit_step(&window, "Installing Ollama...", true, "");
        if which_exists("ollama") {
            results.push(StepResult {
                step: "Ollama".into(),
                success: true,
                message: format!("Already installed ({})", get_version("ollama", "--version").unwrap_or_default()),
            });
        } else {
            let (ok, out) = run_cmd("bash", &[
                "-c",
                "curl -fsSL https://ollama.com/install.sh | sh",
            ]);
            if ok {
                results.push(StepResult {
                    step: "Ollama".into(),
                    success: true,
                    message: "Installed successfully".into(),
                });
            } else {
                results.push(StepResult {
                    step: "Ollama".into(),
                    success: false,
                    message: format!("Install failed: {}. Falling back to cloud.", out),
                });
                // Don't return -- fall back to cloud
            }
        }

        // Start Ollama service if not running
        if which_exists("ollama") {
            let _ = run_cmd("ollama", &["serve"]);
            // Give it a moment to start
            std::thread::sleep(std::time::Duration::from_secs(2));

            // Pull the model
            emit_step(&window, &format!("Downloading {} ({} may take a few minutes)...", hw.model, hw.model_size), true, "");
            let (ok, out) = run_cmd("ollama", &["pull", &hw.model]);
            if ok {
                results.push(StepResult {
                    step: "AI Model".into(),
                    success: true,
                    message: format!("{} downloaded", hw.model),
                });
            } else {
                results.push(StepResult {
                    step: "AI Model".into(),
                    success: false,
                    message: format!("Download failed: {}. Using cloud fallback.", out),
                });
            }
        }
    }

    // ── Step 3: Configure Hermes ─────────────────────────────────
    emit_step(&window, "Configuring Hermes...", true, "");
    let home = std::env::var("HOME").unwrap_or_default();
    let env_path = format!("{}/.hermes/.env", home);
    let existing = std::fs::read_to_string(&env_path).unwrap_or_default();
    let mut lines: Vec<String> = existing.lines().map(|l| l.to_string()).collect();

    // Determine provider config
    let ollama_running = use_ollama && which_exists("ollama");

    if ollama_running {
        // Use Ollama as primary provider
        lines = lines.into_iter()
            .filter(|l| !l.starts_with("OPENAI_API_KEY=") && !l.starts_with("OPENAI_BASE_URL=") && !l.starts_with("LLM_MODEL="))
            .collect();
        lines.push("OPENAI_BASE_URL=http://localhost:11434/v1".into());
        lines.push("OPENAI_API_KEY=ollama".into());  // Ollama doesn't need a real key
        lines.push(format!("LLM_MODEL={}", hw.model));

        // If we also have cloud, save it as fallback
        if has_cloud {
            if let Some(ref token) = cloud_token {
                lines.push(format!("SPAWN_CLOUD_TOKEN={}", token));
                lines.push(format!("SPAWN_CLOUD_URL={}/v1", CLOUD_API_BASE));
            }
        }

        results.push(StepResult {
            step: "Hermes Config".into(),
            success: true,
            message: format!("Connected to local Ollama ({}){}", hw.model, if has_cloud { " + cloud backup" } else { "" }),
        });
    } else if has_cloud {
        // Cloud-only mode
        lines = lines.into_iter()
            .filter(|l| !l.starts_with("OPENAI_API_KEY=") && !l.starts_with("OPENAI_BASE_URL=") && !l.starts_with("LLM_MODEL="))
            .collect();
        lines.push(format!("OPENAI_API_KEY={}", api_key));
        lines.push(format!("OPENAI_BASE_URL={}/v1", CLOUD_API_BASE));
        lines.push("LLM_MODEL=glm-4.5-air".into());

        results.push(StepResult {
            step: "Hermes Config".into(),
            success: true,
            message: "Connected via Spawn Cloud (free, 50 messages/day)".into(),
        });
    } else if api_key.starts_with("sk-") {
        // User's own OpenAI key
        lines = lines.into_iter()
            .filter(|l| !l.starts_with("OPENAI_API_KEY=") && !l.starts_with("LLM_MODEL="))
            .collect();
        lines.push(format!("OPENAI_API_KEY={}", api_key));
        lines.push("LLM_MODEL=gpt-4o".into());

        results.push(StepResult {
            step: "Hermes Config".into(),
            success: true,
            message: "API key configured".into(),
        });
    } else {
        results.push(StepResult {
            step: "Hermes Config".into(),
            success: false,
            message: "No AI backend available. Run 'hermes setup' after install.".into(),
        });
    }

    std::fs::write(&env_path, lines.join("\n") + "\n")
        .map_err(|e| format!("Failed to write .env: {}", e))?;

    // ── Step 4: Install Paperclip (optional) ─────────────────────
    emit_step(&window, "Setting up Paperclip...", true, "");
    if which_exists("node") && which_exists("npx") {
        let (ok, _out) = run_cmd("npx", &["paperclipai", "onboard", "--yes"]);
        results.push(StepResult {
            step: "Paperclip".into(),
            success: ok,
            message: if ok { "Paperclip ready".into() } else { "Paperclip setup skipped (optional)".into() },
        });
    } else {
        results.push(StepResult {
            step: "Paperclip".into(),
            success: true,
            message: "Skipped (Node.js not found, optional)".into(),
        });
    }

    // ── Step 5: Import company template + plugins ────────────────
    if which_exists("npx") {
        emit_step(&window, "Importing project...", true, "");
        let template_path = resolve_resource(&format!("templates/{}", project_id));

        if let Some(tp) = template_path {
            let (ok, _out) = run_cmd("npx", &[
                "paperclipai", "company", "import", &tp, "--yes",
            ]);
            results.push(StepResult {
                step: "Project Import".into(),
                success: ok,
                message: if ok { format!("{} imported", project.name) } else { "Import skipped".into() },
            });
        } else {
            let (ok, _out) = run_cmd("npx", &[
                "paperclipai", "company", "import",
                &project.repo_url, "--ref", "main", "--yes",
            ]);
            results.push(StepResult {
                step: "Project Import".into(),
                success: ok,
                message: if ok { format!("{} imported (from GitHub)", project.name) } else { "Import skipped".into() },
            });
        }

        for plugin in &project.plugins {
            emit_step(&window, &format!("Plugin {}...", plugin), true, "");
            let (ok, _out) = run_cmd("npx", &["paperclipai", "plugin", "install", plugin]);
            results.push(StepResult {
                step: format!("Plugin: {}", plugin),
                success: ok,
                message: if ok { format!("{} installed", plugin) } else { "Skipped".into() },
            });
        }
    }

    // ── Step 6: Clone project repo ────────────────────────────────
    emit_step(&window, "Cloning project...", true, "");
    let project_dir = format!("{}/zion/projects/{}", home, project_id);
    if !std::path::Path::new(&project_dir).exists() {
        let _ = std::fs::create_dir_all(format!("{}/zion/projects", home));
        let (ok, _out) = run_cmd("git", &["clone", &project.repo_url, &project_dir]);
        results.push(StepResult {
            step: "Project Clone".into(),
            success: ok,
            message: if ok { format!("Cloned to {}", project_dir) } else { "Clone failed (you can clone manually)".into() },
        });
    } else {
        results.push(StepResult {
            step: "Project Clone".into(),
            success: true,
            message: "Already exists".into(),
        });
    }

    // ── Step 7: Done ──────────────────────────────────────────────
    emit_step(&window, "Done!", true, "");

    let summary = if ollama_running {
        format!("Local AI ready ({}). Run: hermes chat", hw.model)
    } else if has_cloud {
        "Cloud AI ready (50 free msgs/day). Run: hermes chat".into()
    } else {
        "Run 'hermes setup' to add an AI provider. Run: hermes chat".into()
    };

    results.push(StepResult {
        step: "Ready".into(),
        success: true,
        message: summary,
    });

    Ok(results)
}

// ── Init ───────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            list_projects,
            check_prerequisites,
            detect_system,
            connect_cloud,
            run_setup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
