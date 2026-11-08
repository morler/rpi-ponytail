#![allow(clippy::not_unsafe_ptr_arg_deref)]
use rpi_plugin_sdk::{callbacks::free_string, StbStringRef};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};

const SKILL: &str = include_str!("ponytail.md");
static MODE: OnceLock<RwLock<String>> = OnceLock::new();

fn valid_mode(value: &str) -> Option<&str> {
    match value {
        "off" | "lite" | "full" | "ultra" => Some(value),
        _ => None,
    }
}

fn config_path() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            if cfg!(windows) {
                std::env::var_os("APPDATA")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            } else {
                std::env::var_os("HOME")
                    .filter(|v| !v.is_empty())
                    .map(|h| PathBuf::from(h).join(".config"))
            }
        })
        .map(|dir| dir.join("ponytail/config.json"))
}

fn env_mode() -> Option<String> {
    let env = std::env::var("PONYTAIL_DEFAULT_MODE")
        .unwrap_or_default()
        .trim()
        .to_lowercase();
    valid_mode(&env).map(str::to_owned)
}

fn parse_mode(content: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(content.trim_start_matches('\u{feff}'))
        .ok()
        .and_then(|v| v["defaultMode"].as_str().map(|s| s.trim().to_lowercase()))
        .filter(|s| valid_mode(s).is_some())
}

fn file_mode(path: &Path) -> Option<String> {
    parse_mode(&std::fs::read_to_string(path).ok()?)
}

fn default_mode() -> String {
    env_mode()
        .or_else(|| config_path().and_then(|path| file_mode(&path)))
        .unwrap_or_else(|| "full".into())
}

/// Merge `mode` into the existing JSON object and replace the file atomically, so
/// a failed write never leaves a half-written config behind.
fn save_mode(path: &Path, mode: &str) -> io::Result<()> {
    let mut config = match std::fs::read_to_string(path) {
        Ok(content) => {
            serde_json::from_str::<serde_json::Value>(content.trim_start_matches('\u{feff}'))
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => serde_json::json!({}),
        Err(e) => return Err(e),
    };
    let object = config
        .as_object_mut()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "config is not a JSON object"))?;
    object.insert("defaultMode".into(), mode.into());
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "config has no parent"))?;
    std::fs::create_dir_all(parent)?;
    let temp = parent.join("config.json.tmp");
    let mut file = std::fs::File::create(&temp)?;
    serde_json::to_writer_pretty(&mut file, &config)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(&temp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&temp);
    })
}

fn save_default(mode: &str) -> io::Result<String> {
    let path = config_path()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no Ponytail config directory"))?;
    save_mode(&path, mode)?;
    Ok(match env_mode().filter(|env| env != mode) {
        Some(env) => format!(
            "Ponytail mode set to {mode}. PONYTAIL_DEFAULT_MODE keeps the default at {env}."
        ),
        None => format!("Ponytail mode set to {mode} and saved."),
    })
}

fn mode() -> &'static RwLock<String> {
    MODE.get_or_init(|| RwLock::new(default_mode()))
}

/// True for a Levels-table row of a level other than the active one; every
/// other line (rules, the active row) survives verbatim.
fn is_other_level_row(line: &str, mode: &str) -> bool {
    ["lite", "full", "ultra"]
        .iter()
        .any(|level| level != &mode && line.starts_with(&format!("| **{level}** |")))
}

fn instructions(mode: &str) -> String {
    let body = SKILL
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("---\n").map(|(_, body)| body))
        .unwrap_or(SKILL);
    let body = body
        .lines()
        .filter(|line| !is_other_level_row(line, mode))
        .collect::<Vec<_>>()
        .join("\n");
    format!("PONYTAIL MODE ACTIVE — level: {mode}\n\n{body}")
}

