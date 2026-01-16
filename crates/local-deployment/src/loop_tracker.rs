//! Copilot Loop Tracker
//!
//! This module provides state tracking for Copilot's auto-loop functionality.
//! When loop is enabled, Copilot will automatically retry with follow-up requests
//! until either the completion promise is detected in output or max iterations is reached.

use std::{collections::HashMap, sync::Arc};

use executors::profile::ExecutorProfileId;
use tokio::sync::RwLock;
use uuid::Uuid;

/// State for tracking a single Copilot loop execution
#[derive(Debug, Clone)]
pub struct CopilotLoopState {
    /// Current iteration number (starts at 0)
    pub iteration: u32,
    /// Maximum number of iterations allowed
    pub max_iterations: u32,
    /// String that signals task completion (e.g., '<promise>COMPLETE</promise>')
    pub completion_promise: Option<String>,
    /// Original prompt to append "繼續" for follow-up
    pub original_prompt: String,
    /// Session ID from the Copilot executor
    pub session_id: String,
    /// Executor profile ID for creating follow-up requests
    pub executor_profile_id: ExecutorProfileId,
    /// Optional working directory
    pub working_dir: Option<String>,
}

impl CopilotLoopState {
    /// Check if we can continue with another iteration
    pub fn can_continue(&self) -> bool {
        self.iteration < self.max_iterations
    }

    /// Increment the iteration counter
    pub fn increment(&mut self) {
        self.iteration += 1;
    }

    /// Build the follow-up prompt by appending "繼續" to original prompt
    pub fn build_follow_up_prompt(&self) -> String {
        format!("{}\n\n繼續", self.original_prompt)
    }
}

/// Tracker for managing Copilot loop states across multiple workspaces
#[derive(Debug, Clone, Default)]
pub struct CopilotLoopTracker {
    /// Map of workspace ID to loop state
    states: Arc<RwLock<HashMap<Uuid, CopilotLoopState>>>,
}

impl CopilotLoopTracker {
    /// Create a new empty tracker
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new loop state for a workspace
    pub async fn register(
        &self,
        workspace_id: Uuid,
        max_iterations: u32,
        completion_promise: Option<String>,
        original_prompt: String,
        session_id: String,
        executor_profile_id: ExecutorProfileId,
        working_dir: Option<String>,
    ) {
        let state = CopilotLoopState {
            iteration: 0,
            max_iterations,
            completion_promise,
            original_prompt,
            session_id,
            executor_profile_id,
            working_dir,
        };
        self.states.write().await.insert(workspace_id, state);
        tracing::info!(
            "Registered Copilot loop state for workspace {}: max_iterations={}",
            workspace_id,
            max_iterations
        );
    }

    /// Get the current loop state for a workspace
    pub async fn get(&self, workspace_id: &Uuid) -> Option<CopilotLoopState> {
        self.states.read().await.get(workspace_id).cloned()
    }

    /// Update the session ID for a workspace (called after follow-up spawn)
    pub async fn update_session_id(&self, workspace_id: &Uuid, session_id: String) {
        if let Some(state) = self.states.write().await.get_mut(workspace_id) {
            state.session_id = session_id;
        }
    }

    /// Increment iteration and return whether we can continue
    pub async fn increment_and_check(&self, workspace_id: &Uuid) -> bool {
        if let Some(state) = self.states.write().await.get_mut(workspace_id) {
            state.increment();
            let can_continue = state.can_continue();
            tracing::info!(
                "Copilot loop iteration {} of {} for workspace {} (can_continue={})",
                state.iteration,
                state.max_iterations,
                workspace_id,
                can_continue
            );
            can_continue
        } else {
            false
        }
    }

    /// Remove loop state for a workspace (called when loop completes or fails)
    pub async fn remove(&self, workspace_id: &Uuid) {
        if self.states.write().await.remove(workspace_id).is_some() {
            tracing::info!("Removed Copilot loop state for workspace {}", workspace_id);
        }
    }

    /// Check if a workspace has an active loop
    pub async fn has_active_loop(&self, workspace_id: &Uuid) -> bool {
        self.states.read().await.contains_key(workspace_id)
    }

    /// Get the completion promise for a workspace
    pub async fn get_completion_promise(&self, workspace_id: &Uuid) -> Option<String> {
        self.states
            .read()
            .await
            .get(workspace_id)
            .and_then(|s| s.completion_promise.clone())
    }
}

/// Check if the output contains the completion promise string
pub fn check_completion_promise(output: &str, completion_promise: &str) -> bool {
    if completion_promise.is_empty() {
        return false;
    }
    let found = output.contains(completion_promise);
    if found {
        tracing::info!(
            "Completion promise '{}' detected in output",
            completion_promise
        );
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use executors::executors::BaseCodingAgent;

    #[tokio::test]
    async fn test_loop_tracker_lifecycle() {
        let tracker = CopilotLoopTracker::new();
        let workspace_id = Uuid::new_v4();
        let profile_id = ExecutorProfileId::new(BaseCodingAgent::Copilot);

        // Register a new loop
        tracker
            .register(
                workspace_id,
                5,
                Some("<promise>COMPLETE</promise>".to_string()),
                "Test prompt".to_string(),
                "session-123".to_string(),
                profile_id,
                None,
            )
            .await;

        // Check initial state
        assert!(tracker.has_active_loop(&workspace_id).await);
        let state = tracker.get(&workspace_id).await.unwrap();
        assert_eq!(state.iteration, 0);
        assert_eq!(state.max_iterations, 5);

        // Increment and check
        assert!(tracker.increment_and_check(&workspace_id).await);
        let state = tracker.get(&workspace_id).await.unwrap();
        assert_eq!(state.iteration, 1);

        // Remove
        tracker.remove(&workspace_id).await;
        assert!(!tracker.has_active_loop(&workspace_id).await);
    }

    #[test]
    fn test_check_completion_promise() {
        let output = "Task completed. <promise>COMPLETE-HUNTER</promise> Done.";
        assert!(check_completion_promise(
            output,
            "<promise>COMPLETE-HUNTER</promise>"
        ));
        assert!(!check_completion_promise(output, "<promise>OTHER</promise>"));
        assert!(!check_completion_promise(output, ""));
    }

    #[test]
    fn test_build_follow_up_prompt() {
        let state = CopilotLoopState {
            iteration: 1,
            max_iterations: 5,
            completion_promise: None,
            original_prompt: "Build the feature".to_string(),
            session_id: "test".to_string(),
            executor_profile_id: ExecutorProfileId::new(BaseCodingAgent::Copilot),
            working_dir: None,
        };
        assert_eq!(state.build_follow_up_prompt(), "Build the feature\n\n繼續");
    }
}
