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

fn run_cmd_env(program: &str, args: &[&str], env_vars: &[(&str, &str)]) -> (bool, String) {
    let mut cmd = std::process::Command::new(program);
    cmd.args(args);
    for (k, v) in env_vars {
        cmd.env(k, v);
    }
    match cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).output() {
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

/// Resolve a bundled resource path. In dev: relative to project root.
/// In production: relative to the resource dir inside the app bundle.
fn resolve_resource(relative_path: &str) -> Option<String> {
    // Try as-is first (dev mode: relative to CWD or src-tauri)
    let candidates = vec![
        format!("../{}", relative_path),
        format!("./{}", relative_path),
    ];

    for candidate in &candidates {
        if std::path::Path::new(candidate).exists() {
            return Some(std::fs::canonicalize(candidate).ok()?.to_string_lossy().to_string());
        }
    }

    // Try XDG data dir (Linux .deb install)
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
        // Hermes installer handles Python + Node + everything else
    ]
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
) -> Result<Vec<StepResult>, String> {
    let mut results = Vec::new();
    let projects = get_projects();
    let project = projects
        .iter()
        .find(|p| p.id == project_id)
        .ok_or("Unknown project")?;

    let has_cloud = cloud_token.is_some();
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
        // Hermes install script handles Python, Node, everything
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

    // ── Step 2: Configure Hermes model + API key ─────────────────
    emit_step(&window, "Configuring Hermes...", true, "");
    if !api_key.is_empty() {
        // Write the cloud token or API key to Hermes .env
        let home = std::env::var("HOME").unwrap_or_default();
        let env_path = format!("{}/.hermes/.env", home);

        // Read existing .env
        let existing = std::fs::read_to_string(&env_path).unwrap_or_default();

        // Build new .env with our key
        let mut lines: Vec<String> = existing.lines().map(|l| l.to_string()).collect();

        // Update or add the key
        if has_cloud {
            // Cloud proxy: set OPENAI_API_KEY to cloud token, OPENAI_BASE_URL to proxy
            lines = lines.into_iter().filter(|l| !l.starts_with("OPENAI_API_KEY=") && !l.starts_with("OPENAI_BASE_URL=")).collect();
            lines.push(format!("OPENAI_API_KEY={}", api_key));
            lines.push(format!("OPENAI_BASE_URL={}/v1", CLOUD_API_BASE));
        } else if api_key.starts_with("sk-") {
            lines = lines.into_iter().filter(|l| !l.starts_with("OPENAI_API_KEY=")).collect();
            lines.push(format!("OPENAI_API_KEY={}", api_key));
        }

        // Set model to a good default
        lines = lines.into_iter().filter(|l| !l.starts_with("LLM_MODEL=")).collect();
        if has_cloud {
            lines.push("LLM_MODEL=glm-4.5-air".into());
        } else {
            lines.push("LLM_MODEL=gpt-4o".into());
        }

        std::fs::write(&env_path, lines.join("\n") + "\n")
            .map_err(|e| format!("Failed to write .env: {}", e))?;

        // Also update config.yaml for the model
        let _ = run_cmd("hermes", &["config", "set", "model", if has_cloud { "gpt-4o-mini" } else { "gpt-4o" }]);

        results.push(StepResult {
            step: "Hermes Config".into(),
            success: true,
            message: if has_cloud {
                "Connected via Spawn Cloud (free)".into()
            } else {
                "API key configured".into()
            },
        });
    } else {
        results.push(StepResult {
            step: "Hermes Config".into(),
            success: false,
            message: "No API key. Run 'hermes setup' after install to add one.".into(),
        });
    }

    // ── Step 3: Install Paperclip (optional, for project management) ──
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

    // ── Step 4: Import company template + plugins ────────────────
    // Use bundled template (works offline) instead of GitHub URL
    if which_exists("npx") {
        emit_step(&window, "Importing project...", true, "");

        // Resolve template path from bundled resources
        let template_path = resolve_resource(&format!("templates/{}", project_id));

        if let Some(tp) = template_path {
            let (ok, _out) = run_cmd("npx", &[
                "paperclipai", "company", "import",
                &tp, "--yes",
            ]);
            results.push(StepResult {
                step: "Project Import".into(),
                success: ok,
                message: if ok { format!("{} imported", project.name) } else { "Import skipped".into() },
            });
        } else {
            // Fallback: try GitHub URL if template not bundled
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

    // ── Step 5: Clone project repo ────────────────────────────────
    emit_step(&window, "Cloning project...", true, "");
    let home = std::env::var("HOME").unwrap_or_default();
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

    // ── Step 6: Done ──────────────────────────────────────────────
    emit_step(&window, "Done!", true, "");

    results.push(StepResult {
        step: "Ready".into(),
        success: true,
        message: "Open a terminal and run: hermes chat".into(),
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
            connect_cloud,
            run_setup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
