use std::io;
use std::path::{Path, PathBuf};

pub const PRIMARY_CLI_NAME: &str = "sebas";
pub const CLI_ALIASES: &[&str] = &["codex"];
pub const CANONICAL_STATE_DIR: &str = ".codex";
pub const LEGACY_STATE_DIR: &str = ".claw";
pub const COMPAT_STATE_DIR: &str = ".claude";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigLayer {
    User,
    Project,
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigPath {
    pub layer: ConfigLayer,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DefinitionSource {
    ProjectCodex,
    ProjectClaw,
    ProjectClaude,
    UserCodexHome,
    UserCodex,
    UserClaw,
    UserClaude,
}

impl DefinitionSource {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ProjectCodex => "Project (.codex)",
            Self::ProjectClaw => "Project (.claw)",
            Self::ProjectClaude => "Project (.claude)",
            Self::UserCodexHome => "User ($CODEX_HOME)",
            Self::UserCodex => "User (~/.codex)",
            Self::UserClaw => "User (~/.claw)",
            Self::UserClaude => "User (~/.claude)",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionRoot {
    pub source: DefinitionSource,
    pub path: PathBuf,
}

#[must_use]
pub fn default_config_home() -> PathBuf {
    std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(CANONICAL_STATE_DIR))
        })
        .unwrap_or_else(|| PathBuf::from(CANONICAL_STATE_DIR))
}

#[must_use]
pub fn project_state_dir(cwd: &Path) -> PathBuf {
    cwd.join(CANONICAL_STATE_DIR)
}

#[must_use]
pub fn sessions_dir(cwd: &Path) -> PathBuf {
    project_state_dir(cwd).join("sessions")
}

pub fn credentials_home_dir() -> io::Result<PathBuf> {
    if let Some(path) = std::env::var_os("CODEX_HOME") {
        return Ok(PathBuf::from(path));
    }
    let home = std::env::var_os("HOME")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
    Ok(PathBuf::from(home).join(CANONICAL_STATE_DIR))
}

#[must_use]
pub fn config_paths(cwd: &Path, config_home: &Path) -> Vec<ConfigPath> {
    let user_parent = config_home
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);

    let mut paths = Vec::new();
    for (layer, path) in [
        (ConfigLayer::User, config_home.join("settings.json")),
        (
            ConfigLayer::User,
            user_parent.join(format!("{LEGACY_STATE_DIR}.json")),
        ),
        (
            ConfigLayer::User,
            user_parent.join(LEGACY_STATE_DIR).join("settings.json"),
        ),
        (
            ConfigLayer::User,
            user_parent.join(COMPAT_STATE_DIR).join("settings.json"),
        ),
        (
            ConfigLayer::Project,
            cwd.join(CANONICAL_STATE_DIR).join("settings.json"),
        ),
        (
            ConfigLayer::Project,
            cwd.join(format!("{LEGACY_STATE_DIR}.json")),
        ),
        (
            ConfigLayer::Project,
            cwd.join(LEGACY_STATE_DIR).join("settings.json"),
        ),
        (
            ConfigLayer::Project,
            cwd.join(COMPAT_STATE_DIR).join("settings.json"),
        ),
        (
            ConfigLayer::Local,
            cwd.join(CANONICAL_STATE_DIR).join("settings.local.json"),
        ),
        (
            ConfigLayer::Local,
            cwd.join(LEGACY_STATE_DIR).join("settings.local.json"),
        ),
    ] {
        push_unique_config_path(&mut paths, layer, path);
    }
    paths
}

#[must_use]
pub fn instruction_candidates(dir: &Path) -> Vec<PathBuf> {
    vec![
        dir.join("CLAUDE.md"),
        dir.join("CLAUDE.local.md"),
        dir.join(CANONICAL_STATE_DIR).join("CLAUDE.md"),
        dir.join(CANONICAL_STATE_DIR).join("instructions.md"),
        dir.join(LEGACY_STATE_DIR).join("CLAUDE.md"),
        dir.join(LEGACY_STATE_DIR).join("instructions.md"),
        dir.join(COMPAT_STATE_DIR).join("CLAUDE.md"),
        dir.join(COMPAT_STATE_DIR).join("instructions.md"),
    ]
}

