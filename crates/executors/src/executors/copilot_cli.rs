use std::{
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use futures::StreamExt;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::{
    fs,
    io::AsyncWriteExt,
    process::Command,
    time::{interval, timeout},
};
use ts_rs::TS;
use uuid::Uuid;
use workspace_utils::{
    command_ext::GroupSpawnNoWindowExt, msg_store::MsgStore, path::get_vibe_kanban_temp_dir,
};

use crate::{
    command::{CmdOverrides, CommandBuildError, CommandBuilder, apply_overrides},
    env::ExecutionEnv,
    executor_discovery::ExecutorDiscoveredOptions,
    executors::{
        AppendPrompt, AvailabilityInfo, BaseCodingAgent, ExecutorError, SpawnedChild,
        StandardCodingAgentExecutor,
    },
    logs::{
        NormalizedEntry, NormalizedEntryType,
        plain_text_processor::PlainTextLogProcessor,
        stderr_processor::normalize_stderr_logs,
        utils::{EntryIndexProvider, patch},
    },
    model_selector::{ModelInfo, ModelSelectorConfig, PermissionPolicy},
    profile::ExecutorConfig,
};

/// Default base command for Copilot CLI in legacy (non-ACP) stdio mode.
///
/// Uses the current package version (`0.0.403`).  If the legacy flags
/// (`--no-color --log-level debug --log-dir`) turn out to be incompatible
/// with this version, the version will be pinned to a known-good legacy
/// build (e.g. `0.0.375`) during integration testing (task D.01.a.i).
const DEFAULT_BASE_COMMAND: &str = "npx -y @github/copilot@latest";

/// Prefix injected into stdout lines by [`Self::send_session_id`] so that
/// [`Self::normalize_logs`] can intercept and route the session id to
/// [`MsgStore::push_session_id`] instead of treating it as assistant output.
const SESSION_PREFIX: &str = "[copilot-session] ";

/// Copilot CLI executor — legacy direct-process mode (stdin prompt, `--resume`
/// session management, log-dir based session discovery).
///
/// This executor intentionally does **not** use the ACP shared harness.  It
/// spawns the Copilot CLI as a child process, writes the prompt to stdin, and
/// reads normalised output from stdout.  Session follow-up is handled via the
/// `--resume <session_id>` flag, and the session id is discovered by scanning
/// the temporary log directory for `<UUID>.log` files.
///
/// The struct mirrors the serialisable fields of the ACP-based [`super::copilot::Copilot`]
/// executor but omits the `approvals` field because the legacy stdio path has
/// no ACP `request_permission()` channel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS, JsonSchema)]
pub struct CopilotCli {
    #[serde(default)]
    pub append_prompt: AppendPrompt,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_all_tools: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_tool: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deny_tool: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub add_dir: Option<Vec<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_mcp_server: Option<Vec<String>>,

    #[serde(flatten)]
    pub cmd: CmdOverrides,
}

// ---------------------------------------------------------------------------
// Command builder helpers  (B.01.b.i)
// ---------------------------------------------------------------------------

impl CopilotCli {
    /// Build the legacy CLI command (**without** `--acp`).
    ///
    /// Includes the required legacy flags for session-id discovery:
    /// `--no-color --log-level debug --log-dir <dir>`.
    ///
    /// The `log_dir` is supplied by [`Self::create_temp_log_dir`] at spawn
    /// time.
    fn build_command_builder(&self, log_dir: &str) -> Result<CommandBuilder, CommandBuildError> {
        // Start with the base command and attach mandatory legacy flags.
        let mut builder = CommandBuilder::new(DEFAULT_BASE_COMMAND).params([
            "--no-color",
            "--log-level",
            "debug",
            "--log-dir",
            log_dir,
        ]);

        // --- optional config flags (same order as ACP Copilot) ---
        if self.allow_all_tools.unwrap_or(false) {
            builder = builder.extend_params(["--allow-all-tools"]);
        }

        if let Some(model) = &self.model {
            builder = builder.extend_params(["--model", model]);
        }

        if let Some(tool) = &self.allow_tool {
            builder = builder.extend_params(["--allow-tool", tool]);
        }

        if let Some(tool) = &self.deny_tool {
            builder = builder.extend_params(["--deny-tool", tool]);
        }

        if let Some(dirs) = &self.add_dir {
            for dir in dirs {
                builder = builder.extend_params(["--add-dir", dir]);
            }
        }

        if let Some(servers) = &self.disable_mcp_server {
            for server in servers {
                builder = builder.extend_params(["--disable-mcp-server", server]);
            }
        }

        // NOTE: intentionally omitting `--acp` — this executor uses legacy
        // stdin/stdout mode.

        apply_overrides(builder, &self.cmd)
    }
}

// ---------------------------------------------------------------------------
// Private helpers  (B.02.a.i + B.03.a.i)
// ---------------------------------------------------------------------------

impl CopilotCli {
    /// Create a fresh, per-run temporary log directory for the Copilot CLI.
    ///
    /// Layout:
    /// ```text
    /// <vibe-kanban-temp>/copilot_cli_logs/<worktree-basename>/<uuid>/
    /// ```
    ///
    /// The directory is created eagerly so that the CLI can write its log files
    /// immediately after startup.  The path is later passed via `--log-dir`.
    pub(crate) async fn create_temp_log_dir(current_dir: &Path) -> Result<PathBuf, ExecutorError> {
        let base_log_dir = get_vibe_kanban_temp_dir().join("copilot_cli_logs");
        fs::create_dir_all(&base_log_dir)
            .await
            .map_err(ExecutorError::Io)?;

        let run_log_dir = base_log_dir
            .join(current_dir.file_name().unwrap_or_default())
            .join(Uuid::new_v4().to_string());
        fs::create_dir_all(&run_log_dir)
            .await
            .map_err(ExecutorError::Io)?;

        Ok(run_log_dir)
    }

    /// Build a [`PlainTextLogProcessor`] that strips ANSI escapes and emits
    /// `AssistantMessage` entries — the same normaliser the legacy Copilot
    /// executor used for plain-text stdout.
    fn create_simple_stdout_normalizer(
        index_provider: EntryIndexProvider,
    ) -> PlainTextLogProcessor {
        PlainTextLogProcessor::builder()
            .normalized_entry_producer(Box::new(|content: String| NormalizedEntry {
                timestamp: None,
                entry_type: NormalizedEntryType::AssistantMessage,
                content,
                metadata: None,
            }))
            .transform_lines(Box::new(|lines| {
                for line in lines.iter_mut() {
                    let stripped = strip_ansi_escapes::strip_str(line.as_str());
                    *line = stripped;
                }
            }))
            .index_provider(index_provider)
            .build()
    }

    /// Scan the log directory for a `.log` file containing a session UUID.
    ///
    /// The Copilot CLI writes debug log files that include a line matching
    /// `events to session <UUID>`.  This function polls the log directory
    /// every 200 ms (up to 10 minutes) until a valid UUID is found.
    async fn watch_session_id(log_dir_path: PathBuf) -> Result<String, String> {
        let session_regex =
            Regex::new(r"events to session ([0-9a-fA-F-]{36})").map_err(|e| e.to_string())?;

        let log_dir_clone = log_dir_path.clone();
        timeout(Duration::from_secs(600), async move {
            let mut ticker = interval(Duration::from_millis(200));
            loop {
                if let Ok(mut rd) = fs::read_dir(&log_dir_clone).await {
                    while let Ok(Some(entry)) = rd.next_entry().await {
                        let path = entry.path();
                        if path.extension().map(|e| e == "log").unwrap_or(false)
                            && let Ok(content) = fs::read_to_string(&path).await
                            && let Some(caps) = session_regex.captures(&content)
                            && let Some(matched) = caps.get(1)
                        {
                            let uuid_str = matched.as_str();
                            if Uuid::parse_str(uuid_str).is_ok() {
                                return Ok(uuid_str.to_string());
                            }
                        }
                    }
                }
                ticker.tick().await;
            }
        })
        .await
        .map_err(|_| format!("No session ID found in log files at {log_dir_path:?}"))?
    }

    /// Spawn a background task that watches for a session ID in the log
    /// directory and injects a `[copilot-session] <uuid>\n` line into the
    /// provided writer (typically the stdout pipe that [`MsgStore`] reads).
    ///
    /// The injected line is later intercepted by [`Self::normalize_logs`]
    /// and routed to [`MsgStore::push_session_id`].
    fn send_session_id(
        log_dir_path: PathBuf,
        mut writer: impl tokio::io::AsyncWrite + Send + Unpin + 'static,
    ) {
        tokio::spawn(async move {
            match Self::watch_session_id(log_dir_path).await {
                Ok(session_id) => {
                    let session_line = format!("{SESSION_PREFIX}{session_id}\n");
                    if let Err(e) = writer.write_all(session_line.as_bytes()).await {
                        tracing::error!("Failed to write session ID to stdout pipe: {e}");
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to find Copilot CLI session ID: {e}");
                }
            }
        });
    }
}

// ---------------------------------------------------------------------------
// StandardCodingAgentExecutor implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl StandardCodingAgentExecutor for CopilotCli {
    // ----- B.01.c.i: apply_overrides -----------------------------------------
    fn apply_overrides(&mut self, executor_config: &ExecutorConfig) {
        if let Some(model_id) = &executor_config.model_id {
            self.model = Some(model_id.clone());
        }

        if let Some(permission_policy) = &executor_config.permission_policy {
            self.allow_all_tools = Some(matches!(
                permission_policy,
                crate::model_selector::PermissionPolicy::Auto
            ));
        }

        // agent_id, reasoning_id, temperature, max_tokens — not supported by
        // the Copilot CLI; intentionally ignored.
    }

    // ----- B.02.c.i: spawn — direct process, stdin prompt, stdout tee + session id injection -----
    async fn spawn(
        &self,
        current_dir: &Path,
        prompt: &str,
        env: &ExecutionEnv,
    ) -> Result<SpawnedChild, ExecutorError> {
        // 1. Per-run log directory (for session-id discovery)
        let log_dir = Self::create_temp_log_dir(current_dir).await?;
        let log_dir_str = log_dir.to_string_lossy().to_string();

        // 2. Build the legacy CLI command (no --acp)
        let command_parts = self.build_command_builder(&log_dir_str)?.build_initial()?;
        let (executable_path, args) = command_parts.into_resolved().await?;

        let combined_prompt = self.append_prompt.combine_prompt(prompt);

        // 3. Prepare the child process with piped stdio
        let mut command = Command::new(executable_path);
        command
            .kill_on_drop(true)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(current_dir)
            .env("NPM_CONFIG_LOGLEVEL", "error")
            .args(&args);

        env.clone()
            .with_profile(&self.cmd)
            .apply_to_command(&mut command);

        let mut child = command.group_spawn_no_window()?;

        // 4. Write prompt to stdin, then close so the CLI sees EOF
        if let Some(mut stdin) = child.inner().stdin.take() {
            stdin.write_all(combined_prompt.as_bytes()).await?;
            stdin.shutdown().await?;
        }

        // 5. Tee real stdout through a merged pipe; get an injection writer
        //    that send_session_id can use to push the session marker into
        //    the same stream MsgStore reads.
        let inject_writer = crate::stdout_dup::tee_stdout_with_line_injector(&mut child)?;
        Self::send_session_id(log_dir, inject_writer);

        Ok(child.into())
    }

    // ----- B.02.d.i: spawn_follow_up — same as spawn but with --resume <session_id> -----
    async fn spawn_follow_up(
        &self,
        current_dir: &Path,
        prompt: &str,
        session_id: &str,
        _reset_to_message_id: Option<&str>,
        env: &ExecutionEnv,
    ) -> Result<SpawnedChild, ExecutorError> {
        // 1. Per-run log directory
        let log_dir = Self::create_temp_log_dir(current_dir).await?;
        let log_dir_str = log_dir.to_string_lossy().to_string();

        // 2. Build command with --resume <session_id> appended
        let command_parts = self
            .build_command_builder(&log_dir_str)?
            .build_follow_up(&["--resume".to_string(), session_id.to_string()])?;
        let (executable_path, args) = command_parts.into_resolved().await?;

        let combined_prompt = self.append_prompt.combine_prompt(prompt);

        // 3. Prepare child process
        let mut command = Command::new(executable_path);
        command
            .kill_on_drop(true)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(current_dir)
            .env("NPM_CONFIG_LOGLEVEL", "error")
            .args(&args);

        env.clone()
            .with_profile(&self.cmd)
            .apply_to_command(&mut command);

        let mut child = command.group_spawn_no_window()?;

        // 4. Write prompt to stdin, then close
        if let Some(mut stdin) = child.inner().stdin.take() {
            stdin.write_all(combined_prompt.as_bytes()).await?;
            stdin.shutdown().await?;
        }

        // 5. Tee stdout + background session-id injection
        let inject_writer = crate::stdout_dup::tee_stdout_with_line_injector(&mut child)?;
        Self::send_session_id(log_dir, inject_writer);

        Ok(child.into())
    }

    // ----- B.03.a.i: normalize_logs ------------------------------------------
    fn normalize_logs(
        &self,
        msg_store: Arc<MsgStore>,
        _worktree_path: &Path,
    ) -> Vec<tokio::task::JoinHandle<()>> {
        let entry_index_provider = EntryIndexProvider::start_from(&msg_store);

        // 1. stderr — reuse the standard stderr normaliser (2 s time-gap,
        //    ErrorMessage entries).
        let h_stderr = normalize_stderr_logs(msg_store.clone(), entry_index_provider.clone());

        // 2. stdout — plain text normaliser that strips ANSI escapes and emits
        //    AssistantMessage entries.  Lines prefixed with `SESSION_PREFIX` are
        //    intercepted and forwarded to `push_session_id()`.
        let msg_store_stdout = msg_store.clone();
        let h_stdout = tokio::spawn(async move {
            let mut stdout_lines = msg_store_stdout.stdout_lines_stream();

            let mut processor = CopilotCli::create_simple_stdout_normalizer(entry_index_provider);

            while let Some(Ok(line)) = stdout_lines.next().await {
                // Intercept session-id marker injected by send_session_id().
                if let Some(session_id) = line.strip_prefix(SESSION_PREFIX) {
                    msg_store_stdout.push_session_id(session_id.trim().to_string());
                    continue;
                }

                for patch in processor.process(line + "\n") {
                    msg_store_stdout.push_patch(patch);
                }
            }
        });

        vec![h_stderr, h_stdout]
    }

    // ----- B.01.d.i: default_mcp_config_path ---------------------------------
    fn default_mcp_config_path(&self) -> Option<std::path::PathBuf> {
        dirs::home_dir().map(|home| home.join(".copilot").join("mcp-config.json"))
    }

    // ----- B.01.d.i: get_availability_info -----------------------------------
    fn get_availability_info(&self) -> AvailabilityInfo {
        let mcp_config_found = self
            .default_mcp_config_path()
            .map(|p| p.exists())
            .unwrap_or(false);

        let installation_indicator_found = dirs::home_dir()
            .map(|home| home.join(".copilot").join("config.json").exists())
            .unwrap_or(false);

        if mcp_config_found || installation_indicator_found {
            AvailabilityInfo::InstallationFound
        } else {
            AvailabilityInfo::NotFound
        }
    }

    // ----- B.01.c.i: get_preset_options --------------------------------------
    fn get_preset_options(&self) -> ExecutorConfig {
        ExecutorConfig {
            executor: BaseCodingAgent::CopilotCli,
            variant: None,
            model_id: self.model.clone(),
            agent_id: None,
            reasoning_id: None,
            permission_policy: Some(PermissionPolicy::Auto),
        }
    }

    // ----- B.01.d.i: discover_options ----------------------------------------
    async fn discover_options(
        &self,
        _workdir: Option<&Path>,
        _repo_path: Option<&Path>,
    ) -> Result<futures::stream::BoxStream<'static, json_patch::Patch>, ExecutorError> {
        let options = ExecutorDiscoveredOptions {
            model_selector: ModelSelectorConfig {
                models: [
                    ("gpt-5.4", "GPT-5.4"),
                    ("claude-opus-4.6", "Claude Opus 4.6"),
                    ("claude-opus-4.6-fast", "Claude Opus 4.6 Fast"),
                    ("gpt-5.3-codex", "GPT-5.3 Codex"),
                    ("claude-sonnet-4.6", "Claude Sonnet 4.6"),
                    ("claude-haiku-4.5", "Claude Haiku 4.5"),
                    ("gemini-3-pro-preview", "Gemini 3 Pro Preview"),
                    ("gpt-5.2-codex", "GPT-5.2 Codex"),
                    ("gpt-5.2", "GPT-5.2"),
                    ("gpt-5.1-codex-max", "GPT-5.1 Codex Max"),
                    ("gpt-5.1-codex", "GPT-5.1 Codex"),
                    ("gpt-5.1", "GPT-5.1"),
                    ("gpt-5.1-codex-mini", "GPT-5.1 Codex Mini"),
                    ("gpt-5-mini", "GPT-5 Mini"),
                    ("gpt-4.1", "GPT-4.1"),
                    ("claude-opus-4.5", "Claude Opus 4.5"),
                    ("claude-sonnet-4.5", "Claude Sonnet 4.5"),
                    ("claude-sonnet-4", "Claude Sonnet 4"),
                ]
                .into_iter()
                .map(|(id, name)| ModelInfo {
                    id: id.to_string(),
                    name: name.to_string(),
                    provider_id: None,
                    reasoning_options: vec![],
                })
                .collect(),
                permissions: vec![PermissionPolicy::Auto, PermissionPolicy::Supervised],
                ..Default::default()
            },
            ..Default::default()
        };
        Ok(Box::pin(futures::stream::once(async move {
            patch::executor_discovered_options(options)
        })))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use workspace_utils::log_msg::LogMsg;

    use super::*;
    use crate::{
        command::CmdOverrides, logs::utils::patch::extract_normalized_entry_from_patch,
        model_selector::PermissionPolicy,
    };

    /// Create a default `CopilotCli` instance with all optional fields `None`.
    fn default_cli() -> CopilotCli {
        CopilotCli {
            append_prompt: AppendPrompt::default(),
            model: None,
            allow_all_tools: None,
            allow_tool: None,
            deny_tool: None,
            add_dir: None,
            disable_mcp_server: None,
            cmd: CmdOverrides::default(),
        }
    }

    // -----------------------------------------------------------------------
    // D.01.a.i — build_command_builder tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_build_command_builder_default_flags() {
        let cli = default_cli();
        let builder = cli.build_command_builder("/tmp/logs").unwrap();

        assert_eq!(builder.base, DEFAULT_BASE_COMMAND);
        let params = builder.params.as_ref().expect("params should exist");
        assert!(params.contains(&"--no-color".to_string()));
        assert!(params.contains(&"--log-level".to_string()));
        assert!(params.contains(&"debug".to_string()));
        assert!(params.contains(&"--log-dir".to_string()));
        assert!(params.contains(&"/tmp/logs".to_string()));
        // Must NOT contain --acp
        assert!(!params.contains(&"--acp".to_string()));
    }

    #[test]
    fn test_build_command_builder_with_model() {
        let cli = CopilotCli {
            model: Some("gpt-5.4".to_string()),
            ..default_cli()
        };
        let builder = cli.build_command_builder("/tmp/logs").unwrap();
        let params = builder.params.unwrap();
        let model_idx = params.iter().position(|p| p == "--model").unwrap();
        assert_eq!(params[model_idx + 1], "gpt-5.4");
    }

    #[test]
    fn test_build_command_builder_with_allow_all_tools() {
        let cli = CopilotCli {
            allow_all_tools: Some(true),
            ..default_cli()
        };
        let builder = cli.build_command_builder("/tmp/logs").unwrap();
        let params = builder.params.unwrap();
        assert!(params.contains(&"--allow-all-tools".to_string()));
    }

    #[test]
    fn test_build_command_builder_allow_all_tools_false_omits_flag() {
        let cli = CopilotCli {
            allow_all_tools: Some(false),
            ..default_cli()
        };
        let builder = cli.build_command_builder("/tmp/logs").unwrap();
        let params = builder.params.unwrap();
        assert!(!params.contains(&"--allow-all-tools".to_string()));
    }

    #[test]
    fn test_build_command_builder_with_allow_deny_tool() {
        let cli = CopilotCli {
            allow_tool: Some("file_read".to_string()),
            deny_tool: Some("shell".to_string()),
            ..default_cli()
        };
        let builder = cli.build_command_builder("/tmp/logs").unwrap();
        let params = builder.params.unwrap();
        let allow_idx = params.iter().position(|p| p == "--allow-tool").unwrap();
        assert_eq!(params[allow_idx + 1], "file_read");
        let deny_idx = params.iter().position(|p| p == "--deny-tool").unwrap();
        assert_eq!(params[deny_idx + 1], "shell");
    }

    #[test]
    fn test_build_command_builder_with_add_dirs() {
        let cli = CopilotCli {
            add_dir: Some(vec!["/src".to_string(), "/lib".to_string()]),
            ..default_cli()
        };
        let builder = cli.build_command_builder("/tmp/logs").unwrap();
        let params = builder.params.unwrap();
        let dir_positions: Vec<usize> = params
            .iter()
            .enumerate()
            .filter(|(_, p)| p.as_str() == "--add-dir")
            .map(|(i, _)| i)
            .collect();
        assert_eq!(dir_positions.len(), 2);
        assert_eq!(params[dir_positions[0] + 1], "/src");
        assert_eq!(params[dir_positions[1] + 1], "/lib");
    }

    #[test]
    fn test_build_command_builder_with_disable_mcp_server() {
        let cli = CopilotCli {
            disable_mcp_server: Some(vec!["server-a".to_string()]),
            ..default_cli()
        };
        let builder = cli.build_command_builder("/tmp/logs").unwrap();
        let params = builder.params.unwrap();
        let idx = params
            .iter()
            .position(|p| p == "--disable-mcp-server")
            .unwrap();
        assert_eq!(params[idx + 1], "server-a");
    }

    #[test]
    fn test_build_command_builder_follow_up_adds_resume() {
        let cli = default_cli();
        let builder = cli.build_command_builder("/tmp/logs").unwrap();
        let parts = builder
            .build_follow_up(&["--resume".to_string(), "sess-123".to_string()])
            .unwrap();
        // CommandParts is opaque, but build_follow_up must succeed
        let dbg = format!("{parts:?}");
        assert!(dbg.contains("--resume"));
        assert!(dbg.contains("sess-123"));
    }

    #[test]
    fn test_build_command_builder_full_config() {
        let cli = CopilotCli {
            model: Some("claude-sonnet-4.6".to_string()),
            allow_all_tools: Some(true),
            allow_tool: Some("edit".to_string()),
            deny_tool: Some("exec".to_string()),
            add_dir: Some(vec!["/extra".to_string()]),
            disable_mcp_server: Some(vec!["srv".to_string()]),
            ..default_cli()
        };
        let builder = cli.build_command_builder("/my/logs").unwrap();
        let params = builder.params.unwrap();

        // All flags present
        assert!(params.contains(&"--no-color".to_string()));
        assert!(params.contains(&"--allow-all-tools".to_string()));
        assert!(params.contains(&"--model".to_string()));
        assert!(params.contains(&"--allow-tool".to_string()));
        assert!(params.contains(&"--deny-tool".to_string()));
        assert!(params.contains(&"--add-dir".to_string()));
        assert!(params.contains(&"--disable-mcp-server".to_string()));
        // No ACP
        assert!(!params.contains(&"--acp".to_string()));
    }

    // -----------------------------------------------------------------------
    // D.01.b.i — follow-up command tests: --resume <session_id> correctly
    //            appended by the spawn_follow_up() command-build path
    // -----------------------------------------------------------------------

    #[test]
    fn test_follow_up_resume_with_uuid_session_id() {
        let cli = default_cli();
        let builder = cli.build_command_builder("/tmp/logs").unwrap();
        let uuid = "a1b2c3d4-e5f6-7890-abcd-ef1234567890";
        let parts = builder
            .build_follow_up(&["--resume".to_string(), uuid.to_string()])
            .unwrap();
        let dbg = format!("{parts:?}");
        assert!(dbg.contains("--resume"), "follow-up must include --resume");
        assert!(
            dbg.contains(uuid),
            "follow-up must include the session UUID"
        );
        // --resume must appear after base params (appended, not prepended)
        let resume_pos = dbg.find("--resume").unwrap();
        let log_dir_pos = dbg.find("--log-dir").unwrap();
        assert!(
            resume_pos > log_dir_pos,
            "--resume should appear after base params (--log-dir)"
        );
    }

    #[test]
    fn test_follow_up_resume_session_id_immediately_follows_flag() {
        let cli = default_cli();
        let builder = cli.build_command_builder("/tmp/logs").unwrap();
        let session = "test-session-42";
        let parts = builder
            .build_follow_up(&["--resume".to_string(), session.to_string()])
            .unwrap();
        let dbg = format!("{parts:?}");
        // Verify --resume is immediately followed by session_id in args vec
        let pattern = format!("\"--resume\", \"{session}\"");
        assert!(
            dbg.contains(&pattern),
            "--resume must be immediately followed by session_id; got: {dbg}"
        );
    }

    #[test]
    fn test_follow_up_preserves_all_base_params_with_resume() {
        let cli = CopilotCli {
            model: Some("gpt-5.4".to_string()),
            allow_all_tools: Some(true),
            ..default_cli()
        };
        let builder = cli.build_command_builder("/tmp/logs").unwrap();
        let parts = builder
            .build_follow_up(&["--resume".to_string(), "sess-abc".to_string()])
            .unwrap();
        let dbg = format!("{parts:?}");
        // Base params still present
        assert!(dbg.contains("--no-color"));
        assert!(dbg.contains("--log-level"));
        assert!(dbg.contains("--model"));
        assert!(dbg.contains("gpt-5.4"));
        assert!(dbg.contains("--allow-all-tools"));
        // --resume appended at the end
        assert!(dbg.contains("--resume"));
        assert!(dbg.contains("sess-abc"));
    }

    #[test]
    fn test_follow_up_empty_args_has_no_resume() {
        let cli = default_cli();
        let builder = cli.build_command_builder("/tmp/logs").unwrap();
        // build_follow_up with empty args should NOT add --resume
        let parts = builder.build_follow_up(&[]).unwrap();
        let dbg = format!("{parts:?}");
        assert!(
            !dbg.contains("--resume"),
            "empty follow-up args should not contain --resume"
        );
    }

    // -----------------------------------------------------------------------
    // D.02.a.i — apply_overrides / get_preset_options / discover_options tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_apply_overrides_model_id() {
        let mut cli = default_cli();
        let mut config = ExecutorConfig::new(BaseCodingAgent::CopilotCli);
        config.model_id = Some("gpt-5.1".to_string());
        cli.apply_overrides(&config);
        assert_eq!(cli.model, Some("gpt-5.1".to_string()));
    }

    #[test]
    fn test_apply_overrides_permission_auto() {
        let mut cli = default_cli();
        let mut config = ExecutorConfig::new(BaseCodingAgent::CopilotCli);
        config.permission_policy = Some(PermissionPolicy::Auto);
        cli.apply_overrides(&config);
        assert_eq!(cli.allow_all_tools, Some(true));
    }

    #[test]
    fn test_apply_overrides_permission_supervised() {
        let mut cli = default_cli();
        let mut config = ExecutorConfig::new(BaseCodingAgent::CopilotCli);
        config.permission_policy = Some(PermissionPolicy::Supervised);
        cli.apply_overrides(&config);
        assert_eq!(cli.allow_all_tools, Some(false));
    }

    #[test]
    fn test_apply_overrides_ignores_unsupported_fields() {
        let mut cli = CopilotCli {
            model: Some("original".to_string()),
            ..default_cli()
        };
        let mut config = ExecutorConfig::new(BaseCodingAgent::CopilotCli);
        config.agent_id = Some("some-agent".to_string());
        config.reasoning_id = Some("some-reasoning".to_string());
        cli.apply_overrides(&config);
        // Model unchanged (no model_id in config)
        assert_eq!(cli.model, Some("original".to_string()));
    }

    #[test]
    fn test_get_preset_options_defaults() {
        let cli = default_cli();
        let preset = cli.get_preset_options();
        assert_eq!(preset.executor, BaseCodingAgent::CopilotCli);
        assert_eq!(preset.model_id, None);
        assert_eq!(preset.agent_id, None);
        assert_eq!(preset.reasoning_id, None);
        assert_eq!(preset.permission_policy, Some(PermissionPolicy::Auto));
    }

    #[test]
    fn test_get_preset_options_with_model() {
        let cli = CopilotCli {
            model: Some("gpt-5.4".to_string()),
            ..default_cli()
        };
        let preset = cli.get_preset_options();
        assert_eq!(preset.model_id, Some("gpt-5.4".to_string()));
    }

    #[tokio::test]
    async fn test_discover_options_models_and_permissions() {
        let cli = default_cli();
        let mut stream = cli.discover_options(None, None).await.unwrap();
        let patch = stream.next().await.expect("should have one patch");

        // Verify the patch is non-empty (contains model list + permissions)
        assert!(
            !patch.0.is_empty(),
            "discovered options patch should not be empty"
        );
    }

    #[tokio::test]
    async fn test_discover_options_no_acp_flag() {
        // discover_options should succeed without any workdir
        let cli = default_cli();
        let result = cli.discover_options(None, None).await;
        assert!(result.is_ok());
    }

    // -----------------------------------------------------------------------
    // D.02.b.i — session ID extraction tests (watch_session_id)
    // -----------------------------------------------------------------------

    /// Create a unique temp directory for a test.
    async fn make_test_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("copilot_cli_tests")
            .join(format!("{label}-{}", Uuid::new_v4()));
        tokio::fs::create_dir_all(&dir).await.unwrap();
        dir
    }

    #[tokio::test]
    async fn test_watch_session_id_finds_uuid_in_log() {
        let dir = make_test_dir("find-uuid").await;
        let log_path = dir.join("debug.log");

        let uuid = "a1b2c3d4-e5f6-7890-abcd-ef1234567890";
        let content = format!("some debug output\nSaving events to session {uuid}\nmore output\n");
        tokio::fs::write(&log_path, content.as_bytes())
            .await
            .unwrap();

        let result = CopilotCli::watch_session_id(dir.clone()).await;
        assert_eq!(result.unwrap(), uuid);

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn test_watch_session_id_ignores_non_log_files() {
        let dir = make_test_dir("ignore-txt").await;

        // Write a .txt file (not .log) with a valid session line
        let txt_path = dir.join("debug.txt");
        let uuid = "11111111-2222-3333-4444-555555555555";
        let content = format!("Saving events to session {uuid}\n");
        tokio::fs::write(&txt_path, content.as_bytes())
            .await
            .unwrap();

        // watch_session_id should time out since there's no .log file
        let result = tokio::time::timeout(
            Duration::from_millis(500),
            CopilotCli::watch_session_id(dir.clone()),
        )
        .await;
        assert!(result.is_err(), "should time out without .log file");

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn test_watch_session_id_rejects_invalid_uuid() {
        let dir = make_test_dir("invalid-uuid").await;
        let log_path = dir.join("run.log");

        // Write a non-UUID match
        tokio::fs::write(
            &log_path,
            b"Saving events to session not-a-valid-uuid-here!\n",
        )
        .await
        .unwrap();

        let result = tokio::time::timeout(
            Duration::from_millis(500),
            CopilotCli::watch_session_id(dir.clone()),
        )
        .await;
        assert!(result.is_err(), "should time out with invalid UUID");

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn test_watch_session_id_picks_up_late_file() {
        let dir = make_test_dir("late-file").await;
        let dir_for_write = dir.clone();

        let uuid = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
        let uuid_clone = uuid.to_string();

        // Spawn watcher first, then write the log file after a short delay
        let watcher = tokio::spawn(async move { CopilotCli::watch_session_id(dir).await });

        tokio::time::sleep(Duration::from_millis(300)).await;
        let log_path = dir_for_write.join("late.log");
        let content = format!("events to session {uuid_clone}\n");
        tokio::fs::write(&log_path, content.as_bytes())
            .await
            .unwrap();

        let result = tokio::time::timeout(Duration::from_secs(5), watcher)
            .await
            .expect("watcher should complete")
            .expect("task should not panic");
        assert_eq!(result.unwrap(), uuid);

        let _ = tokio::fs::remove_dir_all(&dir_for_write).await;
    }

    // -----------------------------------------------------------------------
    // D.03.a.i — normalize_logs tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_normalize_logs_session_prefix_intercepted() {
        let msg_store = Arc::new(MsgStore::new());
        let uuid = "12345678-1234-1234-1234-123456789abc";

        // Push stdout data BEFORE calling normalize_logs so it's in history.
        msg_store.push_stdout(format!("{SESSION_PREFIX}{uuid}\n"));
        msg_store.push_stdout("Hello from copilot\n");
        msg_store.push(LogMsg::Finished);

        let cli = default_cli();
        let handles = cli.normalize_logs(msg_store.clone(), Path::new("/tmp"));

        // Wait for all normalizer tasks to complete.
        for h in handles {
            let _ = tokio::time::timeout(Duration::from_secs(5), h).await;
        }

        let history = msg_store.get_history();

        // Should find a SessionId message
        let session_ids: Vec<&str> = history
            .iter()
            .filter_map(|m| match m {
                LogMsg::SessionId(s) => Some(s.as_str()),
                _ => None,
            })
            .collect();
        assert!(
            session_ids.contains(&uuid),
            "session ID should be intercepted; got {session_ids:?}"
        );

        // Should find a JsonPatch for the "Hello from copilot" line
        let patches: Vec<_> = history
            .iter()
            .filter_map(|m| match m {
                LogMsg::JsonPatch(p) => Some(p),
                _ => None,
            })
            .collect();
        assert!(
            !patches.is_empty(),
            "should have at least one json patch for stdout"
        );

        // Extract entries from patches to verify content
        let entries: Vec<_> = patches
            .iter()
            .filter_map(|p| extract_normalized_entry_from_patch(p))
            .collect();
        let has_hello = entries
            .iter()
            .any(|(_, e)| e.content.contains("Hello from copilot"));
        assert!(
            has_hello,
            "should have assistant message with 'Hello from copilot'"
        );
    }

    #[tokio::test]
    async fn test_normalize_logs_plain_text_becomes_assistant_message() {
        let msg_store = Arc::new(MsgStore::new());

        msg_store.push_stdout("Line one\n");
        msg_store.push_stdout("Line two\n");
        msg_store.push(LogMsg::Finished);

        let cli = default_cli();
        let handles = cli.normalize_logs(msg_store.clone(), Path::new("/w"));

        for h in handles {
            let _ = tokio::time::timeout(Duration::from_secs(5), h).await;
        }

        let history = msg_store.get_history();
        let patches: Vec<_> = history
            .iter()
            .filter_map(|m| match m {
                LogMsg::JsonPatch(p) => Some(p),
                _ => None,
            })
            .collect();

        let entries: Vec<_> = patches
            .iter()
            .filter_map(|p| extract_normalized_entry_from_patch(p))
            .collect();

        assert!(!entries.is_empty(), "should produce at least one entry");
        // All entries should be AssistantMessage
        for (_, entry) in &entries {
            assert!(
                matches!(entry.entry_type, NormalizedEntryType::AssistantMessage),
                "expected AssistantMessage, got {:?}",
                entry.entry_type
            );
        }
    }

    #[tokio::test]
    async fn test_normalize_logs_strips_ansi() {
        let msg_store = Arc::new(MsgStore::new());

        // Push a line with ANSI escape codes
        msg_store.push_stdout("\x1b[32mgreen text\x1b[0m\n");
        msg_store.push(LogMsg::Finished);

        let cli = default_cli();
        let handles = cli.normalize_logs(msg_store.clone(), Path::new("/w"));

        for h in handles {
            let _ = tokio::time::timeout(Duration::from_secs(5), h).await;
        }

        let history = msg_store.get_history();
        let entries: Vec<_> = history
            .iter()
            .filter_map(|m| match m {
                LogMsg::JsonPatch(p) => extract_normalized_entry_from_patch(p),
                _ => None,
            })
            .collect();

        let has_clean = entries
            .iter()
            .any(|(_, e)| e.content.contains("green text") && !e.content.contains("\x1b"));
        assert!(has_clean, "ANSI escapes should be stripped from output");
    }

    #[tokio::test]
    async fn test_normalize_logs_no_session_when_absent() {
        let msg_store = Arc::new(MsgStore::new());

        msg_store.push_stdout("regular line\n");
        msg_store.push(LogMsg::Finished);

        let cli = default_cli();
        let handles = cli.normalize_logs(msg_store.clone(), Path::new("/w"));

        for h in handles {
            let _ = tokio::time::timeout(Duration::from_secs(5), h).await;
        }

        let history = msg_store.get_history();
        let session_ids: Vec<_> = history
            .iter()
            .filter(|m| matches!(m, LogMsg::SessionId(_)))
            .collect();
        assert!(session_ids.is_empty(), "no session ID should be emitted");
    }
}
