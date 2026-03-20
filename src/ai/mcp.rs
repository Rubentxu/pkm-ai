//! MCP (Model Context Protocol) integration for AI agents
//!
//! This module provides an MCP server that exposes PKM-AI tools to AI agents
//! following the Model Context Protocol specification.

use crate::ai::{GhostDetector, LinkSuggester};
use crate::db::Database;
use crate::models::{Block, BlockType, Edge, FractionalIndex, LinkType};
use crate::spine::SpineEngine;
use crate::synthesis::{OutputFormat, Synthesizer};
use crate::versioning::{AgentId, VersionRepo};
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, ErrorCode, Implementation, InitializeResult,
    ListToolsResult, ServerCapabilities, Tool,
};
use rmcp::service::{RoleServer, ServiceExt};
use rmcp::transport::io::stdio;
use rmcp::ServerHandler;
use rmcp::ErrorData;
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use ulid::Ulid;

// =============================================================================
// Rate Limiter Implementation
// =============================================================================

/// Token Bucket rate limiter for MCP server
#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    /// Create a new RateLimiter
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    /// Check if a request is allowed for the given agent_id
    pub async fn check(&self, agent_id: &str) -> Result<(), ErrorData> {
        let mut requests = self.requests.write().await;
        let now = Instant::now();

        // Clean old entries
        for (_, times) in requests.iter_mut() {
            times.retain(|&t| now.duration_since(t) < self.window);
        }

        // Count requests from this agent
        let count = requests.get(agent_id).map(|v| v.len()).unwrap_or(0);

        if count >= self.max_requests {
            return Err(ErrorData::new(
                ErrorCode(-32001),
                format!(
                    "Rate limit exceeded: {} requests per {} seconds",
                    self.max_requests,
                    self.window.as_secs()
                ),
                None,
            ));
        }

        requests
            .entry(agent_id.to_string())
            .or_insert_with(Vec::new)
            .push(now);

        Ok(())
    }
}

// =============================================================================
// MCP Server Implementation
// =============================================================================

/// MCP Server for PKM-AI
#[allow(dead_code)]
#[derive(Clone)]
pub struct NexusMcpServer {
    /// Database connection
    db: Arc<Database>,
    /// Link suggester for AI-powered link suggestions
    link_suggester: LinkSuggester,
    /// Ghost detector for identifying gaps
    ghost_detector: GhostDetector,
    /// Rate limiter for API protection
    rate_limiter: RateLimiter,
}

// =============================================================================
// Input Validation & Sanitization
// =============================================================================

impl NexusMcpServer {
    const MAX_CONTENT_LENGTH: usize = 1_000_000; // 1MB
    const MAX_TAG_LENGTH: usize = 50;
    const MAX_TAGS_COUNT: usize = 20;
    const MAX_TITLE_LENGTH: usize = 500;
    const MAX_PROPERTY_KEY_LENGTH: usize = 100;
    const MAX_PROPERTY_VALUE_LENGTH: usize = 10_000;

    /// Sanitizes a string to prevent XSS by removing null bytes and limiting length
    fn sanitize_string(input: &str) -> String {
        let cleaned = input.replace('\0', "");
        cleaned.chars().take(Self::MAX_CONTENT_LENGTH).collect()
    }

    /// Sanitizes a title by removing null bytes, trimming, and limiting length
    fn sanitize_title(input: &str) -> String {
        let cleaned = input.replace('\0', "").trim().to_string();
        cleaned.chars().take(Self::MAX_TITLE_LENGTH).collect()
    }

    /// Validates and sanitizes tags, returning an error for invalid tags
    fn validate_tags(tags: Vec<String>) -> Result<Vec<String>, ErrorData> {
        if tags.len() > Self::MAX_TAGS_COUNT {
            return Err(ErrorData::invalid_params(
                format!("Maximum {} tags allowed, got {}", Self::MAX_TAGS_COUNT, tags.len()),
                None,
            ));
        }

        let mut sanitized = Vec::with_capacity(tags.len());
        for tag in tags {
            let cleaned = tag.replace('\0', "").trim().to_lowercase();

            if cleaned.is_empty() {
                continue;
            }

            if cleaned.len() > Self::MAX_TAG_LENGTH {
                return Err(ErrorData::invalid_params(
                    format!("Tag '{}' exceeds {} characters", cleaned, Self::MAX_TAG_LENGTH),
                    None,
                ));
            }

            // Only alphanumeric, hyphens, and underscores allowed
            if !cleaned.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                return Err(ErrorData::invalid_params(
                    format!("Tag '{}' contains invalid characters. Use only a-z, 0-9, -, _", cleaned),
                    None,
                ));
            }

            sanitized.push(cleaned);
        }

        Ok(sanitized)
    }

    /// Validates and sanitizes properties (metadata) with prototype pollution protection
    fn validate_properties(props: Map<String, Value>) -> Result<Map<String, Value>, ErrorData> {
        let mut validated = Map::new();

        for (key, value) in props {
            // Sanitize key
            let clean_key = key.replace('\0', "").trim().to_string();
            if clean_key.len() > Self::MAX_PROPERTY_KEY_LENGTH {
                return Err(ErrorData::invalid_params(
                    format!("Property key exceeds {} characters", Self::MAX_PROPERTY_KEY_LENGTH),
                    None,
                ));
            }

            // Prevent prototype pollution attacks
            if clean_key == "__proto__" || clean_key == "constructor" || clean_key == "prototype" {
                continue; // Ignore dangerous keys
            }

            // Sanitize value if it's a string
            let clean_value = match value {
                Value::String(s) => Value::String(Self::sanitize_string(&s)),
                Value::Number(n) => Value::Number(n),
                Value::Bool(b) => Value::Bool(b),
                Value::Null => Value::Null,
                _ => Value::String(Self::sanitize_string(&serde_json::to_string(&value).unwrap_or_default())),
            };

            validated.insert(clean_key, clean_value);
        }

        Ok(validated)
    }
}

#[allow(dead_code)]
impl NexusMcpServer {
    /// Create a new MCP server instance
    pub fn new(db: Database) -> Self {
        Self {
            db: Arc::new(db),
            link_suggester: LinkSuggester::new(),
            ghost_detector: GhostDetector::new(),
            rate_limiter: RateLimiter::new(100, 60), // 100 req/min
        }
    }

    /// Run the MCP server using stdio transport
    pub async fn run(self) -> anyhow::Result<()> {
        let transport = stdio();

        let service = self;
        service
            .serve(transport)
            .await
            .map_err(|e| anyhow::anyhow!("MCP server error: {}", e))?;

        Ok(())
    }
}

// =============================================================================
// ServerHandler Implementation
// =============================================================================

impl ServerHandler for NexusMcpServer {
    fn get_info(&self) -> InitializeResult {
        InitializeResult::new(ServerCapabilities::default())
            .with_server_info(Implementation::new("pkm-ai", env!("CARGO_PKG_VERSION")))
            .with_instructions("PKM-AI MCP Server - Knowledge Graph Management")
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        // Extract agent_id from context or use default
        let agent_id = request
            .meta
            .as_ref()
            .and_then(|m| m.0.get("agent_id").and_then(|v| v.as_str()))
            .unwrap_or("anonymous");

        // Rate limit check
        self.rate_limiter.check(agent_id).await?;

        let arguments = request.arguments.unwrap_or_default();

        let result = match request.name.as_ref() {
            // Block tools
            "create_block" => self.create_block(&arguments).await,
            "get_block" => self.get_block(&arguments).await,
            "search_blocks" => self.search_blocks(&arguments).await,
            "update_block" => self.update_block(&arguments).await,
            "delete_block" => self.delete_block(&arguments).await,

            // Link tools
            "create_link" => self.create_link(&arguments).await,
            "delete_link" => self.delete_link(&arguments).await,
            "get_links" => self.get_links(&arguments).await,
            "suggest_links" => self.suggest_links(&arguments).await,

            // Spine tools
            "traverse_spine" => self.traverse_spine(&arguments).await,
            "gravity_check" => self.gravity_check(&arguments).await,
            "reorder_block" => self.reorder_block(&arguments).await,

            // Structure tools
            "get_section_map" => self.get_section_map(&arguments).await,
            "detect_gaps" => self.detect_gaps(&arguments).await,
            "list_ghosts" => self.list_ghosts(&arguments).await,

            // Synthesis tools
            "synthesize" => self.synthesize(&arguments).await,
            "get_toc" => self.get_toc(&arguments).await,

            // Versioning tools
            "stage_block" => self.stage_block(&arguments).await,
            "commit_changes" => self.commit_changes(&arguments).await,
            "get_working_set_status" => self.get_working_set_status().await,
            "unstage_block" => self.unstage_block(&arguments).await,
            "discard_working_set" => self.discard_working_set().await,

            _ => {
                return Err(ErrorData::new(
                    ErrorCode::METHOD_NOT_FOUND,
                    format!("Unknown tool: {}", request.name),
                    None,
                ))
            }
        };

        match result {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(output)])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!("Error: {}", e))])),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult {
            tools: vec![
                // Block tools
                Tool::new(
                    "create_block",
                    "Create a new block in the knowledge graph",
                    create_block_schema(),
                ),
                Tool::new(
                    "get_block",
                    "Get a block by ID",
                    get_block_schema(),
                ),
                Tool::new(
                    "search_blocks",
                    "Search blocks by query, type, or tags",
                    search_blocks_schema(),
                ),
                Tool::new(
                    "update_block",
                    "Update a block's content or properties",
                    update_block_schema(),
                ),
                Tool::new(
                    "delete_block",
                    "Delete a block and all its associated edges",
                    delete_block_schema(),
                ),

                // Link tools
                Tool::new(
                    "create_link",
                    "Create a link between two blocks",
                    create_link_schema(),
                ),
                Tool::new(
                    "delete_link",
                    "Delete a link between two blocks",
                    delete_link_schema(),
                ),
                Tool::new(
                    "get_links",
                    "Get all links from or to a block",
                    get_links_schema(),
                ),
                Tool::new(
                    "suggest_links",
                    "Suggest links for a block using AI",
                    suggest_links_schema(),
                ),

                // Spine tools
                Tool::new(
                    "traverse_spine",
                    "Traverse the structural spine from a root",
                    traverse_spine_schema(),
                ),
                Tool::new(
                    "gravity_check",
                    "Check gravity/connectivity of a block",
                    gravity_check_schema(),
                ),
                Tool::new(
                    "reorder_block",
                    "Reorder a block in the spine",
                    reorder_block_schema(),
                ),

                // Structure tools
                Tool::new(
                    "get_section_map",
                    "Get the section hierarchy from a root",
                    get_section_map_schema(),
                ),
                Tool::new(
                    "detect_gaps",
                    "Detect gaps in a section",
                    detect_gaps_schema(),
                ),
                Tool::new(
                    "list_ghosts",
                    "List ghost nodes (content placeholders)",
                    list_ghosts_schema(),
                ),

                // Synthesis tools
                Tool::new(
                    "synthesize",
                    "Synthesize a document from a structure",
                    synthesize_schema(),
                ),
                Tool::new(
                    "get_toc",
                    "Get table of contents for a structure",
                    get_toc_schema(),
                ),

                // Versioning tools (Git-like workflow)
                Tool::new(
                    "stage_block",
                    "Stage a block for commit (git-like workflow)",
                    stage_block_schema(),
                ),
                Tool::new(
                    "commit_changes",
                    "Commit all staged changes with a message",
                    commit_changes_schema(),
                ),
                Tool::new(
                    "get_working_set_status",
                    "Get current staging area status",
                    get_working_set_status_schema(),
                ),
                Tool::new(
                    "unstage_block",
                    "Remove a block from the staging area (git-like workflow)",
                    unstage_block_schema(),
                ),
                Tool::new(
                    "discard_working_set",
                    "Discard all staged changes (clear the staging area)",
                    discard_working_set_schema(),
                ),
            ],
            next_cursor: None,
            meta: None,
        })
    }
}