#[must_use]
pub fn discover_definition_roots(cwd: &Path, leaf: &str) -> Vec<DefinitionRoot> {
    let mut roots = Vec::new();

    for ancestor in cwd.ancestors() {
        push_unique_root(
            &mut roots,
            DefinitionSource::ProjectCodex,
            ancestor.join(CANONICAL_STATE_DIR).join(leaf),
        );
        push_unique_root(
            &mut roots,
            DefinitionSource::ProjectClaw,
            ancestor.join(LEGACY_STATE_DIR).join(leaf),
        );
        push_unique_root(
            &mut roots,
            DefinitionSource::ProjectClaude,
            ancestor.join(COMPAT_STATE_DIR).join(leaf),
        );
    }

    if let Ok(codex_home) = std::env::var("CODEX_HOME") {
        push_unique_root(
            &mut roots,
            DefinitionSource::UserCodexHome,
            PathBuf::from(codex_home).join(leaf),
        );
    }

    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        push_unique_root(
            &mut roots,
            DefinitionSource::UserCodex,
            home.join(CANONICAL_STATE_DIR).join(leaf),
        );
        push_unique_root(
            &mut roots,
            DefinitionSource::UserClaw,
            home.join(LEGACY_STATE_DIR).join(leaf),
        );
        push_unique_root(
            &mut roots,
            DefinitionSource::UserClaude,
            home.join(COMPAT_STATE_DIR).join(leaf),
        );
    }

    roots
}

fn push_unique_root(roots: &mut Vec<DefinitionRoot>, source: DefinitionSource, path: PathBuf) {
    if roots.iter().any(|root| root.path == path) {
        return;
    }
    roots.push(DefinitionRoot { source, path });
}

fn push_unique_config_path(paths: &mut Vec<ConfigPath>, layer: ConfigLayer, path: PathBuf) {
    if paths.iter().any(|entry| entry.path == path) {
        return;
    }
    paths.push(ConfigPath { layer, path });
}

#[cfg(test)]
mod tests {
    use super::{
        config_paths, discover_definition_roots, instruction_candidates, ConfigLayer,
        DefinitionSource, CANONICAL_STATE_DIR,
    };
    use std::path::Path;

    #[test]
    fn prefers_codex_paths_before_legacy_paths() {
        let cwd = Path::new("/tmp/project");
        let config_home = Path::new("/tmp/home/.codex");
        let paths = config_paths(cwd, config_home);

        assert_eq!(
            paths
                .iter()
                .take(4)
                .map(|entry| (&entry.layer, entry.path.as_path()))
                .collect::<Vec<_>>(),
            vec![
                (&ConfigLayer::User, Path::new("/tmp/home/.codex/settings.json")),
                (&ConfigLayer::User, Path::new("/tmp/home/.claw.json")),
                (&ConfigLayer::User, Path::new("/tmp/home/.claw/settings.json")),
                (&ConfigLayer::User, Path::new("/tmp/home/.claude/settings.json")),
            ]
        );
        assert_eq!(paths[4].path, Path::new("/tmp/project/.codex/settings.json"));
    }

    #[test]
    fn advertises_codex_first_instruction_candidates() {
        let candidates = instruction_candidates(Path::new("/tmp/project"));
        assert_eq!(candidates[0], Path::new("/tmp/project/CLAUDE.md"));
        assert_eq!(candidates[2], Path::new("/tmp/project/.codex/CLAUDE.md"));
        assert_eq!(candidates[4], Path::new("/tmp/project/.claw/CLAUDE.md"));
        assert_eq!(candidates[6], Path::new("/tmp/project/.claude/CLAUDE.md"));
    }

    #[test]
    fn definition_root_labels_match_existing_reports() {
        let roots = discover_definition_roots(Path::new("/tmp/project"), "agents");
        assert_eq!(roots[0].source, DefinitionSource::ProjectCodex);
        assert_eq!(roots[0].source.label(), "Project (.codex)");
        assert!(
            roots
                .iter()
                .any(|root| root.path.ends_with(Path::new(CANONICAL_STATE_DIR).join("agents")))
        );
    }
}
