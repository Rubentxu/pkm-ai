//! CLI module

mod commands;
mod app;
mod config;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "nexus",
    author,
    version,
    about = "Nexus-Grafo: High-performance PKM with Zettelkasten + SurrealDB + AI",
    long_about = "A Personal Knowledge Management system treating ORDER and STRUCTURE as first-class citizens.\n\nFeatures:\n  • Block-Atom Model with ULID\n  • Structural Spine (Folgezettel digital)\n  • Document Synthesis as Priority #1\n  • AI as active weaver of the knowledge graph"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Database path (defaults to ~/.pkmai/data.db)
    #[arg(short, long, global = true, env = "NEXUS_DB_PATH")]
    pub db_path: Option<String>,

    /// Enable verbose output (show info logs)
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Auto-stage changes after create/link operations (default: true)
    #[arg(long, global = true)]
    pub stage: bool,

    /// Disable auto-staging after create/link operations
    #[arg(long, global = true)]
    pub no_stage: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialize PKM-AI (creates .pkmai/config.toml)
    Init {
        /// Initialize in home directory (~/.pkmai/) instead of current directory
        #[arg(long)]
        home: bool,

        /// Force overwrite existing config
        #[arg(long)]
        force: bool,
    },

    /// Create a new block
    ///
    /// Examples:
    ///   pkmai create -t permanent --title "My Note" --content "Note content"
    ///   pkmai create -t task --title "TODO: Fix bug" -T rust,bug
    ///   pkmai create --title "Quick idea" -i  # interactive mode with AI suggestions
    #[command(alias = "c")]
    Create {
        /// Block type: fleeting (f), literature (l), permanent (p), structure (s), hub (h), task (t), reference (r), outline (o)
        #[arg(short = 't', long)]
        block_type: String,

        /// Title of the block
        #[arg(long)]
        title: String,

        /// Content (reads from stdin if not provided)
        #[arg(long)]
        content: Option<String>,

        /// Tags (comma-separated)
        #[arg(short = 'T', long)]
        tags: Option<String>,

        /// Enable interactive AI pre-flight mode with suggestions
        #[arg(short = 'i', long)]
        interactive: bool,
    },

    /// Quick capture: create + stage + commit in one command
    ///
    /// Examples:
    ///   pkmai quick "My quick note idea"
    ///   pkmai quick "Important task" -t task -T work,urgent
    ///   pkmai q "Reference to article" -t reference
    #[command(alias = "q")]
    Quick {
        /// Content of the note
        content: String,

        /// Block type (default: fleeting). Types: fleeting, literature, permanent, task, reference
        #[arg(short = 't', long)]
        block_type: Option<String>,

        /// Tags (comma-separated)
        #[arg(short = 'T', long)]
        tags: Option<String>,
    },

    /// List blocks
    ///
    /// Examples:
    ///   pkmai list                    # List all blocks (table format)
    ///   pkmai list -t permanent       # List only permanent blocks
    ///   pkmai list -T rust,python     # List blocks tagged with rust OR python
    ///   pkmai list -s ownership        # Fuzzy search "ownership" in titles
    ///   pkmai list -n 10 -o json      # List 10 blocks as JSON
    #[command(alias = "ls")]
    List {
        /// Filter by block type (fleeting, literature, permanent, structure, hub, task, reference, outline)
        #[arg(short = 't', long)]
        block_type: Option<String>,

        /// Filter by tags (comma-separated, OR logic)
        #[arg(short = 'T', long)]
        tags: Option<String>,

        /// Fuzzy search by title
        #[arg(short = 's', long)]
        search: Option<String>,

        /// Limit number of results (default: 50)
        #[arg(short = 'n', long, default_value = "50")]
        limit: usize,

        /// Output format: table, json, simple
        #[arg(short = 'o', long, default_value = "table")]
        output: String,
    },

    /// Fuzzy search blocks by title
    #[command(alias = "f")]
    Search {
        /// Search query (supports fuzzy matching)
        query: String,

        /// Limit number of results
        #[arg(short = 'n', long, default_value = "50")]
        limit: usize,
    },

    /// Search block content
    #[command(alias = "g")]
    Grep {
        /// Search pattern (regex)
        pattern: String,

        /// Search in content only (not titles)
        #[arg(short = 'c', long)]
        content_only: bool,

        /// Case sensitive search
        #[arg(short = 'i', long)]
        case_sensitive: bool,

        /// Limit number of results
        #[arg(short = 'n', long, default_value = "50")]
        limit: usize,
    },

    /// Show a block
    #[command(alias = "s")]
    Show {
        /// Block ID (ULID)
        id: String,

        /// Show related blocks
        #[arg(short, long)]
        related: bool,
    },

    /// Link blocks
    #[command(alias = "ln")]
    Link {
        /// Source block ID
        from: String,

        /// Target block ID
        to: String,

        /// Link type: extends, refines, contradicts, questions, supports, references, related, similar_to, section_of, next
        #[arg(short = 't', long, default_value = "related")]
        link_type: String,

        /// Sequence weight (for ordered links)
        #[arg(short = 'w', long, default_value = "0.0")]
        weight: f32,

        /// Context
        #[arg(short = 'c', long)]
        context: Option<String>,
    },

    /// Traverse the structural spine
    Traverse {
        /// Starting block ID (optional, defaults to spine root)
        #[arg(short, long)]
        from: Option<String>,

        /// Maximum depth
        #[arg(short = 'd', long, default_value = "10")]
        depth: u32,

        /// Filter by link type
        #[arg(short = 't', long)]
        link_type: Option<String>,

        /// Show content
        #[arg(short = 'c', long)]
        content: bool,
    },

    /// Check gravity hooks and semantic clustering
    GravityCheck {
        /// Block ID to check
        id: String,

        /// Threshold for similarity
        #[arg(short = 't', long, default_value = "0.7")]
        threshold: f32,
    },

    /// Generate table of contents
    Toc {
        /// Structure block ID
        id: String,
    },

    /// Synthesize a document
    Synthesize {
        /// Structure block ID
        id: String,

        /// Output format: pdf, html, markdown, typst
        #[arg(short = 'o', long, default_value = "pdf")]
        output: String,

        /// Template name
        #[arg(short = 't', long)]
        template: Option<String>,

        /// Output file
        #[arg(short = 'f', long)]
        file: Option<String>,
    },

    /// Manage ghost nodes
    Ghost {
        #[command(subcommand)]
        command: GhostCommands,
    },

    /// Interactive TUI
    Architect,

    /// Validate structural integrity
    Lint {
        /// Fix issues automatically
        #[arg(short, long)]
        fix: bool,
    },

    /// Promote a block to a higher order type
    Promote {
        /// Block ID to promote
        id: String,
        /// Target block type
        #[arg(short = 't', long, default_value = "permanent")]
        block_type: String,
        /// Add to staging automatically
        #[arg(long)]
        stage: bool,
    },

    /// Database management
    Db {
        #[command(subcommand)]
        command: DbCommands,
    },

    /// Version control commands (Git-like)
    Version {
        #[command(subcommand)]
        command: VersionCommands,
    },

    /// MCP server (stdio mode for AI agents)
    Mcp,

    /// Session management commands
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },

    /// Check PKM-AI health and configuration
    Doctor,
}