/// Returns the next mode (persisted by the caller) and the reply text. An empty
/// text means the caller reports the outcome of the write.
fn command(args: &str, current: &str, default: &str) -> (Option<String>, String) {
    let arg = args.trim().to_lowercase();
    if arg.is_empty() && current == "off" {
        let next = if default == "off" { "full" } else { default };
        return (Some(next.into()), String::new());
    }
    if arg.is_empty() || arg == "status" {
        return (
            None,
            format!("Ponytail: current {current} • default {default}"),
        );
    }
    if let Some(next) = valid_mode(&arg) {
        return (Some(next.into()), String::new());
    }
    (None, "Usage: /ponytail [lite|full|ultra|off|status]".into())
}

rpi_plugin_sdk::string_handler! {
    fn on_command(input, out, _user_data) {
        let value: serde_json::Value = serde_json::from_str(input).unwrap_or_default();
        let args = value["args"].as_str().unwrap_or("");
        let mut current = mode().write().unwrap_or_else(|e| e.into_inner());
        let (next, mut text) = command(args, &current, &default_mode());
        if let Some(next) = next {
            match save_default(&next) {
                Ok(message) => {
                    *current = next;
                    text = message;
                }
                Err(e) => text = format!("Failed to save Ponytail mode: {e}"),
            }
        }
        i32::from(!out.write(serde_json::json!({"kind":"message", "text":text}).to_string()))
    }
}
rpi_plugin_sdk::string_handler! {
    fn on_start(input, out, _user_data) {
        let value: serde_json::Value = match serde_json::from_str(input) { Ok(v) => v, Err(_) => return 1 };
        let current = mode().read().unwrap_or_else(|e| e.into_inner());
        if *current == "off" { return i32::from(!out.write("{}".to_string())); }
        let base = value["systemPrompt"].as_str().unwrap_or("");
        let prompt = format!("{base}\n\n{}", instructions(&current));
        i32::from(!out.write(serde_json::json!({"systemPrompt":prompt}).to_string()))
    }
}
rpi_plugin_sdk::export_plugin!(|api| {
    let Some(register_prompt) = api.register_before_agent_start else {
        return 1;
    };
    let Some(register_command) = api.register_command else {
        return 1;
    };
    let rc = register_prompt(on_start, free_string, std::ptr::null_mut());
    if rc != 0 {
        return rc;
    }
    register_command(
        StbStringRef::from_str("ponytail"),
        StbStringRef::from_str("Set Ponytail mode: lite, full, ultra, off, status"),
        on_command,
    )
});

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("rpi-ponytail-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn modes_and_status() {
        assert_eq!(command("", "off", "off").0.as_deref(), Some("full"));
        assert_eq!(
            command(" ULTRA ", "full", "full").0.as_deref(),
            Some("ultra")
        );
        assert!(command("review", "full", "full").0.is_none());
        assert!(command("status", "full", "lite").1.contains("current full"));
    }

    #[test]
    fn filters_other_levels() {
        let text = instructions("lite");
        assert!(text.contains("| **lite** |"));
        assert!(!text.contains("| **ultra** |"));
        assert!(text.contains("Never cut:"));
    }

    #[test]
    fn save_mode_creates_missing_file_and_keeps_other_keys() {
        let dir = temp_dir("create");
        let path = dir.join("ponytail/config.json");
        save_mode(&path, "lite").unwrap();
        assert_eq!(file_mode(&path).as_deref(), Some("lite"));

        std::fs::write(&path, r#"{"defaultMode":"full","quietStartup":true}"#).unwrap();
        save_mode(&path, "ultra").unwrap();
        let saved: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(saved["defaultMode"], "ultra");
        assert_eq!(saved["quietStartup"], true);
        assert!(!dir.join("ponytail/config.json.tmp").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn invalid_config_is_reported_and_left_untouched() {
        let dir = temp_dir("invalid");
        let path = dir.join("ponytail/config.json");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        for bad in ["not json", "[1,2]"] {
            std::fs::write(&path, bad).unwrap();
            assert!(save_mode(&path, "lite").is_err());
            assert_eq!(std::fs::read_to_string(&path).unwrap(), bad);
        }
        assert_eq!(
            parse_mode("\u{feff}{\"defaultMode\":\" OFF \"}").as_deref(),
            Some("off")
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
