//! Meteor Distributed Data Protocol (DDP) method mapping and stubbed client.
//!
//! Provides protocol mapping for Meteor RPC calls (login, giveHotness, addComment, reportHandler).

use crate::auth::{HubSessionToken, MeteorPasswordDigest};
use crate::types::{
    CommentSubmission, HotnessVoteResult, HubComment, ReportReason, UploadHandlerRequest,
    UploadedHandlerResult,
};
use nucleus_common::error::{NucleusError, NucleusResult};
use serde::{Deserialize, Serialize};

/// Configuration mapping of remote Meteor method names.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdpMethodMap {
    pub login: String,
    pub give_hotness: String,
    pub add_comment: String,
    pub report_handler: String,
    pub upload_handler: String,
}

impl Default for DdpMethodMap {
    fn default() -> Self {
        Self {
            login: "login".to_string(),
            give_hotness: "giveHotness".to_string(),
            add_comment: "addComment".to_string(),
            report_handler: "reportHandler".to_string(),
            upload_handler: "uploadHandler".to_string(),
        }
    }
}

/// Stubbed/mockable DDP transport for Community Hub authenticated actions.
#[derive(Debug, Clone, Default)]
pub struct DdpClient {
    methods: DdpMethodMap,
    simulate: bool,
}

impl DdpClient {
    /// Creates a new `DdpClient` with default method mappings.
    pub fn new() -> Self {
        Self {
            methods: DdpMethodMap::default(),
            simulate: true,
        }
    }

    /// Sets whether the client runs in local simulation/stub mode.
    pub fn with_simulation(mut self, simulate: bool) -> Self {
        self.simulate = simulate;
        self
    }

    /// Authenticates with username/email and hashed password.
    pub async fn login_with_password(
        &self,
        identifier: &str,
        digest: &MeteorPasswordDigest,
    ) -> NucleusResult<HubSessionToken> {
        if identifier.trim().is_empty() {
            return Err(NucleusError::Auth(
                "Username or email cannot be empty".to_string(),
            ));
        }

        // Stubbed response as requested by user
        tracing::info!(
            "DdpClient: Authenticating '{}' using method '{}' (digest: {}...)",
            identifier,
            self.methods.login,
            &digest.digest[..8.min(digest.digest.len())]
        );

        Ok(HubSessionToken {
            user_id: format!("usr_{}", hex::encode(&digest.digest.as_bytes()[..6])),
            username: identifier.to_string(),
            token: format!("sim_tok_{}", hex::encode(&digest.digest.as_bytes()[6..18])),
            token_expires_unix_ms: Some(1775000000000),
        })
    }

    /// Resumes an existing session token.
    pub async fn login_with_resume_token(&self, token: &str) -> NucleusResult<HubSessionToken> {
        if token.trim().is_empty() {
            return Err(NucleusError::Auth("Resume token is empty".to_string()));
        }

        tracing::info!("DdpClient: Resuming session with token '{}'", token);

        Ok(HubSessionToken {
            user_id: "usr_resumed".to_string(),
            username: "ResumedUser".to_string(),
            token: token.to_string(),
            token_expires_unix_ms: Some(1775000000000),
        })
    }

    /// Casts a hotness vote for a handler.
    pub async fn give_hotness(
        &self,
        session: &HubSessionToken,
        handler_id: &str,
    ) -> NucleusResult<HotnessVoteResult> {
        tracing::info!(
            "DdpClient: '{}' giving hotness to '{}' via method '{}'",
            session.username,
            handler_id,
            self.methods.give_hotness
        );

        Ok(HotnessVoteResult {
            handler_id: handler_id.to_string(),
            total_hotness: 42,
            voted: true,
        })
    }

    /// Posts a comment or reply to a handler.
    pub async fn post_comment(
        &self,
        session: &HubSessionToken,
        req: CommentSubmission,
    ) -> NucleusResult<HubComment> {
        if req.content.trim().is_empty() {
            return Err(NucleusError::Other(
                "Comment content cannot be empty".to_string(),
            ));
        }

        tracing::info!(
            "DdpClient: '{}' posting comment on '{}' via method '{}'",
            session.username,
            req.handler_id,
            self.methods.add_comment
        );

        Ok(HubComment {
            id: format!(
                "cmt_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            ),
            author: session.username.clone(),
            owner: Some(session.user_id.clone()),
            reply_to: req.reply_to,
            content: req.content,
            created_at: "Just now".to_string(),
        })
    }

    /// Submits a moderation report for a handler.
    pub async fn report_handler(
        &self,
        session: &HubSessionToken,
        handler_id: &str,
        reason: ReportReason,
        details: &str,
    ) -> NucleusResult<()> {
        tracing::info!(
            "DdpClient: '{}' reporting handler '{}' ({:?}) via method '{}': {}",
            session.username,
            handler_id,
            reason,
            self.methods.report_handler,
            details
        );
        Ok(())
    }

    /// Packages and uploads a handler.
    pub async fn upload_handler(
        &self,
        session: &HubSessionToken,
        req: UploadHandlerRequest,
    ) -> NucleusResult<UploadedHandlerResult> {
        tracing::info!(
            "DdpClient: '{}' uploading handler '{}' ({} bytes) via method '{}'",
            session.username,
            req.title,
            req.archive_bytes.len(),
            self.methods.upload_handler
        );

        Ok(UploadedHandlerResult {
            handler_id: req.handler_id.unwrap_or_else(|| "h_new_123".to_string()),
            package_id: "pkg_new_456".to_string(),
            version: 1,
            message: "Handler package uploaded successfully!".to_string(),
        })
    }
}
