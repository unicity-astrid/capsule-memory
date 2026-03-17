#![deny(unsafe_code)]
#![deny(clippy::all)]
#![deny(unreachable_pub)]
#![warn(missing_docs)]

//! Cross-session memory capsule for Astrid OS.
//!
//! Stores agent memory in the capsule's KV store and injects it into the
//! system prompt via `prompt_builder.v1.hook.before_build`. Exposes an
//! `add_memory` tool so the agent can persist notes across sessions.

use astrid_sdk::prelude::*;
use astrid_sdk::schemars::{self, JsonSchema};
use serde::Deserialize;

/// KV key for the memory content.
const MEMORY_KEY: &str = "memory";

/// Maximum size in bytes for memory content.
///
/// Prevents unbounded context window consumption from agent-written
/// content. Memory is truncated at the nearest char boundary if exceeded.
const MAX_MEMORY_BYTES: usize = 32_768;

/// Input for the `add_memory` tool.
#[derive(Deserialize, JsonSchema)]
pub struct AddMemoryInput {
    /// The full memory content to store.
    content: String,
}

/// Cross-session memory capsule.
#[derive(Default)]
pub struct MemoryCapsule;

#[capsule]
impl MemoryCapsule {
    /// Intercepts `prompt_builder.v1.hook.before_build` events.
    ///
    /// Reads memory from KV and publishes a hook response with
    /// `appendSystemContext`. If memory is empty, this is a no-op.
    #[astrid::interceptor("on_before_prompt_build")]
    pub fn on_before_prompt_build(&self, payload: serde_json::Value) -> Result<(), SysError> {
        let response_topic = payload
            .get("response_topic")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                SysError::ApiError("missing response_topic in before_build payload".into())
            })?;

        let content = match kv::get_bytes(MEMORY_KEY) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            Err(_) => return Ok(()),
        };

        if content.trim().is_empty() {
            return Ok(());
        }

        let section = if content.len() > MAX_MEMORY_BYTES {
            let end = content.floor_char_boundary(MAX_MEMORY_BYTES);
            format!("# Memory\n\n{}\n\n[Memory truncated]", &content[..end])
        } else {
            format!("# Memory\n\n{content}")
        };

        ipc::publish_json(
            response_topic,
            &serde_json::json!({ "appendSystemContext": section }),
        )?;

        Ok(())
    }

    /// Handles `/memory-export` command from the CLI.
    ///
    /// Reads memory from KV and writes it to `.astrid/memory.md` in the
    /// workspace, then responds to the user.
    #[astrid::interceptor("handle_command")]
    pub fn handle_command(&self, payload: serde_json::Value) -> Result<(), SysError> {
        let text = payload.get("text").and_then(|v| v.as_str()).unwrap_or("");
        let session_id = payload
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        if text.trim() != "memory-export" {
            return Ok(());
        }

        let content = match kv::get_bytes(MEMORY_KEY) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            Err(_) => String::new(),
        };

        if content.trim().is_empty() {
            ipc::publish_json(
                "agent.v1.response",
                &serde_json::json!({
                    "type": "agent_response",
                    "text": "No memory stored yet.",
                    "is_final": true,
                    "session_id": session_id,
                }),
            )?;
            return Ok(());
        }

        fs::write(".astrid/memory.md", &content)?;

        ipc::publish_json(
            "agent.v1.response",
            &serde_json::json!({
                "type": "agent_response",
                "text": format!("Memory exported to .astrid/memory.md ({} bytes)", content.len()),
                "is_final": true,
                "session_id": session_id,
            }),
        )?;

        Ok(())
    }

    /// Persist cross-session memory. Use this to store notes, user
    /// preferences, project context, or anything that should survive
    /// across sessions. Overwrites the entire memory content — check
    /// the existing memory in the system prompt context and include
    /// it if you need to append rather than replace.
    #[astrid::tool]
    pub fn add_memory(&self, input: AddMemoryInput) -> Result<serde_json::Value, SysError> {
        if input.content.len() > MAX_MEMORY_BYTES {
            return Err(SysError::ApiError(format!(
                "memory content exceeds maximum size of {MAX_MEMORY_BYTES} bytes"
            )));
        }

        kv::set_bytes(MEMORY_KEY, input.content.as_bytes())?;

        Ok(serde_json::json!({
            "status": "ok",
            "bytes_written": input.content.len()
        }))
    }
}