// =============================================================================
// JSON Schemas
// =============================================================================

fn create_block_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "block_type": {
                "type": "string",
                "enum": ["fleeting", "literature", "permanent", "structure", "hub", "task", "reference", "outline", "ghost"],
                "description": "Type of block to create"
            },
            "content": {
                "type": "string",
                "description": "Block content (Markdown)"
            },
            "title": {
                "type": "string",
                "description": "Block title (for Structure/Hub/Permanent)"
            },
            "tags": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Tags for classification"
            },
            "enrich": {
                "type": "boolean",
                "default": false,
                "description": "Enable enriched response with link suggestions, tag suggestions, gravity info, and type recommendations"
            }
        },
        "required": ["block_type"]
    })).unwrap())
}

fn get_block_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "block_id": {"type": "string", "description": "Block ULID"},
            "include_content": {"type": "boolean", "default": true}
        },
        "required": ["block_id"]
    })).unwrap())
}

fn search_blocks_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "query": {"type": "string", "description": "Search query"},
            "block_type": {
                "type": "string",
                "enum": ["fleeting", "literature", "permanent", "structure", "hub", "task", "reference", "outline", "ghost"]
            },
            "limit": {"type": "integer", "default": 20}
        }
    })).unwrap())
}

fn update_block_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "block_id": {"type": "string", "description": "Block ULID"},
            "content": {"type": "string", "description": "New content"},
            "properties": {"type": "object", "description": "Properties to update"}
        },
        "required": ["block_id"]
    })).unwrap())
}

fn delete_block_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "block_id": {"type": "string", "description": "Block ULID to delete"},
            "force": {
                "type": "boolean",
                "default": false,
                "description": "If true, delete even if block has incoming links (will delete all associated edges)"
            }
        },
        "required": ["block_id"]
    })).unwrap())
}

fn create_link_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "source_id": {"type": "string", "description": "Source block ULID"},
            "target_id": {"type": "string", "description": "Target block ULID"},
            "link_type": {
                "type": "string",
                "enum": ["extends", "refines", "contradicts", "questions", "supports", "references", "related", "similar_to", "section_of", "subsection_of", "ordered_child", "next", "next_sibling", "first_child", "contains", "parent", "ai_suggested"]
            }
        },
        "required": ["source_id", "target_id", "link_type"]
    })).unwrap())
}

fn delete_link_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "source_id": {"type": "string", "description": "Source block ULID"},
            "target_id": {"type": "string", "description": "Target block ULID"}
        },
        "required": ["source_id", "target_id"]
    })).unwrap())
}

fn get_links_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "block_id": {"type": "string", "description": "Block ULID"},
            "link_types": {
                "type": "array",
                "items": {"type": "string"}
            },
            "direction": {
                "type": "string",
                "enum": ["outgoing", "incoming", "both"],
                "default": "both"
            }
        },
        "required": ["block_id"]
    })).unwrap())
}

fn suggest_links_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "block_id": {"type": "string", "description": "Block ULID"},
            "confidence_threshold": {"type": "number", "default": 0.5},
            "limit": {"type": "integer", "default": 10}
        },
        "required": ["block_id"]
    })).unwrap())
}

fn traverse_spine_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "root_id": {"type": "string", "description": "Root structure block ULID"},
            "max_depth": {"type": "integer", "default": 0}
        }
    })).unwrap())
}

fn gravity_check_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "block_id": {"type": "string", "description": "Block ULID"}
        },
        "required": ["block_id"]
    })).unwrap())
}

fn reorder_block_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "block_id": {"type": "string", "description": "Block ULID"},
            "after_id": {"type": "string", "description": "Block ULID to place after"},
            "before_id": {"type": "string", "description": "Block ULID to place before"}
        },
        "required": ["block_id"]
    })).unwrap())
}

fn get_section_map_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "root_id": {"type": "string", "description": "Root structure block ULID"}
        },
        "required": ["root_id"]
    })).unwrap())
}

fn detect_gaps_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "section_id": {"type": "string", "description": "Section block ULID"}
        },
        "required": ["section_id"]
    })).unwrap())
}

fn list_ghosts_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "status": {
                "type": "string",
                "enum": ["detected", "acknowledged", "in_progress", "filled", "dismissed"]
            },
            "confidence_below": {"type": "number"}
        }
    })).unwrap())
}

fn synthesize_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "structure_id": {"type": "string", "description": "Structure block ULID"},
            "template": {"type": "string", "default": "default"},
            "output_path": {"type": "string"}
        },
        "required": ["structure_id"]
    })).unwrap())
}

fn get_toc_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "structure_id": {"type": "string", "description": "Structure block ULID"}
        },
        "required": ["structure_id"]
    })).unwrap())
}

fn stage_block_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "block_id": {"type": "string", "description": "Block ULID to stage for commit"}
        },
        "required": ["block_id"]
    })).unwrap())
}

fn commit_changes_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "message": {"type": "string", "description": "Commit message describing the changes"}
        },
        "required": ["message"]
    })).unwrap())
}

fn get_working_set_status_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {}
    })).unwrap())
}

fn unstage_block_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "block_id": {"type": "string", "description": "Block ULID to unstage"}
        },
        "required": ["block_id"]
    })).unwrap())
}

fn discard_working_set_schema() -> Arc<Map<String, Value>> {
    Arc::new(serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {}
    })).unwrap())
}

// =============================================================================
// Tool Implementations
// =============================================================================

