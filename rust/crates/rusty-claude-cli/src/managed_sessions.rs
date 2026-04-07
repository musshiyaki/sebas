use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use runtime::{
    sessions_dir as surface_sessions_dir, LEGACY_STATE_DIR, Session, PRIMARY_CLI_NAME,
};

pub const PRIMARY_SESSION_EXTENSION: &str = "jsonl";
pub const LEGACY_SESSION_EXTENSION: &str = "json";
pub const LATEST_SESSION_REFERENCE: &str = "latest";
pub const SESSION_REFERENCE_ALIASES: &[&str] = &[LATEST_SESSION_REFERENCE, "last", "recent"];

#[derive(Debug, Clone)]
pub struct SessionHandle {
    pub id: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ManagedSessionSummary {
    pub id: String,
    pub path: PathBuf,
    pub modified_epoch_millis: u128,
    pub message_count: usize,
    pub parent_session_id: Option<String>,
    pub branch_name: Option<String>,
}

pub fn sessions_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cwd = env::current_dir()?;
    let path = surface_sessions_dir(&cwd);
    fs::create_dir_all(&path)?;
    Ok(path)
}

pub fn create_managed_session_handle(
    session_id: &str,
) -> Result<SessionHandle, Box<dyn std::error::Error>> {
    let id = session_id.to_string();
    let path = sessions_dir()?.join(format!("{id}.{PRIMARY_SESSION_EXTENSION}"));
    Ok(SessionHandle { id, path })
}

pub fn resolve_session_reference(
    reference: &str,
) -> Result<SessionHandle, Box<dyn std::error::Error>> {
    if SESSION_REFERENCE_ALIASES
        .iter()
        .any(|alias| reference.eq_ignore_ascii_case(alias))
    {
        let latest = latest_managed_session()?;
        return Ok(SessionHandle {
            id: latest.id,
            path: latest.path,
        });
    }

    let direct = PathBuf::from(reference);
    let looks_like_path = direct.extension().is_some() || direct.components().count() > 1;
    let path = if direct.exists() {
        direct
    } else if looks_like_path {
        return Err(format_missing_session_reference(reference).into());
    } else {
        resolve_managed_session_path(reference)?
    };
    let id = path
        .file_name()
        .and_then(|value| value.to_str())
        .and_then(|name| {
            name.strip_suffix(&format!(".{PRIMARY_SESSION_EXTENSION}"))
                .or_else(|| name.strip_suffix(&format!(".{LEGACY_SESSION_EXTENSION}")))
        })
        .unwrap_or(reference)
        .to_string();
    Ok(SessionHandle { id, path })
}

pub fn latest_managed_session() -> Result<ManagedSessionSummary, Box<dyn std::error::Error>> {
    list_managed_sessions()?
        .into_iter()
        .next()
        .ok_or_else(|| format_no_managed_sessions().into())
}

pub fn render_session_list(
    active_session_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let sessions = list_managed_sessions()?;
    let mut lines = vec![
        "Sessions".to_string(),
        format!("  Directory         {}", sessions_dir()?.display()),
    ];
    if sessions.is_empty() {
        lines.push("  No managed sessions saved yet.".to_string());
        return Ok(lines.join("\n"));
    }
    for session in sessions {
        let marker = if session.id == active_session_id {
            "● current"
        } else {
            "○ saved"
        };
        let lineage = match (
            session.branch_name.as_deref(),
            session.parent_session_id.as_deref(),
        ) {
            (Some(branch_name), Some(parent_session_id)) => {
                format!(" branch={branch_name} from={parent_session_id}")
            }
            (None, Some(parent_session_id)) => format!(" from={parent_session_id}"),
            (Some(branch_name), None) => format!(" branch={branch_name}"),
            (None, None) => String::new(),
        };
        lines.push(format!(
            "  {id:<20} {marker:<10} msgs={msgs:<4} modified={modified}{lineage} path={path}",
            id = session.id,
            msgs = session.message_count,
            modified = format_session_modified_age(session.modified_epoch_millis),
            lineage = lineage,
            path = session.path.display(),
        ));
    }
    Ok(lines.join("\n"))
}

pub fn format_session_modified_age(modified_epoch_millis: u128) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map_or(modified_epoch_millis, |duration| duration.as_millis());
    let delta_seconds = now
        .saturating_sub(modified_epoch_millis)
        .checked_div(1_000)
        .unwrap_or_default();
    match delta_seconds {
        0..=4 => "just-now".to_string(),
        5..=59 => format!("{delta_seconds}s-ago"),
        60..=3_599 => format!("{}m-ago", delta_seconds / 60),
        3_600..=86_399 => format!("{}h-ago", delta_seconds / 3_600),
        _ => format!("{}d-ago", delta_seconds / 86_400),
    }
}

fn resolve_managed_session_path(session_id: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    for directory in compatible_session_dirs()? {
        for extension in [PRIMARY_SESSION_EXTENSION, LEGACY_SESSION_EXTENSION] {
            let path = directory.join(format!("{session_id}.{extension}"));
            if path.exists() {
                return Ok(path);
            }
        }
    }
    Err(format_missing_session_reference(session_id).into())
}

fn is_managed_session_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|extension| {
            extension == PRIMARY_SESSION_EXTENSION || extension == LEGACY_SESSION_EXTENSION
        })
}

pub fn list_managed_sessions() -> Result<Vec<ManagedSessionSummary>, Box<dyn std::error::Error>> {
    let mut sessions: Vec<ManagedSessionSummary> = Vec::new();
    for directory in compatible_session_dirs()? {
        if !directory.is_dir() {
            continue;
        }
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            if !is_managed_session_file(&path)
                || sessions.iter().any(|session| session.path == path)
            {
                continue;
            }
            let metadata = entry.metadata()?;
            let modified_epoch_millis = metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_millis())
                .unwrap_or_default();
            let (id, message_count, parent_session_id, branch_name) =
                match Session::load_from_path(&path) {
                    Ok(session) => {
                        let parent_session_id = session
                            .fork
                            .as_ref()
                            .map(|fork| fork.parent_session_id.clone());
                        let branch_name = session
                            .fork
                            .as_ref()
                            .and_then(|fork| fork.branch_name.clone());
                        (
                            session.session_id,
                            session.messages.len(),
                            parent_session_id,
                            branch_name,
                        )
                    }
                    Err(_) => (
                        path.file_stem()
                            .and_then(|value| value.to_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        0,
                        None,
                        None,
                    ),
                };
            sessions.push(ManagedSessionSummary {
                id,
                path,
                modified_epoch_millis,
                message_count,
                parent_session_id,
                branch_name,
            });
        }
    }
    sessions.sort_by(|left, right| {
        right
            .modified_epoch_millis
            .cmp(&left.modified_epoch_millis)
            .then_with(|| right.id.cmp(&left.id))
    });
    Ok(sessions)
}

fn compatible_session_dirs() -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let cwd = env::current_dir()?;
    Ok(vec![
        surface_sessions_dir(&cwd),
        cwd.join(LEGACY_STATE_DIR).join("sessions"),
    ])
}

fn format_missing_session_reference(reference: &str) -> String {
    format!(
        "session not found: {reference}\nHint: managed sessions live in .codex/sessions/. Try `{LATEST_SESSION_REFERENCE}` for the most recent session or `/session list` in the interactive agent."
    )
}

fn format_no_managed_sessions() -> String {
    format!(
        "no managed sessions found in .codex/sessions/\nStart `{PRIMARY_CLI_NAME}` to create a session, then rerun with `--resume {LATEST_SESSION_REFERENCE}`."
    )
}