#[derive(Debug, Subcommand)]
pub enum GhostCommands {
    /// List ghost nodes
    List,

    /// Show ghost node details
    Show {
        id: String,
    },

    /// Fill a ghost node with real content
    Fill {
        id: String,

        /// Content
        #[arg(short, long)]
        content: String,
    },

    /// Dismiss a ghost node
    Dismiss {
        id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum DbCommands {
    /// Initialize database
    Init,

    /// Export database
    Export {
        #[arg(short, long)]
        format: String,
    },

    /// Import database
    Import {
        file: String,
    },

    /// Show database statistics
    Stats,
}

#[derive(Debug, Subcommand)]
pub enum VersionCommands {
    /// Show working tree status
    Status {
        /// Repository path (defaults to current directory)
        #[arg(short = 'r', long)]
        repo: Option<String>,
    },

    /// Show commit logs
    Log {
        /// Repository path (defaults to current directory)
        #[arg(short = 'r', long)]
        repo: Option<String>,

        /// Show one line per commit
        #[arg(short, long)]
        oneline: bool,

        /// Show ASCII graph
        #[arg(short, long)]
        graph: bool,

        /// Limit number of commits
        #[arg(short = 'n', long, default_value = "50")]
        limit: usize,
    },

    /// Show changes between commits
    Diff {
        /// Repository path (defaults to current directory)
        #[arg(short = 'r', long)]
        repo: Option<String>,

        /// Block ID to diff
        #[arg(short = 'b', long)]
        block_id: Option<String>,
    },

    /// Stage changes for commit
    Add {
        /// Repository path (defaults to current directory)
        #[arg(short = 'r', long)]
        repo: Option<String>,

        /// Block ID to stage
        block_id: String,
    },

    /// Create a new commit
    Commit {
        /// Repository path (defaults to current directory)
        #[arg(short = 'r', long)]
        repo: Option<String>,

        /// Commit message
        #[arg(short = 'm', long)]
        message: String,

        /// Author name
        #[arg(short = 'a', long, default_value = "user")]
        author: String,

        /// Amend the last commit
        #[arg(long)]
        amend: bool,

        /// Amend without changing message (use last commit message)
        #[arg(long)]
        no_edit: bool,
    },

    /// List all branches
    Branch {
        /// Repository path (defaults to current directory)
        #[arg(short = 'r', long)]
        repo: Option<String>,

        /// Branch name (to create)
        name: Option<String>,

        /// Delete a branch
        #[arg(long)]
        delete: bool,

        /// Force delete (even if not merged)
        #[arg(long)]
        force_delete: bool,
    },

    /// Switch branches
    Checkout {
        /// Repository path (defaults to current directory)
        #[arg(short = 'r', long)]
        repo: Option<String>,

        /// Branch name
        name: String,

        /// Create and switch to new branch
        #[arg(short = 'b', long)]
        create_new: bool,
    },

    /// Merge a branch into current HEAD
    Merge {
        /// Repository path (defaults to current directory)
        #[arg(short = 'r', long)]
        repo: Option<String>,

        /// Branch name to merge
        name: String,

        /// Merge strategy: ours, theirs, or merge (default)
        #[arg(short = 's', long)]
        strategy: Option<String>,
    },

    /// Tag operations
    Tag {
        /// Repository path (defaults to current directory)
        #[arg(short = 'r', long)]
        repo: Option<String>,

        /// Tag name
        name: Option<String>,

        /// Commit ID to tag (defaults to HEAD)
        #[arg(short = 'c', long)]
        commit_id: Option<String>,

        /// Tag message (for annotated tags)
        #[arg(short = 'm', long)]
        message: Option<String>,

        /// Delete a tag
        #[arg(long)]
        delete: bool,
    },

    /// Search commit messages
    LogGrep {
        /// Repository path (defaults to current directory)
        #[arg(short = 'r', long)]
        repo: Option<String>,

        /// Pattern to search for
        pattern: String,

        /// Limit number of commits
        #[arg(short = 'n', long, default_value = "50")]
        limit: usize,
    },

    /// List orphan blocks (blocks without incoming edges)
    Orphan {
        /// Repository path (defaults to current directory)
        #[arg(short = 'r', long)]
        repo: Option<String>,
    },

    /// Reset HEAD to a previous commit
    Reset {
        /// Repository path (defaults to current directory)
        #[arg(short = 'r', long)]
        repo: Option<String>,

        /// Soft reset: keep changes staged
        #[arg(long)]
        soft: bool,

        /// Hard reset: discard all changes
        #[arg(long)]
        hard: bool,

        /// Commit to reset to (defaults to HEAD~1)
        #[arg(short, long)]
        commit: Option<String>,
    },

    /// Rebase current branch onto another branch
    Rebase {
        /// Repository path (defaults to current directory)
        #[arg(short = 'r', long)]
        repo: Option<String>,

        /// Branch to rebase onto
        branch: String,
    },

    /// Push refs to a remote
    Push {
        /// Repository path (defaults to current directory)
        #[arg(short = 'r', long)]
        repo: Option<String>,

        /// Remote name
        #[arg(short = 'm', long, default_value = "origin")]
        remote: String,

        /// Refs to push (defaults to all)
        #[arg(short = 'r', long)]
        refs: Vec<String>,

        /// Force push (skip fast-forward check)
        #[arg(short = 'f', long)]
        force: bool,
    },

    /// Pull from a remote
    Pull {
        /// Repository path (defaults to current directory)
        #[arg(short = 'r', long)]
        repo: Option<String>,

        /// Remote name
        #[arg(short = 'm', long, default_value = "origin")]
        remote: String,

        /// Branch to pull
        #[arg(short = 'b', long)]
        branch: String,

        /// Merge strategy: ours, theirs, or merge (default)
        #[arg(short = 's', long)]
        strategy: Option<String>,
    },

    /// Clone a repository
    Clone {
        /// Source repository path
        source: String,

        /// Destination path (defaults to source directory name)
        #[arg(short = 'd', long)]
        destination: Option<String>,

        /// Branch to clone (defaults to main)
        #[arg(short = 'b', long)]
        branch: Option<String>,

        /// Clone depth (for shallow clone)
        #[arg(short = 'D', long)]
        depth: Option<usize>,
    },

    /// Fetch from a remote without applying changes
    Fetch {
        /// Repository path (defaults to current directory)
        #[arg(short = 'r', long)]
        repo: Option<String>,

        /// Remote name
        #[arg(short = 'm', long, default_value = "origin")]
        remote: String,
    },

    /// List remotes
    RemoteList {
        /// Repository path (defaults to current directory)
        #[arg(short = 'r', long)]
        repo: Option<String>,
    },

    /// Add a remote
    RemoteAdd {
        /// Repository path (defaults to current directory)
        #[arg(short = 'r', long)]
        repo: Option<String>,

        /// Remote name
        name: String,

        /// Remote path or URL
        url: String,
    },
}
#[derive(Debug, Subcommand)]
pub enum SessionCommands {
    /// Start a new knowledge session
    Start {
        /// Agent name (e.g., claude-code, opencode)
        #[arg(long)]
        agent: String,

        /// Project name
        #[arg(long)]
        project: String,

        /// Session ID (optional, auto-generated if not provided)
        #[arg(long)]
        session_id: Option<String>,

        /// Working directory
        #[arg(long)]
        cwd: Option<String>,

        /// Session description
        #[arg(long)]
        description: Option<String>,
    },

    /// End the current session
    End {
        /// Session ID
        #[arg(long)]
        session_id: String,

        /// Project name
        #[arg(long)]
        project: Option<String>,

        /// Auto-generate summary
        #[arg(long)]
        auto_summary: Option<bool>,

        /// Summary message
        summary: Option<String>,

        /// Promote blocks created in session to target type (literature|permanent)
        #[arg(long)]
        promote_to: Option<String>,
    },

    /// List all sessions
    List {
        /// Limit results
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Project name
        #[arg(long)]
        project: Option<String>,
    },

    /// Restore a session context
    Restore {
        /// Session ID to restore
        #[arg(long)]
        session_id: String,

        /// Project name
        #[arg(long)]
        project: Option<String>,
    },

    /// Create a checkpoint before context compaction
    Checkpoint {
        /// Session ID
        #[arg(long)]
        session_id: String,

        /// Project name
        #[arg(long)]
        project: Option<String>,
    },

    /// Capture a block linked to an active session
    Capture {
        /// Session ID
        #[arg(long)]
        session_id: String,

        /// Project name
        #[arg(long)]
        project: Option<String>,

        /// Content for the capture
        content: String,
    },

    /// List blocks created in a session
    Blocks {
        /// Session ID
        #[arg(long)]
        session_id: String,

        /// Project name
        #[arg(long)]
        project: Option<String>,
    },
}
impl Cli {
    pub async fn execute(&self) -> anyhow::Result<()> {
        // Version commands don't need the database - they use filesystem-based VersionRepo
        if let Commands::Version { command } = &self.command {
            use commands::version;
            match command {
                VersionCommands::Status { repo } => {
                    return version::status(repo.as_deref()).await;
                }
                VersionCommands::Log { repo, oneline, graph, limit } => {
                    return version::log(repo.as_deref(), *oneline, *graph, *limit).await;
                }
                VersionCommands::Diff { repo, block_id } => {
                    return version::diff(repo.as_deref(), block_id.as_deref()).await;
                }
                VersionCommands::Add { repo, block_id } => {
                    return version::add(repo.as_deref(), block_id).await;
                }
                VersionCommands::Commit { repo, message, author, amend, no_edit } => {
                    return version::commit(repo.as_deref(), message, author, *amend, *no_edit).await;
                }
                VersionCommands::Branch { repo, name, delete, force_delete } => {
                    if *delete || *force_delete {
                        if let Some(branch_name) = name {
                            return version::branch_delete(repo.as_deref(), branch_name, *force_delete).await;
                        } else {
                            anyhow::bail!("Branch name required for delete");
                        }
                    } else if let Some(branch_name) = name {
                        return version::branch_create(repo.as_deref(), branch_name).await;
                    } else {
                        return version::branch_list(repo.as_deref()).await;
                    }
                }
                VersionCommands::Checkout { repo, name, create_new } => {
                    return version::checkout(repo.as_deref(), name, *create_new).await;
                }
                VersionCommands::Merge { repo, name, strategy } => {
                    return version::merge(repo.as_deref(), name, strategy.as_deref()).await;
                }
                VersionCommands::Tag { repo, name, commit_id, message, delete } => {
                    if *delete {
                        if let Some(tag_name) = name {
                            return version::tag_delete(repo.as_deref(), tag_name).await;
                        } else {
                            anyhow::bail!("Tag name required for delete");
                        }
                    } else if let Some(tag_name) = name {
                        return version::tag_create(
                            repo.as_deref(),
                            tag_name,
                            commit_id.as_deref(),
                            message.as_deref(),
                        ).await;
                    } else {
                        return version::tag_list(repo.as_deref()).await;
                    }
                }
                VersionCommands::LogGrep { repo, pattern, limit } => {
                    return version::log_grep(repo.as_deref(), pattern, *limit).await;
                }
                VersionCommands::Orphan { repo } => {
                    return version::orphan_list(repo.as_deref()).await;
                }
                VersionCommands::Reset { repo, soft, hard, commit } => {
                    return version::reset(repo.as_deref(), *soft, *hard, commit.as_deref()).await;
                }
                VersionCommands::Rebase { repo, branch } => {
                    return version::rebase(repo.as_deref(), branch).await;
                }
                VersionCommands::Push { repo, remote, refs, force } => {
                    use commands::sync;
                    return sync::push(repo.as_deref(), remote, refs, *force).await;
                }
                VersionCommands::Pull { repo, remote, branch, strategy } => {
                    use commands::sync;
                    return sync::pull(repo.as_deref(), remote, branch, strategy.as_deref()).await;
                }
                VersionCommands::Fetch { repo, remote } => {
                    use commands::sync;
                    return sync::fetch(repo.as_deref(), remote).await;
                }
                VersionCommands::RemoteList { repo } => {
                    use commands::sync;
                    return sync::remote_list(repo.as_deref()).await;
                }
                VersionCommands::RemoteAdd { repo, name, url } => {
                    use commands::sync;
                    return sync::remote_add(repo.as_deref(), name, url).await;
                }
                VersionCommands::Clone { source, destination, branch, depth } => {
                    use commands::sync;
                    return sync::clone(source, destination.as_deref(), branch.as_deref(), *depth).await;
                }
            }
        }

        // MCP commands don't need database - they run in stdio mode
        if let Commands::Mcp = &self.command {
            return commands::mcp::execute_serve().await;
        }

        // Initialize database for other commands
        use crate::db::Database;
        use std::path::PathBuf;

        // Determine database path priority:
        // 1. Explicit --db-path flag
        // Determine database path with priority:
        // 1. --db-path flag (explicit override)
        // 2. .pkmai/config.toml in current directory
        // 3. ~/.pkmai/config.toml in home directory
        // 4. Default: ~/.pkmai/data.db
        let db_path = if let Some(path) = &self.db_path {
            tracing::info!("[CONFIG] Using explicit --db-path: {}", path);
            PathBuf::from(path)
        } else if let Some(config) = config::Config::find() {
            // Determine which config file was found
            let config_source = if std::path::Path::new(".pkmai/config.toml").exists() {
                ".pkmai/config.toml"
            } else {
                "~/.pkmai/config.toml"
            };
            tracing::info!("[CONFIG] Using {} (priority: project > home)", config_source);
            config.database_path()
        } else {
            // Default: ~/.pkmai/data.db
            let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push(".pkmai");
            std::fs::create_dir_all(&path).ok();
            path.push("data.db");
            tracing::info!("[CONFIG] No config found, using default: {}", path.display());
            path
        };

        let db = Database::rocksdb(&db_path).await?;

        // Calculate auto_stage: defaults to true unless --no-stage is specified
        let auto_stage = !self.no_stage;

        match &self.command {
            Commands::Init { .. } => {
                // Should not reach here - handled before database init
                unreachable!("Init is handled before database init");
            }
            Commands::Create { block_type, title, content, tags, interactive } => {
                commands::create::execute(&db, block_type, title, content, tags, self.verbose, auto_stage, *interactive).await
            }
            Commands::Quick { content, block_type, tags } => {
                commands::quick::execute(&db, content, block_type, tags).await
            }
            Commands::List { block_type, tags, search, limit, output } => {
                commands::list::execute(&db, block_type, tags, search, *limit, output).await
            }
            Commands::Search { query, limit } => {
                commands::search::execute(&db, query, *limit).await
            }
            Commands::Grep { pattern, content_only, case_sensitive, limit } => {
                commands::grep::execute(&db, pattern, *content_only, *case_sensitive, *limit).await
            }
            Commands::Show { id, related } => {
                commands::show::execute(&db, id, *related).await
            }
            Commands::Link { from, to, link_type, weight, context } => {
                commands::link::execute(&db, from, to, link_type, *weight, context, auto_stage).await
            }
            Commands::Traverse { from, depth, link_type, content } => {
                commands::traverse::execute(&db, from, *depth, link_type, *content).await
            }
            Commands::GravityCheck { id, threshold } => {
                commands::gravity_check::execute(&db, id, *threshold).await
            }
            Commands::Toc { id } => {
                commands::toc::execute(&db, id).await
            }
            Commands::Synthesize { id, output, template, file } => {
                commands::synthesize::execute(&db, id, output, template, file).await
            }
            Commands::Ghost { command } => {
                commands::ghost::execute(&db, command).await
            }
            Commands::Architect => {
                commands::architect::execute(&db).await
            }
            Commands::Lint { fix } => {
                commands::lint::execute(&db, *fix).await
            }
            Commands::Promote { id, block_type, stage } => {
                commands::promote::execute(&db, id, block_type, *stage).await
            }
            Commands::Db { command } => {
                commands::db_cmd::execute(&db, command).await
            }
            Commands::Version { .. } => {
                // Should not reach here - handled above
                unreachable!("Version commands are handled before database init");
            }
            Commands::Mcp { .. } => {
                // Should not reach here - handled above
                unreachable!("Mcp commands are handled before database init");
            }
            Commands::Session { command } => {
                commands::session::execute(command).await
            }
            Commands::Doctor => {
                commands::doctor::execute(&db).await
            }
        }
    }
}