impl NexusMcpServer {
    /// Get a string value from arguments map
    fn get_string(args: &Map<String, Value>, key: &str) -> Result<String, ErrorData> {
        args.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ErrorData::invalid_params(format!("Missing or invalid parameter: {}", key), None))
    }

    /// Get an optional string value from arguments map
    fn get_opt_string(args: &Map<String, Value>, key: &str) -> Option<String> {
        args.get(key).and_then(|v| v.as_str().map(|s| s.to_string()))
    }

    #[allow(dead_code)]
    /// Get an f32 value from arguments map
    fn get_f32(args: &Map<String, Value>, key: &str) -> Result<f32, ErrorData> {
        args.get(key)
            .and_then(|v| v.as_f64())
            .map(|f| f as f32)
            .ok_or_else(|| ErrorData::invalid_params(format!("Missing or invalid parameter: {}", key), None))
    }

    /// Get an optional f32 value from arguments map
    fn get_opt_f32(args: &Map<String, Value>, key: &str) -> Option<f32> {
        args.get(key).and_then(|v| v.as_f64().map(|f| f as f32))
    }

    #[allow(dead_code)]
    /// Get an i64 value from arguments map
    fn get_i64(args: &Map<String, Value>, key: &str) -> Result<i64, ErrorData> {
        args.get(key)
            .and_then(|v| v.as_i64())
            .ok_or_else(|| ErrorData::invalid_params(format!("Missing or invalid parameter: {}", key), None))
    }

    /// Get an optional i64 value from arguments map
    fn get_opt_i64(args: &Map<String, Value>, key: &str) -> Option<i64> {
        args.get(key).and_then(|v| v.as_i64())
    }

    /// Get a boolean value from arguments map (default false)
    fn get_bool(args: &Map<String, Value>, key: &str) -> bool {
        args.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
    }

    /// Get link suggestions for a newly created block
    async fn get_link_suggestions_for_block(&self, block: &Block) -> Vec<Value> {
        let Ok(candidates) = self.db.blocks().list_all().await else {
            return vec![];
        };

        let filtered: Vec<Block> = self.link_suggester.filter_candidates(&candidates);
        let exclude_ids: HashSet<Ulid> = [block.id].into_iter().collect();

        let Ok(suggestions) = self
            .link_suggester
            .suggest_outgoing(block, &filtered, Some(&exclude_ids))
            .await
        else {
            return vec![];
        };

        suggestions
            .into_iter()
            .take(10)
            .map(|s| {
                serde_json::json!({
                    "target_id": s.target_id.to_string(),
                    "link_type": format!("{:?}", s.link_type).to_lowercase(),
                    "confidence": s.confidence,
                    "reason": s.reason
                })
            })
            .collect()
    }

    /// Get tag suggestions based on content similarity
    async fn get_tag_suggestions_for_block(&self, block: &Block) -> Vec<String> {
        let Ok(all_blocks) = self.db.blocks().list_all().await else {
            return vec![];
        };

        let mut tag_scores: std::collections::HashMap<String, f32> = std::collections::HashMap::new();

        // Pre-compute block words once outside the loop
        let block_lower = block.content.to_lowercase();
        let block_words: std::collections::HashSet<_> = block_lower.split_whitespace().collect();

        for other in all_blocks.iter().filter(|b| b.id != block.id && !b.tags.is_empty()) {
            // Calculate simple similarity based on content words
            let other_lower = other.content.to_lowercase();
            let other_words: std::collections::HashSet<_> = other_lower.split_whitespace().collect();

            if !block_words.is_empty() {
                let intersection: usize = block_words.intersection(&other_words).count();
                let union: usize = block_words.union(&other_words).count();
                let jaccard = if union > 0 {
                    intersection as f32 / union as f32
                } else {
                    0.0
                };

                for tag in &other.tags {
                    *tag_scores.entry(tag.clone()).or_insert(0.0) += jaccard;
                }
            }
        }

        let mut tags: Vec<_> = tag_scores.into_iter().collect();
        tags.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        tags.into_iter()
            .take(5)
            .map(|(tag, _)| tag)
            .collect()
    }

    /// Get gravity/connectivity info for a block
    async fn get_gravity_info_for_block(&self, block_id: &Ulid) -> Value {
        let outgoing = self.db.edges().outgoing_from(block_id).await.unwrap_or_default();
        let incoming = self.db.edges().incoming_to(block_id).await.unwrap_or_default();
        let outgoing_count = outgoing.len();
        let incoming_count = incoming.len();

        serde_json::json!({
            "gravity_score": (outgoing_count + incoming_count) as f32,
            "outgoing_links": outgoing_count,
            "incoming_links": incoming_count,
            "total_connections": outgoing_count + incoming_count
        })
    }

    /// Detect if block type might be mismatched based on content
    fn detect_type_mismatch(block: &Block, requested_type: &BlockType) -> Option<Value> {
        let content_lower = block.content.to_lowercase();
        let title_lower = block.title.to_lowercase();

        // Simple heuristics for type detection
        let is_question = content_lower.contains('?') || title_lower.starts_with("how") || title_lower.starts_with("what") || title_lower.starts_with("why");
        let is_reference = content_lower.contains("http://") || content_lower.contains("https://") || content_lower.starts_with("# ");
        let is_task = title_lower.starts_with("todo") || title_lower.starts_with("- [ ]") || content_lower.contains("due date") || content_lower.contains("deadline");
        let is_structure = block.content.is_empty() && matches!(requested_type, BlockType::Structure | BlockType::Hub | BlockType::Outline);

        if is_question && matches!(requested_type, BlockType::Permanent) {
            return Some(serde_json::json!({
                "suggested_type": "literature",
                "confidence": 0.6,
                "reasoning": "Content appears to be a question, consider using literature type for source references"
            }));
        }
        if is_reference && matches!(requested_type, BlockType::Permanent) {
            return Some(serde_json::json!({
                "suggested_type": "literature",
                "confidence": 0.7,
                "reasoning": "Content appears to contain external references, consider literature type"
            }));
        }
        if is_task && !matches!(requested_type, BlockType::Task) {
            return Some(serde_json::json!({
                "suggested_type": "task",
                "confidence": 0.8,
                "reasoning": "Content appears to be an action item, consider task type"
            }));
        }
        if is_structure {
            return Some(serde_json::json!({
                "suggested_type": "structure",
                "confidence": 0.9,
                "reasoning": "Empty content with structural purpose, consider structure type"
            }));
        }

        None
    }

    /// Create a new block
    pub(crate) async fn create_block(&self, args: &Map<String, Value>) -> Result<String, ErrorData> {
        let block_type_str = Self::get_string(args, "block_type")?;
        let content = Self::get_opt_string(args, "content").map(|c| Self::sanitize_string(&c));
        let title = Self::get_opt_string(args, "title")
            .map(|t| Self::sanitize_title(&t))
            .unwrap_or_else(|| "Untitled".to_string());
        let tags = args.get("tags").and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect()
            })
        });
        let enrich = Self::get_bool(args, "enrich");

        let block_type = match block_type_str.to_lowercase().as_str() {
            "fleeting" => BlockType::Fleeting,
            "literature" => BlockType::Literature,
            "permanent" => BlockType::Permanent,
            "structure" => BlockType::Structure,
            "hub" => BlockType::Hub,
            "task" => BlockType::Task,
            "reference" => BlockType::Reference,
            "outline" => BlockType::Outline,
            "ghost" => BlockType::Ghost,
            _ => {
                return Err(ErrorData::invalid_params(
                    format!("Unknown block type: {}", block_type_str),
                    None,
                ))
            }
        };

        let mut block = Block::new(block_type, title);

        if let Some(c) = content {
            block.content = c;
        }

        // Validate and sanitize tags
        if let Some(t) = tags {
            block.tags = Self::validate_tags(t)?;
        }

        let requested_type = block.block_type.clone();
        let block_type_str = format!("{:?}", block.block_type).to_lowercase();
        let block_title = block.title.clone();
        let block_id_str = block.id.to_string();
        let block_created_at = block.created_at;

        self.db.blocks().create(block.clone()).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to create block: {}", e), None)
        })?;

        if enrich {
            // Fire all enrichment calls in parallel
            let (link_suggestions, tag_suggestions, gravity_info) = tokio::join!(
                self.get_link_suggestions_for_block(&block),
                self.get_tag_suggestions_for_block(&block),
                self.get_gravity_info_for_block(&block.id)
            );

            let mut response = serde_json::json!({
                "id": block_id_str,
                "block_type": block_type_str,
                "title": block_title,
                "created_at": block_created_at,
                "link_suggestions": link_suggestions,
                "tag_suggestions": tag_suggestions,
                "gravity_info": gravity_info
            });

            // Add type suggestion if detected type differs from requested
            if let Some(type_suggestion) = Self::detect_type_mismatch(&block, &requested_type) {
                response["type_suggestion"] = type_suggestion;
            }

            serde_json::to_string_pretty(&response)
                .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
        } else {
            serde_json::to_string_pretty(&serde_json::json!({
                "id": block_id_str,
                "block_type": block_type_str,
                "title": block_title,
                "created_at": block_created_at
            }))
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
        }
    }

    /// Get a block by ID
    pub(crate) async fn get_block(&self, args: &Map<String, Value>) -> Result<String, ErrorData> {
        let block_id = Self::get_string(args, "block_id")?;
        let include_content = Self::get_bool(args, "include_content");

        let block_id = Ulid::from_string(&block_id)
            .map_err(|e| ErrorData::invalid_params(format!("Invalid ULID: {}", e), None))?;

        let block = self.db.blocks().get(&block_id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to get block: {}", e), None)
        })?;

        let block = block.ok_or_else(|| {
            ErrorData::invalid_params(format!("Block not found: {}", block_id), None)
        })?;

        let output = if include_content {
            serde_json::json!({
                "id": block.id.to_string(),
                "block_type": format!("{:?}", block.block_type).to_lowercase(),
                "title": block.title,
                "content": block.content,
                "tags": block.tags,
                "metadata": block.metadata,
                "created_at": block.created_at,
                "updated_at": block.updated_at
            })
        } else {
            serde_json::json!({
                "id": block.id.to_string(),
                "block_type": format!("{:?}", block.block_type).to_lowercase(),
                "title": block.title,
                "tags": block.tags,
                "created_at": block.created_at,
                "updated_at": block.updated_at
            })
        };

        serde_json::to_string_pretty(&output)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
    }

    /// Search blocks
    pub(crate) async fn search_blocks(&self, args: &Map<String, Value>) -> Result<String, ErrorData> {
        let query = Self::get_opt_string(args, "query");
        let block_type = Self::get_opt_string(args, "block_type");
        let limit = Self::get_opt_i64(args, "limit").unwrap_or(20).min(100) as usize;

        let blocks = if let Some(q) = query {
            // Full-text search
            self.db.blocks().search_content(&q).await.map_err(|e| {
                ErrorData::internal_error(format!("Search failed: {}", e), None)
            })?
        } else if let Some(type_str) = block_type {
            // Filter by type
            let block_type = match type_str.to_lowercase().as_str() {
                "fleeting" => BlockType::Fleeting,
                "literature" => BlockType::Literature,
                "permanent" => BlockType::Permanent,
                "structure" => BlockType::Structure,
                "hub" => BlockType::Hub,
                "task" => BlockType::Task,
                "reference" => BlockType::Reference,
                "outline" => BlockType::Outline,
                "ghost" => BlockType::Ghost,
                _ => {
                    return Err(ErrorData::invalid_params(
                        format!("Unknown block type: {}", type_str),
                        None,
                    ))
                }
            };

            self.db.blocks().list_by_type(block_type).await.map_err(|e| {
                ErrorData::internal_error(format!("Failed to list blocks: {}", e), None)
            })?
        } else {
            // List all
            self.db.blocks().list_all().await.map_err(|e| {
                ErrorData::internal_error(format!("Failed to list blocks: {}", e), None)
            })?
        };

        let blocks: Vec<Value> = blocks
            .into_iter()
            .take(limit)
            .map(|b| {
                serde_json::json!({
                    "id": b.id.to_string(),
                    "block_type": format!("{:?}", b.block_type).to_lowercase(),
                    "title": b.title,
                    "content": b.content.chars().take(200).collect::<String>(),
                    "tags": b.tags,
                    "created_at": b.created_at
                })
            })
            .collect();

        serde_json::to_string_pretty(&serde_json::json!({
            "blocks": blocks,
            "count": blocks.len()
        }))
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
    }

    /// Update a block
    pub(crate) async fn update_block(&self, args: &Map<String, Value>) -> Result<String, ErrorData> {
        let block_id = Self::get_string(args, "block_id")?;
        let content = Self::get_opt_string(args, "content").map(|c| Self::sanitize_string(&c));
        let properties = args.get("properties").and_then(|v| v.as_object().cloned());

        let block_id = Ulid::from_string(&block_id)
            .map_err(|e| ErrorData::invalid_params(format!("Invalid ULID: {}", e), None))?;

        let block = self.db.blocks().get(&block_id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to get block: {}", e), None)
        })?;

        let mut block = block.ok_or_else(|| {
            ErrorData::invalid_params(format!("Block not found: {}", block_id), None)
        })?;

        if let Some(c) = content {
            block.content = c;
        }

        // Validate and sanitize properties
        if let Some(props) = properties {
            let validated_props = Self::validate_properties(props)?;
            for (key, value) in validated_props {
                block.metadata.insert(key, value);
            }
        }

        self.db.blocks().update(block.clone()).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to update block: {}", e), None)
        })?;

        Ok(format!("Block {} updated successfully", block.id))
    }

    /// Delete a block and all its associated edges
    pub(crate) async fn delete_block(&self, args: &Map<String, Value>) -> Result<String, ErrorData> {
        let block_id = Self::get_string(args, "block_id")?;
        let force = Self::get_bool(args, "force");

        let block_id = Ulid::from_string(&block_id)
            .map_err(|e| ErrorData::invalid_params(format!("Invalid ULID: {}", e), None))?;

        // Check if block exists
        let block = self.db.blocks().get(&block_id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to get block: {}", e), None)
        })?;

        if block.is_none() {
            return Err(ErrorData::invalid_params(format!("Block not found: {}", block_id), None));
        }

        // Check for incoming links if not forcing
        if !force {
            let incoming = self.db.edges().incoming_to(&block_id).await.map_err(|e| {
                ErrorData::internal_error(format!("Failed to check incoming links: {}", e), None)
            })?;

            if !incoming.is_empty() {
                return Err(ErrorData::invalid_params(
                    format!(
                        "Block {} has {} incoming link(s). Use force=true to delete anyway.",
                        block_id,
                        incoming.len()
                    ),
                    None,
                ));
            }
        }

        // Delete all edges associated with this block
        self.db.edges().delete_for_block(&block_id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to delete associated edges: {}", e), None)
        })?;

        // Delete the block
        self.db.blocks().delete(&block_id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to delete block: {}", e), None)
        })?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": block_id.to_string(),
            "status": "deleted",
            "message": format!("Block {} and all its edges were deleted", block_id)
        }))
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
    }

    /// Create a link between blocks
    pub(crate) async fn create_link(&self, args: &Map<String, Value>) -> Result<String, ErrorData> {
        let source_id = Self::get_string(args, "source_id")?;
        let target_id = Self::get_string(args, "target_id")?;
        let link_type_str = Self::get_string(args, "link_type")?;

        let source_id = Ulid::from_string(&source_id)
            .map_err(|e| ErrorData::invalid_params(format!("Invalid source ULID: {}", e), None))?;

        let target_id = Ulid::from_string(&target_id)
            .map_err(|e| ErrorData::invalid_params(format!("Invalid target ULID: {}", e), None))?;

        let link_type = parse_link_type(&link_type_str)?;

        let edge = Edge::new(source_id, target_id, link_type);

        self.db.edges().create(edge.clone()).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to create link: {}", e), None)
        })?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": edge.id.to_string(),
            "source_id": edge.from.to_string(),
            "target_id": edge.to.to_string(),
            "link_type": link_type_str
        }))
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
    }

    /// Delete a link between two blocks
    pub(crate) async fn delete_link(&self, args: &Map<String, Value>) -> Result<String, ErrorData> {
        let source_id = Self::get_string(args, "source_id")?;
        let target_id = Self::get_string(args, "target_id")?;

        let source_id = Ulid::from_string(&source_id)
            .map_err(|e| ErrorData::invalid_params(format!("Invalid source ULID: {}", e), None))?;

        let target_id = Ulid::from_string(&target_id)
            .map_err(|e| ErrorData::invalid_params(format!("Invalid target ULID: {}", e), None))?;

        // Verify source block exists
        let source_block = self.db.blocks().get(&source_id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to get source block: {}", e), None)
        })?;

        if source_block.is_none() {
            return Err(ErrorData::invalid_params(format!("Source block not found: {}", source_id), None));
        }

        // Verify target block exists
        let target_block = self.db.blocks().get(&target_id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to get target block: {}", e), None)
        })?;

        if target_block.is_none() {
            return Err(ErrorData::invalid_params(format!("Target block not found: {}", target_id), None));
        }

        // Find and delete the edge between source and target
        let edges = self.db.edges().outgoing_from(&source_id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to get outgoing edges: {}", e), None)
        })?;

        let edge_to_delete = edges.iter().find(|e| e.to == target_id);

        if let Some(edge) = edge_to_delete {
            self.db.edges().delete(&edge.id).await.map_err(|e| {
                ErrorData::internal_error(format!("Failed to delete edge: {}", e), None)
            })?;

            serde_json::to_string_pretty(&serde_json::json!({
                "source_id": source_id.to_string(),
                "target_id": target_id.to_string(),
                "edge_id": edge.id.to_string(),
                "status": "deleted",
                "message": format!("Link from {} to {} was deleted", source_id, target_id)
            }))
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
        } else {
            return Err(ErrorData::invalid_params(
                format!("No link found from {} to {}", source_id, target_id),
                None,
            ));
        }
    }

    /// Get links from/to a block
    pub(crate) async fn get_links(&self, args: &Map<String, Value>) -> Result<String, ErrorData> {
        let block_id = Self::get_string(args, "block_id")?;
        let link_types = args.get("link_types").and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>()
            })
        });
        let direction = Self::get_opt_string(args, "direction").unwrap_or_else(|| "both".to_string());

        let block_id = Ulid::from_string(&block_id)
            .map_err(|e| ErrorData::invalid_params(format!("Invalid ULID: {}", e), None))?;

        let mut edges = Vec::new();

        if direction == "outgoing" || direction == "both" {
            let outgoing = self.db.edges().outgoing_from(&block_id).await.map_err(|e| {
                ErrorData::internal_error(format!("Failed to get outgoing links: {}", e), None)
            })?;
            edges.extend(outgoing);
        }

        if direction == "incoming" || direction == "both" {
            let incoming = self.db.edges().incoming_to(&block_id).await.map_err(|e| {
                ErrorData::internal_error(format!("Failed to get incoming links: {}", e), None)
            })?;
            edges.extend(incoming);
        }

        // Filter by link types if specified
        if let Some(types) = link_types {
            let types: Vec<LinkType> = types
                .iter()
                .filter_map(|t| parse_link_type(t).ok())
                .collect();

            edges.retain(|e| types.contains(&e.link_type));
        }

        let output: Vec<Value> = edges
            .into_iter()
            .map(|e| {
                serde_json::json!({
                    "id": e.id.to_string(),
                    "from": e.from.to_string(),
                    "to": e.to.to_string(),
                    "link_type": format!("{:?}", e.link_type).to_lowercase(),
                    "sequence_weight": e.sequence_weight,
                    "created_at": e.created_at
                })
            })
            .collect();

        serde_json::to_string_pretty(&serde_json::json!({
            "edges": output,
            "count": output.len()
        }))
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
    }

    /// Suggest links for a block using AI
    pub(crate) async fn suggest_links(&self, args: &Map<String, Value>) -> Result<String, ErrorData> {
        let block_id = Self::get_string(args, "block_id")?;
        let threshold = Self::get_opt_f32(args, "confidence_threshold").unwrap_or(0.5);
        let limit = Self::get_opt_i64(args, "limit").unwrap_or(10) as usize;

        let block_id = Ulid::from_string(&block_id)
            .map_err(|e| ErrorData::invalid_params(format!("Invalid ULID: {}", e), None))?;

        let block = self.db.blocks().get(&block_id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to get block: {}", e), None)
        })?;

        let block = block.ok_or_else(|| {
            ErrorData::invalid_params(format!("Block not found: {}", block_id), None)
        })?;

        // Get all candidate blocks
        let candidates = self.db.blocks().list_all().await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to list blocks: {}", e), None)
        })?;

        let filtered: Vec<Block> = self.link_suggester.filter_candidates(&candidates);

        // Get existing outgoing links to exclude
        let existing = self.db.edges().outgoing_from(&block_id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to get existing links: {}", e), None)
        })?;

        let exclude_ids: std::collections::HashSet<_> = existing.iter().map(|e| e.to).collect();

        // Get suggestions
        let suggestions = self
            .link_suggester
            .suggest_outgoing(&block, &filtered, Some(&exclude_ids))
            .await
            .map_err(|e| {
                ErrorData::internal_error(format!("Suggestion failed: {}", e), None)
            })?;

        let filtered_suggestions: Vec<Value> = suggestions
            .into_iter()
            .filter(|s| s.confidence >= threshold)
            .take(limit)
            .map(|s| {
                serde_json::json!({
                    "target_id": s.target_id.to_string(),
                    "link_type": format!("{:?}", s.link_type).to_lowercase(),
                    "confidence": s.confidence,
                    "reason": s.reason
                })
            })
            .collect();

        serde_json::to_string_pretty(&serde_json::json!({
            "suggestions": filtered_suggestions,
            "count": filtered_suggestions.len()
        }))
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
    }

    /// Traverse the structural spine
    pub(crate) async fn traverse_spine(&self, args: &Map<String, Value>) -> Result<String, ErrorData> {
        let root_id = Self::get_opt_string(args, "root_id");
        let max_depth = Self::get_opt_i64(args, "max_depth").unwrap_or(0) as u32;

        let root_id = if let Some(id_str) = root_id {
            Some(
                Ulid::from_string(&id_str)
                    .map_err(|e| ErrorData::invalid_params(format!("Invalid root ULID: {}", e), None))?,
            )
        } else {
            None
        };

        let spine = SpineEngine::new(&self.db);

        let result = spine
            .traverse(root_id, max_depth)
            .await
            .map_err(|e| ErrorData::internal_error(format!("Traversal failed: {}", e), None))?;

        // Get block details for the traversed IDs
        let mut blocks = Vec::new();
        for block_id in &result.blocks {
            if let Some(block) = self.db.blocks().get(block_id).await.map_err(|e| {
                ErrorData::internal_error(format!("Failed to get block: {}", e), None)
            })? {
                blocks.push(serde_json::json!({
                    "id": block.id.to_string(),
                    "block_type": format!("{:?}", block.block_type).to_lowercase(),
                    "title": block.title
                }));
            }
        }

        serde_json::to_string_pretty(&serde_json::json!({
            "root_id": result.root_id.to_string(),
            "blocks": blocks,
            "total_count": result.total_count,
            "depth": result.depth
        }))
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
    }

    /// Check gravity/connectivity of a block
    pub(crate) async fn gravity_check(&self, args: &Map<String, Value>) -> Result<String, ErrorData> {
        let block_id = Self::get_string(args, "block_id")?;

        let block_id = Ulid::from_string(&block_id)
            .map_err(|e| ErrorData::invalid_params(format!("Invalid ULID: {}", e), None))?;

        // Check if block exists
        let block = self.db.blocks().get(&block_id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to get block: {}", e), None)
        })?;

        if block.is_none() {
            return Err(ErrorData::invalid_params(format!("Block not found: {}", block_id), None));
        }

        // Count connections
        let outgoing = self.db.edges().outgoing_from(&block_id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to get outgoing links: {}", e), None)
        })?;

        let incoming = self.db.edges().incoming_to(&block_id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to get incoming links: {}", e), None)
        })?;

        let outgoing_count = outgoing.len();
        let incoming_count = incoming.len();
        let total_connections = outgoing_count + incoming_count;

        // Calculate gravity score (simple metric: total connections)
        let gravity_score = total_connections as f32;

        serde_json::to_string_pretty(&serde_json::json!({
            "block_id": block_id.to_string(),
            "gravity_score": gravity_score,
            "outgoing_links": outgoing_count,
            "incoming_links": incoming_count,
            "total_connections": total_connections
        }))
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
    }

    /// Reorder a block in the spine
    pub(crate) async fn reorder_block(&self, args: &Map<String, Value>) -> Result<String, ErrorData> {
        let block_id = Self::get_string(args, "block_id")?;
        let after_id = Self::get_opt_string(args, "after_id");
        let before_id = Self::get_opt_string(args, "before_id");

        let block_id = Ulid::from_string(&block_id)
            .map_err(|e| ErrorData::invalid_params(format!("Invalid ULID: {}", e), None))?;

        let after_id: Option<Ulid> = after_id
            .map(|s| Ulid::from_string(&s))
            .transpose()
            .map_err(|e| ErrorData::invalid_params(format!("Invalid ULID: {}", e), None))?;

        let before_id: Option<Ulid> = before_id
            .map(|s| Ulid::from_string(&s))
            .transpose()
            .map_err(|e| ErrorData::invalid_params(format!("Invalid ULID: {}", e), None))?;

        // Validate that at least one of after_id or before_id is provided
        if after_id.is_none() && before_id.is_none() {
            return Err(ErrorData::invalid_params(
                "Either 'after_id' or 'before_id' must be provided".to_string(),
                None,
            ));
        }

        // Verify block exists
        let block = self.db.blocks().get(&block_id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to get block: {}", e), None)
        })?;

        if block.is_none() {
            return Err(ErrorData::invalid_params(format!("Block not found: {}", block_id), None));
        }

        // Calculate sequence weight using FractionalIndex
        // For inserting after after_id: we need the weight of after_id's NEXT edge (what after_id points to)
        let after_weight: Option<FractionalIndex> = if let Some(aid) = after_id {
            self.db.edges().outgoing_from(&aid).await.ok()
                .and_then(|edges| edges.iter().find(|e| e.link_type == LinkType::Next).map(|e| e.sequence_weight.clone()))
        } else {
            None
        };

        // For inserting before before_id: we need the weight of before_id's incoming NEXT edge (what points to before_id)
        let before_weight: Option<FractionalIndex> = if let Some(bid) = before_id {
            self.db.edges().incoming_to(&bid).await.ok()
                .and_then(|edges| edges.iter().find(|e| e.link_type == LinkType::Next).map(|e| e.sequence_weight.clone()))
        } else {
            None
        };

        // Calculate new weight using FractionalIndex methods
        let new_weight: FractionalIndex = match (after_weight, before_weight) {
            (Some(after), Some(before)) => {
                // Insert between two existing positions
                FractionalIndex::between(&after, &before)
            }
            (Some(after), None) => {
                // Insert after 'after_id'
                FractionalIndex::after_last(&after)
            }
            (None, Some(before)) => {
                // Insert before 'before_id' - find the predecessor and use between
                // For simplicity, we insert right before the before block
                FractionalIndex::between(&FractionalIndex::first(), &before)
            }
            (None, None) => FractionalIndex::first(), // Default starting weight
        };

        // Update edges
        // If after_id is provided, update its NEXT edge to point to block_id
        if let Some(aid) = after_id {
            let edges = self.db.edges().outgoing_from(&aid).await.map_err(|e| {
                ErrorData::internal_error(format!("Failed to get edges: {}", e), None)
            })?;

            // Find the NEXT edge from after_id
            if let Some(mut edge) = edges.into_iter().find(|e| e.link_type == LinkType::Next) {
                edge.to = block_id;
                self.db.edges().update(edge).await.map_err(|e| {
                    ErrorData::internal_error(format!("Failed to update edge: {}", e), None)
                })?;
            } else {
                // No NEXT edge exists, create one
                let new_edge = Edge::new(aid, block_id, LinkType::Next).with_weight(new_weight.clone());
                self.db.edges().create(new_edge).await.map_err(|e| {
                    ErrorData::internal_error(format!("Failed to create edge: {}", e), None)
                })?;
            }
        }

        // If before_id is provided, update block_id's NEXT edge to point to before_id
        if let Some(bid) = before_id {
            let edges = self.db.edges().outgoing_from(&block_id).await.map_err(|e| {
                ErrorData::internal_error(format!("Failed to get edges: {}", e), None)
            })?;

            if let Some(mut edge) = edges.into_iter().find(|e| e.link_type == LinkType::Next) {
                edge.to = bid;
                self.db.edges().update(edge).await.map_err(|e| {
                    ErrorData::internal_error(format!("Failed to update edge: {}", e), None)
                })?;
            } else {
                // No NEXT edge exists from block_id, create one
                let new_edge = Edge::new(block_id, bid, LinkType::Next).with_weight(new_weight.clone());
                self.db.edges().create(new_edge).await.map_err(|e| {
                    ErrorData::internal_error(format!("Failed to create edge: {}", e), None)
                })?;
            }
        }

        serde_json::to_string_pretty(&serde_json::json!({
            "block_id": block_id.to_string(),
            "after_id": after_id.map(|id| id.to_string()),
            "before_id": before_id.map(|id| id.to_string()),
            "sequence_weight": new_weight,
            "message": "Block reordered successfully"
        }))
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
    }

    /// Get section map from a root
    pub(crate) async fn get_section_map(&self, args: &Map<String, Value>) -> Result<String, ErrorData> {
        let root_id = Self::get_string(args, "root_id")?;

        let root_id = Ulid::from_string(&root_id)
            .map_err(|e| ErrorData::invalid_params(format!("Invalid ULID: {}", e), None))?;

        // Check if block exists
        let root_block = self.db.blocks().get(&root_id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to get block: {}", e), None)
        })?;

        if root_block.is_none() {
            return Err(ErrorData::invalid_params(format!("Block not found: {}", root_id), None));
        }

        // Traverse to get children
        let children = self.db.edges().outgoing_from(&root_id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to get children: {}", e), None)
        })?;

        let mut section_map = Vec::new();
        for edge in children {
            if let Some(block) = self.db.blocks().get(&edge.to).await.map_err(|e| {
                ErrorData::internal_error(format!("Failed to get block: {}", e), None)
            })? {
                section_map.push(serde_json::json!({
                    "id": block.id.to_string(),
                    "title": block.title,
                    "block_type": format!("{:?}", block.block_type).to_lowercase(),
                    "sequence_weight": edge.sequence_weight
                }));
            }
        }

        serde_json::to_string_pretty(&serde_json::json!({
            "root_id": root_id.to_string(),
            "sections": section_map,
            "count": section_map.len()
        }))
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
    }

    /// Detect gaps in a section
    pub(crate) async fn detect_gaps(&self, args: &Map<String, Value>) -> Result<String, ErrorData> {
        let section_id = Self::get_string(args, "section_id")?;

        let section_id = Ulid::from_string(&section_id)
            .map_err(|e| ErrorData::invalid_params(format!("Invalid ULID: {}", e), None))?;

        // Check if block exists
        let section_block = self.db.blocks().get(&section_id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to get block: {}", e), None)
        })?;

        if section_block.is_none() {
            return Err(ErrorData::invalid_params(format!("Block not found: {}", section_id), None));
        }

        // Get all blocks in this section by traversing outgoing edges
        let child_edges = self.db.edges().outgoing_from(&section_id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to get children: {}", e), None)
        })?;

        let mut blocks: Vec<Block> = Vec::new();
        for edge in child_edges {
            if let Some(block) = self.db.blocks().get(&edge.to).await.map_err(|e| {
                ErrorData::internal_error(format!("Failed to get block: {}", e), None)
            })? {
                blocks.push(block);
            }
        }

        // Run ghost detection on the blocks in this section
        let detected_ghosts = self.ghost_detector.detect_all(&blocks).await.map_err(|e| {
            ErrorData::internal_error(format!("Ghost detection failed: {}", e), None)
        })?;

        let ghost_results: Vec<Value> = detected_ghosts
            .into_iter()
            .map(|g| {
                serde_json::json!({
                    "id": g.id.to_string(),
                    "description": g.description,
                    "confidence": g.confidence,
                    "status": format!("{:?}", g.status).to_lowercase(),
                    "ai_rationale": g.ai_rationale,
                    "position_hint": {
                        "after": g.position_hint.after.map(|id| id.to_string()),
                        "before": g.position_hint.before.map(|id| id.to_string()),
                        "parent_section": g.position_hint.parent_section.map(|id| id.to_string())
                    }
                })
            })
            .collect();

        serde_json::to_string_pretty(&serde_json::json!({
            "section_id": section_id.to_string(),
            "detected_gaps": ghost_results,
            "count": ghost_results.len()
        }))
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
    }

    /// List ghost nodes (blocks with type Ghost)
    pub(crate) async fn list_ghosts(&self, args: &Map<String, Value>) -> Result<String, ErrorData> {
        let status = Self::get_opt_string(args, "status");
        let confidence_below = Self::get_opt_f32(args, "confidence_below");

        // Get all ghost blocks from the database
        let all_ghosts = self.db.blocks().list_by_type(BlockType::Ghost).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to list ghosts: {}", e), None)
        })?;

        let filtered: Vec<Value> = all_ghosts
            .into_iter()
            .filter(|b| {
                // Filter by status if provided (stored in metadata)
                if let Some(ref status_str) = status {
                    let block_status = b.metadata.get("status")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_lowercase())
                        .unwrap_or_default();
                    if block_status != status_str.to_lowercase() {
                        return false;
                    }
                }
                // Filter by confidence if provided
                if let Some(threshold) = confidence_below {
                    let confidence = b.metadata.get("ai_confidence")
                        .and_then(|v| v.as_f64())
                        .map(|f| f as f32)
                        .unwrap_or(0.0);
                    if confidence >= threshold {
                        return false;
                    }
                }
                true
            })
            .map(|b| {
                serde_json::json!({
                    "id": b.id.to_string(),
                    "title": b.title,
                    "content": b.content,
                    "ai_confidence": b.ai_confidence,
                    "status": b.metadata.get("status").and_then(|v| v.as_str()).unwrap_or("detected"),
                    "created_at": b.created_at
                })
            })
            .collect();

        serde_json::to_string_pretty(&serde_json::json!({
            "ghosts": filtered,
            "count": filtered.len()
        }))
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
    }

    /// Synthesize a document from a structure
    pub(crate) async fn synthesize(&self, args: &Map<String, Value>) -> Result<String, ErrorData> {
        let structure_id = Self::get_string(args, "structure_id")?;
        let template = Self::get_opt_string(args, "template");
        let _output_path = Self::get_opt_string(args, "output_path");

        let structure_id = Ulid::from_string(&structure_id)
            .map_err(|e| ErrorData::invalid_params(format!("Invalid ULID: {}", e), None))?;

        // Check if structure exists
        let structure = self.db.blocks().get(&structure_id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to get structure: {}", e), None)
        })?;

        if structure.is_none() {
            return Err(ErrorData::invalid_params(format!("Structure not found: {}", structure_id), None));
        }

        // Create synthesizer and run synthesis
        let synthesizer = Synthesizer::new(&self.db);
        let result = synthesizer.synthesize(&structure_id, OutputFormat::Markdown, template.as_deref()).await
            .map_err(|e| ErrorData::internal_error(format!("Synthesis failed: {}", e), None))?;

        // Convert result to JSON
        let content = String::from_utf8(result.content)
            .map_err(|e| ErrorData::internal_error(format!("Invalid UTF-8 in content: {}", e), None))?;

        serde_json::to_string_pretty(&serde_json::json!({
            "structure_id": structure_id.to_string(),
            "title": result.title,
            "format": "markdown",
            "blocks_used": result.blocks_used,
            "blocks_total": result.blocks_total,
            "content": content,
            "message": "Synthesis completed successfully"
        }))
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
    }

    /// Get table of contents for a structure
    pub(crate) async fn get_toc(&self, args: &Map<String, Value>) -> Result<String, ErrorData> {
        let structure_id = Self::get_string(args, "structure_id")?;

        let structure_id = Ulid::from_string(&structure_id)
            .map_err(|e| ErrorData::invalid_params(format!("Invalid ULID: {}", e), None))?;

        // Check if structure exists
        let structure = self.db.blocks().get(&structure_id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to get structure: {}", e), None)
        })?;

        if structure.is_none() {
            return Err(ErrorData::invalid_params(format!("Structure not found: {}", structure_id), None));
        }

        // Build TOC by traversing children
        let mut toc = Vec::new();
        let mut stack: Vec<(Ulid, usize)> = vec![(structure_id, 0)];

        while let Some((current_id, depth)) = stack.pop() {
            // Get children of current block
            let children = self.db.edges().outgoing_from(&current_id).await.map_err(|e| {
                ErrorData::internal_error(format!("Failed to get children: {}", e), None)
            })?;

            // Sort by sequence weight (FractionalIndex already implements Ord)
            let mut sorted_children: Vec<Edge> = children;
            sorted_children.sort_by(|a, b| a.sequence_weight.cmp(&b.sequence_weight));

            // Add children to stack in reverse order (so they're processed in correct order)
            for edge in sorted_children.into_iter().rev() {
                if let Some(block) = self.db.blocks().get(&edge.to).await.map_err(|e| {
                    ErrorData::internal_error(format!("Failed to get block: {}", e), None)
                })? {
                    toc.push(serde_json::json!({
                        "id": block.id.to_string(),
                        "title": block.title,
                        "level": depth + 1
                    }));
                    stack.push((block.id, depth + 1));
                }
            }
        }

        serde_json::to_string_pretty(&serde_json::json!({
            "structure_id": structure_id.to_string(),
            "toc": toc,
            "count": toc.len()
        }))
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
    }

    /// Stage a block for commit (git-like workflow)
    pub(crate) async fn stage_block(&self, args: &Map<String, Value>) -> Result<String, ErrorData> {
        let block_id = Self::get_string(args, "block_id")?;
        let block_id = Ulid::from_string(&block_id)
            .map_err(|e| ErrorData::invalid_params(format!("Invalid ULID: {}", e), None))?;

        // Verify block exists
        let block = self.db.blocks().get(&block_id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to get block: {}", e), None)
        })?;

        if block.is_none() {
            return Err(ErrorData::invalid_params(format!("Block not found: {}", block_id), None));
        }

        // Get the repo path (use current directory/.pkm)
        let root = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".pkm");
        let version_repo = VersionRepo::new(&root);
        version_repo.init().map_err(|e| {
            ErrorData::internal_error(format!("Failed to init repo: {}", e), None)
        })?;

        version_repo.stage(&block_id).map_err(|e| {
            ErrorData::internal_error(format!("Failed to stage block: {}", e), None)
        })?;

        serde_json::to_string_pretty(&serde_json::json!({
            "status": "staged",
            "block_id": block_id.to_string(),
            "message": format!("Block {} staged for commit", block_id)
        }))
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
    }

    /// Commit all staged changes with a message
    pub(crate) async fn commit_changes(&self, args: &Map<String, Value>) -> Result<String, ErrorData> {
        let message = Self::get_string(args, "message")?;

        // Get the repo path
        let root = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".pkm");
        let version_repo = VersionRepo::new(&root);
        version_repo.init().map_err(|e| {
            ErrorData::internal_error(format!("Failed to init repo: {}", e), None)
        })?;

        let author = AgentId::new("mcp-agent");
        let commit_id = version_repo.commit(&message, author).map_err(|e| {
            ErrorData::internal_error(format!("Failed to commit: {}", e), None)
        })?;

        serde_json::to_string_pretty(&serde_json::json!({
            "status": "committed",
            "commit_id": commit_id.to_string(),
            "message": message
        }))
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
    }

    /// Get current working set (staging area) status
    pub(crate) async fn get_working_set_status(&self) -> Result<String, ErrorData> {
        // Get the repo path
        let root = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".pkm");
        let version_repo = VersionRepo::new(&root);

        let ws = version_repo.get_working_set().map_err(|e| {
            ErrorData::internal_error(format!("Failed to get working set: {}", e), None)
        })?;

        serde_json::to_string_pretty(&serde_json::json!({
            "staged_blocks": ws.staged_blocks().len(),
            "staged_edges": ws.staged_edges().len(),
            "removed_blocks": ws.removed_blocks().len(),
            "is_empty": ws.is_empty()
        }))
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
    }

    /// Remove a block from the staging area (unstage)
    pub(crate) async fn unstage_block(&self, args: &Map<String, Value>) -> Result<String, ErrorData> {
        let block_id = Self::get_string(args, "block_id")?;
        let block_id = Ulid::from_string(&block_id)
            .map_err(|e| ErrorData::invalid_params(format!("Invalid ULID: {}", e), None))?;

        // Get the repo path
        let root = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".pkm");
        let version_repo = VersionRepo::new(&root);
        version_repo.init().map_err(|e| {
            ErrorData::internal_error(format!("Failed to init repo: {}", e), None)
        })?;

        version_repo.unstage(&block_id).map_err(|e| {
            ErrorData::internal_error(format!("Failed to unstage block: {}", e), None)
        })?;

        serde_json::to_string_pretty(&serde_json::json!({
            "status": "unstaged",
            "block_id": block_id.to_string(),
            "message": format!("Block {} removed from staging area", block_id)
        }))
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
    }

    /// Discard all staged changes (clear the working set)
    pub(crate) async fn discard_working_set(&self) -> Result<String, ErrorData> {
        // Get the repo path
        let root = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".pkm");
        let version_repo = VersionRepo::new(&root);
        version_repo.init().map_err(|e| {
            ErrorData::internal_error(format!("Failed to init repo: {}", e), None)
        })?;

        // Get current status before discarding
        let ws = version_repo.get_working_set().map_err(|e| {
            ErrorData::internal_error(format!("Failed to get working set: {}", e), None)
        })?;

        let staged_count = ws.staged_blocks().len() + ws.staged_edges().len();
        let removed_count = ws.removed_blocks().len() + ws.removed_edges().len();

        version_repo.discard_working_set().map_err(|e| {
            ErrorData::internal_error(format!("Failed to discard working set: {}", e), None)
        })?;

        serde_json::to_string_pretty(&serde_json::json!({
            "status": "discarded",
            "staged_items_discarded": staged_count,
            "removed_items_discarded": removed_count,
            "message": "All staged changes have been discarded"
        }))
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Parse a link type string into a LinkType enum
fn parse_link_type(s: &str) -> Result<LinkType, ErrorData> {
    match s.to_lowercase().as_str() {
        "extends" => Ok(LinkType::Extends),
        "refines" => Ok(LinkType::Refines),
        "contradicts" => Ok(LinkType::Contradicts),
        "questions" => Ok(LinkType::Questions),
        "supports" => Ok(LinkType::Supports),
        "references" => Ok(LinkType::References),
        "related" => Ok(LinkType::Related),
        "similar_to" => Ok(LinkType::SimilarTo),
        "section_of" => Ok(LinkType::SectionOf),
        "subsection_of" => Ok(LinkType::SubsectionOf),
        "ordered_child" => Ok(LinkType::OrderedChild),
        "next" => Ok(LinkType::Next),
        "next_sibling" => Ok(LinkType::NextSibling),
        "first_child" => Ok(LinkType::FirstChild),
        "contains" => Ok(LinkType::Contains),
        "parent" => Ok(LinkType::Parent),
        "ai_suggested" => Ok(LinkType::AiSuggested),
        _ => Err(ErrorData::invalid_params(format!("Unknown link type: {}", s), None)),
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::LinkType;

    #[test]
    fn test_parse_link_type_valid_types() {
        // Test all valid link types
        let valid_types = vec![
            ("extends", LinkType::Extends),
            ("refines", LinkType::Refines),
            ("contradicts", LinkType::Contradicts),
            ("questions", LinkType::Questions),
            ("supports", LinkType::Supports),
            ("references", LinkType::References),
            ("related", LinkType::Related),
            ("similar_to", LinkType::SimilarTo),
            ("section_of", LinkType::SectionOf),
            ("subsection_of", LinkType::SubsectionOf),
            ("ordered_child", LinkType::OrderedChild),
            ("next", LinkType::Next),
            ("next_sibling", LinkType::NextSibling),
            ("first_child", LinkType::FirstChild),
            ("contains", LinkType::Contains),
            ("parent", LinkType::Parent),
            ("ai_suggested", LinkType::AiSuggested),
        ];

        for (name, expected) in valid_types {
            let result = parse_link_type(name);
            assert!(result.is_ok(), "Failed to parse: {}", name);
            assert_eq!(result.unwrap(), expected);
        }
    }

    #[test]
    fn test_parse_link_type_invalid() {
        let result = parse_link_type("invalid_type");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Unknown link type"));
    }

    #[test]
    fn test_parse_link_type_case_insensitive() {
        let result = parse_link_type("RELATED");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), LinkType::Related);

        let result2 = parse_link_type("Related");
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), LinkType::Related);

        let result3 = parse_link_type("ReLaTeD");
        assert!(result3.is_ok());
        assert_eq!(result3.unwrap(), LinkType::Related);
    }

    #[test]
    fn test_parse_link_type_all_variants() {
        // Test various case variations for a few link types
        assert_eq!(parse_link_type("EXTENDS").unwrap(), LinkType::Extends);
        assert_eq!(parse_link_type("Extends").unwrap(), LinkType::Extends);
        assert_eq!(parse_link_type("extends").unwrap(), LinkType::Extends);

        assert_eq!(parse_link_type("SECTION_OF").unwrap(), LinkType::SectionOf);
        assert_eq!(parse_link_type("Section_Of").unwrap(), LinkType::SectionOf);
        assert_eq!(parse_link_type("section_of").unwrap(), LinkType::SectionOf);

        assert_eq!(parse_link_type("AI_SUGGESTED").unwrap(), LinkType::AiSuggested);
        assert_eq!(parse_link_type("Ai_Suggested").unwrap(), LinkType::AiSuggested);
        assert_eq!(parse_link_type("ai_suggested").unwrap(), LinkType::AiSuggested);
    }

    #[test]
    fn test_get_string_helper_valid() {
        use serde_json::Map;
        use serde_json::Value;

        let mut args = Map::new();
        args.insert("name".to_string(), Value::String("test".to_string()));
        args.insert("count".to_string(), Value::Number(42.into()));

        let result = NexusMcpServer::get_string(&args, "name");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test");
    }

    #[test]
    fn test_get_string_helper_missing() {
        use serde_json::Map;

        let args = Map::new();

        let result = NexusMcpServer::get_string(&args, "missing");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Missing or invalid parameter: missing"));
    }

    #[test]
    fn test_get_string_helper_wrong_type() {
        use serde_json::Map;
        use serde_json::Value;

        let mut args = Map::new();
        args.insert("number".to_string(), Value::Number(42.into()));

        let result = NexusMcpServer::get_string(&args, "number");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_opt_string_helper_some() {
        use serde_json::Map;
        use serde_json::Value;

        let mut args = Map::new();
        args.insert("name".to_string(), Value::String("test".to_string()));

        let result = NexusMcpServer::get_opt_string(&args, "name");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "test");
    }

    #[test]
    fn test_get_opt_string_helper_none() {
        use serde_json::Map;

        let args = Map::new();

        let result = NexusMcpServer::get_opt_string(&args, "missing");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_f32_helper_valid() {
        use serde_json::Map;
        use serde_json::Value;

        let mut args = Map::new();
        args.insert("value".to_string(), Value::Number(serde_json::Number::from_f64(3.14).unwrap()));

        let result = NexusMcpServer::get_f32(&args, "value");
        assert!(result.is_ok());
        assert!((result.unwrap() - 3.14).abs() < 0.001);
    }

    #[test]
    fn test_get_f32_helper_missing() {
        use serde_json::Map;

        let args = Map::new();

        let result = NexusMcpServer::get_f32(&args, "missing");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_i64_helper_valid() {
        use serde_json::Map;
        use serde_json::Value;

        let mut args = Map::new();
        args.insert("count".to_string(), Value::Number(42.into()));

        let result = NexusMcpServer::get_i64(&args, "count");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_get_bool_helper_true() {
        use serde_json::Map;
        use serde_json::Value;

        let mut args = Map::new();
        args.insert("flag".to_string(), Value::Bool(true));

        let result = NexusMcpServer::get_bool(&args, "flag");
        assert!(result);
    }

    #[test]
    fn test_get_bool_helper_false_default() {
        use serde_json::Map;
        use serde_json::Value;

        let mut args = Map::new();
        args.insert("flag".to_string(), Value::Bool(false));

        let result = NexusMcpServer::get_bool(&args, "flag");
        assert!(!result);
    }

    #[test]
    fn test_get_bool_helper_missing_defaults_false() {
        use serde_json::Map;

        let args = Map::new();

        let result = NexusMcpServer::get_bool(&args, "missing");
        assert!(!result);
    }

    #[test]
    fn test_get_bool_helper_wrong_type_defaults_false() {
        use serde_json::Map;
        use serde_json::Value;

        let mut args = Map::new();
        args.insert("flag".to_string(), Value::String("true".to_string()));

        let result = NexusMcpServer::get_bool(&args, "flag");
        assert!(!result);
    }
}

// =============================================================================
// Integration Tests with Database
// =============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::db::Database;

    /// Helper to create argument map
    fn args(pairs: Vec<(&str, Value)>) -> Map<String, Value> {
        pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
    }

    /// Helper to create string argument
    fn s<S: AsRef<str>>(s: S) -> Value {
        Value::String(s.as_ref().to_string())
    }

    /// Helper to create number argument
    fn n(n: i64) -> Value {
        Value::Number(n.into())
    }

    /// Helper to create bool argument
    fn b(b: bool) -> Value {
        Value::Bool(b)
    }

    /// Helper to create array argument
    fn arr(items: Vec<Value>) -> Value {
        Value::Array(items)
    }

    // =============================================================================
    // Block Tool Integration Tests
    // =============================================================================

    #[tokio::test]
    async fn test_create_block_fleeting() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let arguments = args(vec![
            ("block_type", s("fleeting")),
            ("content", s("My fleeting note")),
            ("title", s("Test Note")),
            ("tags", arr(vec![s("test"), s("fleeting")])),
        ]);

        let result = server.create_block(&arguments).await;
        assert!(result.is_ok(), "Failed to create block: {:?}", result.err());
        let json = result.unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();

        assert!(value.get("id").is_some());
        assert_eq!(value.get("block_type").unwrap(), "fleeting");
        assert_eq!(value.get("title").unwrap(), "Test Note");
    }

    #[tokio::test]
    async fn test_create_block_permanent() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let arguments = args(vec![
            ("block_type", s("permanent")),
            ("content", s("# Atomic Note\n\nThis is atomic.")),
            ("title", s("My Zettel")),
        ]);

        let result = server.create_block(&arguments).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value.get("block_type").unwrap(), "permanent");
    }

    #[tokio::test]
    async fn test_create_block_invalid_type() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let arguments = args(vec![
            ("block_type", s("invalid_type")),
        ]);

        let result = server.create_block(&arguments).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Unknown block type"));
    }

    #[tokio::test]
    async fn test_create_block_without_title() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let arguments = args(vec![
            ("block_type", s("permanent")),
        ]);

        let result = server.create_block(&arguments).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value.get("title").unwrap(), "Untitled");
    }

    #[tokio::test]
    async fn test_get_block() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        // Create a block first
        let create_args = args(vec![
            ("block_type", s("permanent")),
            ("content", s("Test content")),
            ("title", s("Test Block")),
            ("tags", arr(vec![s("test")])),
        ]);
        let create_result = server.create_block(&create_args).await.unwrap();
        let create_value: Value = serde_json::from_str(&create_result).unwrap();
        let block_id = create_value.get("id").unwrap().as_str().unwrap();

        // Get the block
        let get_args = args(vec![
            ("block_id", s(block_id)),
            ("include_content", b(true)),
        ]);
        let get_result = server.get_block(&get_args).await;
        assert!(get_result.is_ok(), "Failed to get block: {:?}", get_result.err());
        let json = get_result.unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value.get("id").unwrap().as_str().unwrap(), block_id);
        assert_eq!(value.get("title").unwrap(), "Test Block");
        assert_eq!(value.get("content").unwrap(), "Test content");
    }

    #[tokio::test]
    async fn test_get_block_invalid_ulid() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let get_args = args(vec![
            ("block_id", s("invalid-ulid")),
        ]);
        let get_result = server.get_block(&get_args).await;
        assert!(get_result.is_err());
        let err = get_result.unwrap_err();
        assert!(err.message.contains("Invalid ULID"));
    }

    #[tokio::test]
    async fn test_get_block_not_found() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let get_args = args(vec![
            ("block_id", s("01ARZ3NDEKTSV4RRFFQ69G5FAV")),
        ]);
        let get_result = server.get_block(&get_args).await;
        assert!(get_result.is_err());
        let err = get_result.unwrap_err();
        assert!(err.message.contains("Block not found"));
    }

    #[tokio::test]
    async fn test_search_blocks_by_query() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        for i in 0..5 {
            let create_args = args(vec![
                ("block_type", s("permanent")),
                ("content", s(format!("Test content number {}", i))),
                ("title", s(format!("Block {}", i))),
            ]);
            server.create_block(&create_args).await.unwrap();
        }

        let search_args = args(vec![
            ("query", s("Test content")),
            ("limit", n(10)),
        ]);
        let result = server.search_blocks(&search_args).await;
        assert!(result.is_ok(), "Search failed: {:?}", result.err());
        let json = result.unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();

        assert!(value.get("blocks").is_some());
        assert!(value.get("count").is_some());
    }

    #[tokio::test]
    async fn test_search_blocks_by_type() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let fleeting_args = args(vec![
            ("block_type", s("fleeting")),
            ("title", s("Fleeting Note")),
        ]);
        server.create_block(&fleeting_args).await.unwrap();

        let permanent_args = args(vec![
            ("block_type", s("permanent")),
            ("title", s("Permanent Note")),
        ]);
        server.create_block(&permanent_args).await.unwrap();

        let search_args = args(vec![
            ("block_type", s("fleeting")),
        ]);
        let result = server.search_blocks(&search_args).await.unwrap();
        let value: Value = serde_json::from_str(&result).unwrap();

        let blocks = value.get("blocks").unwrap().as_array().unwrap();
        for block in blocks {
            assert_eq!(block.get("block_type").unwrap(), "fleeting");
        }
    }

    #[tokio::test]
    async fn test_search_blocks_all_types_valid() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let block_types = vec![
            "fleeting", "literature", "permanent", "structure",
            "hub", "task", "reference", "outline", "ghost",
        ];

        for bt in block_types {
            let create_args = args(vec![
                ("block_type", s(bt)),
                ("title", s(format!("{} block", bt))),
            ]);
            let result = server.create_block(&create_args).await;
            assert!(result.is_ok(), "Failed to create {} block: {:?}", bt, result.err());
        }
    }

    #[tokio::test]
    async fn test_update_block_content() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let create_args = args(vec![
            ("block_type", s("permanent")),
            ("title", s("Original Title")),
            ("content", s("Original content")),
        ]);
        let create_result = server.create_block(&create_args).await.unwrap();
        let create_value: Value = serde_json::from_str(&create_result).unwrap();
        let block_id = create_value.get("id").unwrap().as_str().unwrap();

        let update_args = args(vec![
            ("block_id", s(block_id)),
            ("content", s("Updated content")),
        ]);
        let update_result = server.update_block(&update_args).await;
        assert!(update_result.is_ok(), "Update failed: {:?}", update_result.err());

        let get_args = args(vec![
            ("block_id", s(block_id)),
            ("include_content", b(true)),
        ]);
        let get_result = server.get_block(&get_args).await.unwrap();
        let get_value: Value = serde_json::from_str(&get_result).unwrap();
        assert_eq!(get_value.get("content").unwrap(), "Updated content");
    }

    // =============================================================================
    // Link Tool Integration Tests
    // =============================================================================

    #[tokio::test]
    async fn test_create_link() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let block1_args = args(vec![
            ("block_type", s("permanent")),
            ("title", s("Block 1")),
        ]);
        let block1_result = server.create_block(&block1_args).await.unwrap();
        let block1_id: Value = serde_json::from_str(&block1_result).unwrap();
        let block1_id = block1_id.get("id").unwrap().as_str().unwrap();

        let block2_args = args(vec![
            ("block_type", s("permanent")),
            ("title", s("Block 2")),
        ]);
        let block2_result = server.create_block(&block2_args).await.unwrap();
        let block2_id: Value = serde_json::from_str(&block2_result).unwrap();
        let block2_id = block2_id.get("id").unwrap().as_str().unwrap();

        let link_args = args(vec![
            ("source_id", s(block1_id)),
            ("target_id", s(block2_id)),
            ("link_type", s("supports")),
        ]);
        let link_result = server.create_link(&link_args).await;
        assert!(link_result.is_ok(), "Failed to create link: {:?}", link_result.err());
        let json = link_result.unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();

        assert!(value.get("id").is_some());
        assert_eq!(value.get("source_id").unwrap().as_str().unwrap(), block1_id);
        assert_eq!(value.get("target_id").unwrap().as_str().unwrap(), block2_id);
        assert_eq!(value.get("link_type").unwrap(), "supports");
    }

    #[tokio::test]
    async fn test_create_link_invalid_link_type() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let block1_args = args(vec![
            ("block_type", s("permanent")),
            ("title", s("Block 1")),
        ]);
        let block1_result = server.create_block(&block1_args).await.unwrap();
        let block1_id: Value = serde_json::from_str(&block1_result).unwrap();
        let block1_id = block1_id.get("id").unwrap().as_str().unwrap();

        let block2_args = args(vec![
            ("block_type", s("permanent")),
            ("title", s("Block 2")),
        ]);
        let block2_result = server.create_block(&block2_args).await.unwrap();
        let block2_id: Value = serde_json::from_str(&block2_result).unwrap();
        let block2_id = block2_id.get("id").unwrap().as_str().unwrap();

        let link_args = args(vec![
            ("source_id", s(block1_id)),
            ("target_id", s(block2_id)),
            ("link_type", s("invalid_link_type")),
        ]);
        let link_result = server.create_link(&link_args).await;
        assert!(link_result.is_err());
        let err = link_result.unwrap_err();
        assert!(err.message.contains("Unknown link type"));
    }

    #[tokio::test]
    async fn test_get_links_outgoing() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let block1_args = args(vec![("block_type", s("permanent")), ("title", s("Source"))]);
        let block1_result = server.create_block(&block1_args).await.unwrap();
        let block1_id: Value = serde_json::from_str(&block1_result).unwrap();
        let block1_id = block1_id.get("id").unwrap().as_str().unwrap();

        let block2_args = args(vec![("block_type", s("permanent")), ("title", s("Target"))]);
        let block2_result = server.create_block(&block2_args).await.unwrap();
        let block2_id: Value = serde_json::from_str(&block2_result).unwrap();
        let block2_id = block2_id.get("id").unwrap().as_str().unwrap();

        let link_args = args(vec![
            ("source_id", s(block1_id)),
            ("target_id", s(block2_id)),
            ("link_type", s("references")),
        ]);
        server.create_link(&link_args).await.unwrap();

        let get_args = args(vec![
            ("block_id", s(block1_id)),
            ("direction", s("outgoing")),
        ]);
        let result = server.get_links(&get_args).await.unwrap();
        let value: Value = serde_json::from_str(&result).unwrap();

        let edges = value.get("edges").unwrap().as_array().unwrap();
        assert!(!edges.is_empty());
        assert_eq!(edges[0].get("from").unwrap().as_str().unwrap(), block1_id);
    }

    #[tokio::test]
    async fn test_get_links_incoming() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let block1_args = args(vec![("block_type", s("permanent")), ("title", s("Source"))]);
        let block1_result = server.create_block(&block1_args).await.unwrap();
        let block1_id: Value = serde_json::from_str(&block1_result).unwrap();
        let block1_id = block1_id.get("id").unwrap().as_str().unwrap();

        let block2_args = args(vec![("block_type", s("permanent")), ("title", s("Target"))]);
        let block2_result = server.create_block(&block2_args).await.unwrap();
        let block2_id: Value = serde_json::from_str(&block2_result).unwrap();
        let block2_id = block2_id.get("id").unwrap().as_str().unwrap();

        let link_args = args(vec![
            ("source_id", s(block1_id)),
            ("target_id", s(block2_id)),
            ("link_type", s("supports")),
        ]);
        server.create_link(&link_args).await.unwrap();

        let get_args = args(vec![
            ("block_id", s(block2_id)),
            ("direction", s("incoming")),
        ]);
        let result = server.get_links(&get_args).await.unwrap();
        let value: Value = serde_json::from_str(&result).unwrap();

        let edges = value.get("edges").unwrap().as_array().unwrap();
        assert!(!edges.is_empty());
        assert_eq!(edges[0].get("to").unwrap().as_str().unwrap(), block2_id);
    }

    #[tokio::test]
    async fn test_get_links_filter_by_type() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let block1_args = args(vec![("block_type", s("permanent")), ("title", s("Block 1"))]);
        let block1_result = server.create_block(&block1_args).await.unwrap();
        let block1_id: Value = serde_json::from_str(&block1_result).unwrap();
        let block1_id = block1_id.get("id").unwrap().as_str().unwrap();

        let block2_args = args(vec![("block_type", s("permanent")), ("title", s("Block 2"))]);
        let block2_result = server.create_block(&block2_args).await.unwrap();
        let block2_id: Value = serde_json::from_str(&block2_result).unwrap();
        let block2_id = block2_id.get("id").unwrap().as_str().unwrap();

        let block3_args = args(vec![("block_type", s("permanent")), ("title", s("Block 3"))]);
        let block3_result = server.create_block(&block3_args).await.unwrap();
        let block3_id: Value = serde_json::from_str(&block3_result).unwrap();
        let block3_id = block3_id.get("id").unwrap().as_str().unwrap();

        server.create_link(&args(vec![
            ("source_id", s(block1_id)),
            ("target_id", s(block2_id)),
            ("link_type", s("supports")),
        ])).await.unwrap();

        server.create_link(&args(vec![
            ("source_id", s(block1_id)),
            ("target_id", s(block3_id)),
            ("link_type", s("extends")),
        ])).await.unwrap();

        let get_args = args(vec![
            ("block_id", s(block1_id)),
            ("link_types", arr(vec![s("supports")])),
            ("direction", s("outgoing")),
        ]);
        let result = server.get_links(&get_args).await.unwrap();
        let value: Value = serde_json::from_str(&result).unwrap();

        let edges = value.get("edges").unwrap().as_array().unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].get("link_type").unwrap(), "supports");
    }

    // =============================================================================
    // Spine Tool Integration Tests
    // =============================================================================

    #[tokio::test]
    async fn test_traverse_spine() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let structure_args = args(vec![
            ("block_type", s("structure")),
            ("title", s("Root Structure")),
        ]);
        let structure_result = server.create_block(&structure_args).await.unwrap();
        let structure_id: Value = serde_json::from_str(&structure_result).unwrap();
        let structure_id = structure_id.get("id").unwrap().as_str().unwrap();

        let child1_args = args(vec![
            ("block_type", s("permanent")),
            ("title", s("Child 1")),
        ]);
        let child1_result = server.create_block(&child1_args).await.unwrap();
        let child1_id: Value = serde_json::from_str(&child1_result).unwrap();
        let child1_id = child1_id.get("id").unwrap().as_str().unwrap();

        let child2_args = args(vec![
            ("block_type", s("permanent")),
            ("title", s("Child 2")),
        ]);
        let child2_result = server.create_block(&child2_args).await.unwrap();
        let child2_id: Value = serde_json::from_str(&child2_result).unwrap();
        let child2_id = child2_id.get("id").unwrap().as_str().unwrap();

        server.create_link(&args(vec![
            ("source_id", s(structure_id)),
            ("target_id", s(child1_id)),
            ("link_type", s("contains")),
        ])).await.unwrap();

        server.create_link(&args(vec![
            ("source_id", s(structure_id)),
            ("target_id", s(child2_id)),
            ("link_type", s("contains")),
        ])).await.unwrap();

        let traverse_args = args(vec![
            ("root_id", s(structure_id)),
            ("max_depth", n(2)),
        ]);
        let result = server.traverse_spine(&traverse_args).await;
        assert!(result.is_ok(), "Traverse failed: {:?}", result.err());
        let json = result.unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();

        assert!(value.get("blocks").is_some());
    }

    #[tokio::test]
    async fn test_traverse_spine_invalid_ulid() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let traverse_args = args(vec![
            ("root_id", s("invalid-ulid")),
        ]);
        let result = server.traverse_spine(&traverse_args).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Invalid"));
    }

    #[tokio::test]
    async fn test_gravity_check() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let block_args = args(vec![
            ("block_type", s("permanent")),
            ("title", s("Connected Block")),
        ]);
        let block_result = server.create_block(&block_args).await.unwrap();
        let block_id: Value = serde_json::from_str(&block_result).unwrap();
        let block_id = block_id.get("id").unwrap().as_str().unwrap();

        let other_args = args(vec![
            ("block_type", s("permanent")),
            ("title", s("Other Block")),
        ]);
        let other_result = server.create_block(&other_args).await.unwrap();
        let other_id: Value = serde_json::from_str(&other_result).unwrap();
        let other_id = other_id.get("id").unwrap().as_str().unwrap();

        server.create_link(&args(vec![
            ("source_id", s(block_id)),
            ("target_id", s(other_id)),
            ("link_type", s("supports")),
        ])).await.unwrap();

        let gravity_args = args(vec![
            ("block_id", s(block_id)),
        ]);
        let result = server.gravity_check(&gravity_args).await;
        assert!(result.is_ok(), "Gravity check failed: {:?}", result.err());
        let json = result.unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();

        assert!(value.get("gravity_score").is_some());
        assert!(value.get("outgoing_links").is_some());
        assert!(value.get("incoming_links").is_some());
    }

    #[tokio::test]
    async fn test_gravity_check_block_not_found() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let gravity_args = args(vec![
            ("block_id", s("01ARZ3NDEKTSV4RRFFQ69G5FAV")),
        ]);
        let result = server.gravity_check(&gravity_args).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Block not found"));
    }

    #[tokio::test]
    async fn test_reorder_block() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let block1_args = args(vec![("block_type", s("permanent")), ("title", s("Block 1"))]);
        let block1_result = server.create_block(&block1_args).await.unwrap();
        let block1_id: Value = serde_json::from_str(&block1_result).unwrap();
        let block1_id = block1_id.get("id").unwrap().as_str().unwrap();

        let block2_args = args(vec![("block_type", s("permanent")), ("title", s("Block 2"))]);
        let block2_result = server.create_block(&block2_args).await.unwrap();
        let block2_id: Value = serde_json::from_str(&block2_result).unwrap();
        let block2_id = block2_id.get("id").unwrap().as_str().unwrap();

        let block3_args = args(vec![("block_type", s("permanent")), ("title", s("Block 3"))]);
        let block3_result = server.create_block(&block3_args).await.unwrap();
        let block3_id: Value = serde_json::from_str(&block3_result).unwrap();
        let block3_id = block3_id.get("id").unwrap().as_str().unwrap();

        server.create_link(&args(vec![
            ("source_id", s(block1_id)),
            ("target_id", s(block2_id)),
            ("link_type", s("next")),
        ])).await.unwrap();

        let reorder_args = args(vec![
            ("block_id", s(block3_id)),
            ("after_id", s(block1_id)),
            ("before_id", s(block2_id)),
        ]);
        let result = server.reorder_block(&reorder_args).await;
        assert!(result.is_ok(), "Reorder failed: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_reorder_block_requires_after_or_before() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let block_args = args(vec![("block_type", s("permanent")), ("title", s("Block"))]);
        let block_result = server.create_block(&block_args).await.unwrap();
        let block_id: Value = serde_json::from_str(&block_result).unwrap();
        let block_id = block_id.get("id").unwrap().as_str().unwrap();

        let reorder_args = args(vec![
            ("block_id", s(block_id)),
        ]);
        let result = server.reorder_block(&reorder_args).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("after_id"));
    }

    // =============================================================================
    // Structure Tool Integration Tests
    // =============================================================================

    #[tokio::test]
    async fn test_get_section_map() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let structure_args = args(vec![
            ("block_type", s("structure")),
            ("title", s("My Structure")),
        ]);
        let structure_result = server.create_block(&structure_args).await.unwrap();
        let structure_id: Value = serde_json::from_str(&structure_result).unwrap();
        let structure_id = structure_id.get("id").unwrap().as_str().unwrap();

        let child_args = args(vec![
            ("block_type", s("permanent")),
            ("title", s("Section 1")),
        ]);
        let child_result = server.create_block(&child_args).await.unwrap();
        let child_id: Value = serde_json::from_str(&child_result).unwrap();
        let child_id = child_id.get("id").unwrap().as_str().unwrap();

        server.create_link(&args(vec![
            ("source_id", s(structure_id)),
            ("target_id", s(child_id)),
            ("link_type", s("contains")),
        ])).await.unwrap();

        let section_args = args(vec![
            ("root_id", s(structure_id)),
        ]);
        let result = server.get_section_map(&section_args).await;
        assert!(result.is_ok(), "Get section map failed: {:?}", result.err());
        let json = result.unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();

        assert!(value.get("sections").is_some());
    }

    #[tokio::test]
    async fn test_get_section_map_not_found() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let section_args = args(vec![
            ("root_id", s("01ARZ3NDEKTSV4RRFFQ69G5FAV")),
        ]);
        let result = server.get_section_map(&section_args).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Block not found"));
    }

    #[tokio::test]
    async fn test_detect_gaps() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let structure_args = args(vec![
            ("block_type", s("structure")),
            ("title", s("Test Structure")),
        ]);
        let structure_result = server.create_block(&structure_args).await.unwrap();
        let structure_id: Value = serde_json::from_str(&structure_result).unwrap();
        let structure_id = structure_id.get("id").unwrap().as_str().unwrap();

        let child1_args = args(vec![
            ("block_type", s("permanent")),
            ("title", s("Section 1")),
            ("content", s("Content for section 1")),
        ]);
        let child1_result = server.create_block(&child1_args).await.unwrap();
        let child1_id: Value = serde_json::from_str(&child1_result).unwrap();
        let child1_id = child1_id.get("id").unwrap().as_str().unwrap();

        let child2_args = args(vec![
            ("block_type", s("permanent")),
            ("title", s("Section 3")),
            ("content", s("Content for section 3")),
        ]);
        let child2_result = server.create_block(&child2_args).await.unwrap();
        let child2_id: Value = serde_json::from_str(&child2_result).unwrap();
        let child2_id = child2_id.get("id").unwrap().as_str().unwrap();

        server.create_link(&args(vec![
            ("source_id", s(structure_id)),
            ("target_id", s(child1_id)),
            ("link_type", s("contains")),
        ])).await.unwrap();

        server.create_link(&args(vec![
            ("source_id", s(structure_id)),
            ("target_id", s(child2_id)),
            ("link_type", s("contains")),
        ])).await.unwrap();

        let gap_args = args(vec![
            ("section_id", s(structure_id)),
        ]);
        let result = server.detect_gaps(&gap_args).await;
        assert!(result.is_ok(), "Detect gaps failed: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_list_ghosts() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let ghost_args = args(vec![
            ("block_type", s("ghost")),
            ("title", s("Missing Section")),
        ]);
        server.create_block(&ghost_args).await.unwrap();

        let list_args = args(vec![]);
        let result = server.list_ghosts(&list_args).await;
        assert!(result.is_ok(), "List ghosts failed: {:?}", result.err());
        let json = result.unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();

        assert!(value.get("ghosts").is_some());
    }

    // =============================================================================
    // Synthesis Tool Integration Tests
    // =============================================================================

    #[tokio::test]
    async fn test_synthesize() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let structure_args = args(vec![
            ("block_type", s("structure")),
            ("title", s("My Document")),
        ]);
        let structure_result = server.create_block(&structure_args).await.unwrap();
        let structure_id: Value = serde_json::from_str(&structure_result).unwrap();
        let structure_id = structure_id.get("id").unwrap().as_str().unwrap();

        let synthesize_args = args(vec![
            ("structure_id", s(structure_id)),
        ]);
        let result = server.synthesize(&synthesize_args).await;
        assert!(result.is_ok(), "Synthesize failed: {:?}", result.err());
        let json = result.unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();

        assert!(value.get("structure_id").is_some());
        assert!(value.get("content").is_some());
    }

    #[tokio::test]
    async fn test_synthesize_not_found() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let synthesize_args = args(vec![
            ("structure_id", s("01ARZ3NDEKTSV4RRFFQ69G5FAV")),
        ]);
        let result = server.synthesize(&synthesize_args).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Structure not found"));
    }

    #[tokio::test]
    async fn test_get_toc() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let structure_args = args(vec![
            ("block_type", s("structure")),
            ("title", s("Root")),
        ]);
        let structure_result = server.create_block(&structure_args).await.unwrap();
        let structure_id: Value = serde_json::from_str(&structure_result).unwrap();
        let structure_id = structure_id.get("id").unwrap().as_str().unwrap();

        let child1_args = args(vec![
            ("block_type", s("permanent")),
            ("title", s("Chapter 1")),
        ]);
        let child1_result = server.create_block(&child1_args).await.unwrap();
        let child1_id: Value = serde_json::from_str(&child1_result).unwrap();
        let child1_id = child1_id.get("id").unwrap().as_str().unwrap();

        let child2_args = args(vec![
            ("block_type", s("permanent")),
            ("title", s("Chapter 2")),
        ]);
        let child2_result = server.create_block(&child2_args).await.unwrap();
        let child2_id: Value = serde_json::from_str(&child2_result).unwrap();
        let child2_id = child2_id.get("id").unwrap().as_str().unwrap();

        server.create_link(&args(vec![
            ("source_id", s(structure_id)),
            ("target_id", s(child1_id)),
            ("link_type", s("contains")),
        ])).await.unwrap();

        server.create_link(&args(vec![
            ("source_id", s(structure_id)),
            ("target_id", s(child2_id)),
            ("link_type", s("contains")),
        ])).await.unwrap();

        let toc_args = args(vec![
            ("structure_id", s(structure_id)),
        ]);
        let result = server.get_toc(&toc_args).await;
        assert!(result.is_ok(), "Get TOC failed: {:?}", result.err());
        let json = result.unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();

        assert!(value.get("toc").is_some());
    }

    #[tokio::test]
    async fn test_get_toc_not_found() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let toc_args = args(vec![
            ("structure_id", s("01ARZ3NDEKTSV4RRFFQ69G5FAV")),
        ]);
        let result = server.get_toc(&toc_args).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Structure not found"));
    }

    // =============================================================================
    // Edge Cases and Error Handling
    // =============================================================================

    #[tokio::test]
    async fn test_link_suggestions_empty_database() {
        let db = Database::in_memory().await.unwrap();
        let server = NexusMcpServer::new(db);

        let block_args = args(vec![
            ("block_type", s("permanent")),
            ("title", s("Lone Block")),
        ]);
        let block_result = server.create_block(&block_args).await.unwrap();
        let block_id: Value = serde_json::from_str(&block_result).unwrap();
        let block_id = block_id.get("id").unwrap().as_str().unwrap();

        let suggest_args = args(vec![
            ("block_id", s(block_id)),
            ("limit", n(5)),
        ]);
        let result = server.suggest_links(&suggest_args).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();

        let suggestions = value.get("suggestions").unwrap().as_array().unwrap();
        assert!(suggestions.is_empty());
    }
}