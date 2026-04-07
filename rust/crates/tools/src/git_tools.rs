use serde::Serialize;

use runtime::{execute_bash, BashCommandInput};

use crate::{
    GitBranchInput, GitCommitInput, GitDiffInput, GitPullRequestInput, GitPushInput,
    GitStatusInput,
};

#[derive(Debug, Serialize)]
pub(crate) struct GitToolOutput {
    pub(crate) command: String,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

pub(crate) fn execute_git_status(_input: GitStatusInput) -> Result<GitToolOutput, String> {
    execute_git_command("git status --short --branch".to_string())
}

pub(crate) fn execute_git_diff(input: GitDiffInput) -> Result<GitToolOutput, String> {
    let mut command = String::from("git diff --no-ext-diff --stat");
    if input.staged.unwrap_or(false) {
        command.push_str(" --cached");
    }
    if let Some(pathspec) = input.pathspec.filter(|items| !items.is_empty()) {
        command.push_str(" --");
        for item in pathspec {
            command.push(' ');
            command.push_str(&shell_escape(&item));
        }
    }
    execute_git_command(command)
}

pub(crate) fn execute_git_commit(input: GitCommitInput) -> Result<GitToolOutput, String> {
    let message = input.message.trim();
    if message.is_empty() {
        return Err("git commit message must not be empty".to_string());
    }
    let mut command = String::from("git commit");
    if input.all.unwrap_or(false) {
        command.push_str(" -a");
    }
    command.push_str(" -m ");
    command.push_str(&shell_escape(message));
    execute_git_command(command)
}

pub(crate) fn execute_git_branch(input: GitBranchInput) -> Result<GitToolOutput, String> {
    let mut command = if input.checkout.unwrap_or(true) {
        String::from("git checkout")
    } else {
        String::from("git branch")
    };
    command.push(' ');
    if input.checkout.unwrap_or(true) && input.base.is_some() {
        command.push_str("-b ");
    }
    command.push_str(&shell_escape(input.name.trim()));
    if let Some(base) = input.base.filter(|value| !value.trim().is_empty()) {
        command.push(' ');
        command.push_str(&shell_escape(base.trim()));
    }
    execute_git_command(command)
}

pub(crate) fn execute_git_push(input: GitPushInput) -> Result<GitToolOutput, String> {
    let mut command = String::from("git push");
    if input.set_upstream.unwrap_or(true) {
        command.push_str(" --set-upstream");
    }
    command.push(' ');
    command.push_str(&shell_escape(input.remote.as_deref().unwrap_or("origin")));
    if let Some(branch) = input.branch.filter(|value| !value.trim().is_empty()) {
        command.push(' ');
        command.push_str(&shell_escape(branch.trim()));
    }
    execute_git_command(command)
}

pub(crate) fn execute_git_pull_request(
    input: GitPullRequestInput,
) -> Result<GitToolOutput, String> {
    let mut command = String::from("gh pr create");
    if input.draft.unwrap_or(true) {
        command.push_str(" --draft");
    }
    if let Some(title) = input.title.filter(|value| !value.trim().is_empty()) {
        command.push_str(" --title ");
        command.push_str(&shell_escape(title.trim()));
    }
    if let Some(body) = input.body.filter(|value| !value.trim().is_empty()) {
        command.push_str(" --body ");
        command.push_str(&shell_escape(body.trim()));
    }
    execute_git_command(command)
}

fn execute_git_command(command: String) -> Result<GitToolOutput, String> {
    let output = execute_bash(BashCommandInput {
        command: command.clone(),
        timeout: Some(60_000),
        description: Some("git tool".to_string()),
        run_in_background: Some(false),
        dangerously_disable_sandbox: Some(true),
        namespace_restrictions: Some(false),
        isolate_network: Some(false),
        filesystem_mode: None,
        allowed_mounts: None,
    })
    .map_err(|error| error.to_string())?;

    if let Some(return_code) = output.return_code_interpretation.as_deref() {
        if return_code != "timeout" {
            let stderr = output.stderr.trim();
            let stdout = output.stdout.trim();
            let summary = if !stderr.is_empty() {
                stderr
            } else if !stdout.is_empty() {
                stdout
            } else {
                return_code
            };
            return Err(format!("git command failed: {summary}"));
        }
    }

    Ok(GitToolOutput {
        command,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

fn shell_escape(value: &str) -> String {
    let escaped = value.replace('\'', "'\"'\"'");
    format!("'{escaped}'")
}
