//! SurrealDB Schema Definition

pub const SCHEMA_VERSION: &str = "0.1.0";

pub const SCHEMA: &str = r#"
// ==================== BLOCKS ====================

DEFINE TABLE block SCHEMALESS;
DEFINE FIELD ulid ON block TYPE string;
DEFINE FIELD block_type ON block TYPE string
    ASSERT $value IN ['fleeting', 'literature', 'permanent', 'structure', 'hub', 'task', 'reference', 'outline', 'ghost'];
DEFINE FIELD title ON block TYPE string;
DEFINE FIELD content ON block TYPE string;
DEFINE FIELD tags ON block TYPE array<string> DEFAULT [];
DEFINE FIELD metadata ON block FLEXIBLE TYPE object DEFAULT {};
DEFINE FIELD created_at ON block DEFAULT time::now();
DEFINE FIELD updated_at ON block DEFAULT time::now();
DEFINE FIELD version ON block TYPE int DEFAULT 1;
DEFINE FIELD ai_confidence ON block TYPE option<float>
    ASSERT $value >= 0.0 AND $value <= 1.0;
DEFINE FIELD semantic_centroid ON block TYPE option<array<float>> DEFAULT [];

// Indexes
DEFINE INDEX idx_block_type ON block FIELDS block_type;
DEFINE INDEX idx_created_at ON block FIELDS created_at;

// Full-text search (using ascii analyzer for embedded SurrealDB compatibility)
DEFINE ANALYZER block_analyzer TOKENIZERS blank,class FILTERS lowercase;
DEFINE INDEX idx_content_fts ON block FIELDS content SEARCH ANALYZER block_analyzer BM25 HIGHLIGHTS;

// ==================== EDGES ====================

DEFINE TABLE edge SCHEMALESS;
DEFINE FIELD ulid ON edge TYPE string;
DEFINE FIELD src ON edge TYPE string;
DEFINE FIELD dst ON edge TYPE string;
DEFINE FIELD link_type ON edge TYPE string
    ASSERT $value IN [
        'extends', 'refines', 'contradicts', 'questions', 'supports',
        'references', 'related', 'similar_to',
        'section_of', 'subsection_of', 'ordered_child',
        'next', 'next_sibling', 'first_child', 'contains', 'parent',
        'ai_suggested'
    ];
DEFINE FIELD sequence_weight ON edge TYPE string DEFAULT 'a';
DEFINE FIELD context ON edge TYPE option<string>;
DEFINE FIELD ai_justification ON edge TYPE option<string>;
DEFINE FIELD confidence ON edge TYPE option<float>
    ASSERT $value >= 0.0 AND $value <= 1.0;
DEFINE FIELD created_at ON edge DEFAULT time::now();
DEFINE FIELD verified ON edge TYPE bool DEFAULT false;

// Indexes
DEFINE INDEX idx_link_type ON edge FIELDS link_type;
DEFINE INDEX idx_sequence_weight ON edge FIELDS sequence_weight;
DEFINE INDEX idx_src ON edge FIELDS src;
DEFINE INDEX idx_dst ON edge FIELDS dst;

// ==================== SMART SECTIONS ====================

DEFINE TABLE smart_section SCHEMALESS;
DEFINE FIELD intent ON smart_section TYPE string;
DEFINE FIELD boundary_constraints ON smart_section TYPE array<string> DEFAULT [];
DEFINE FIELD keywords ON smart_section TYPE array<string> DEFAULT [];
DEFINE FIELD semantic_centroid ON smart_section TYPE option<array<float>> DEFAULT [];
DEFINE FIELD density ON smart_section TYPE int DEFAULT 0;
DEFINE FIELD expected_density ON smart_section TYPE int;
DEFINE FIELD vacancy ON smart_section TYPE string
    ASSERT $value IN ['full', 'nearly_full', 'partial', 'sparse', 'empty'];
DEFINE FIELD coherence_score ON smart_section TYPE float DEFAULT 0.0;
DEFINE FIELD gravity_hooks ON smart_section TYPE array<string> DEFAULT [];

// ==================== GHOST NODES ====================

DEFINE TABLE ghost_node SCHEMALESS;
DEFINE FIELD description ON ghost_node TYPE string;
DEFINE FIELD ai_rationale ON ghost_node TYPE string;
DEFINE FIELD confidence ON ghost_node TYPE float
    ASSERT $value >= 0.0 AND $value <= 1.0;
DEFINE FIELD position_hint ON ghost_node FLEXIBLE TYPE object DEFAULT {};
DEFINE FIELD status ON ghost_node TYPE string
    ASSERT $value IN ['detected', 'acknowledged', 'in_progress', 'filled', 'dismissed'];
DEFINE FIELD trigger_blocks ON ghost_node TYPE array<string> DEFAULT [];
DEFINE FIELD expected_keywords ON ghost_node TYPE array<string> DEFAULT [];
DEFINE FIELD created_at ON ghost_node DEFAULT time::now();
DEFINE FIELD filled_by ON ghost_node TYPE string;

// Indexes
DEFINE INDEX idx_ghost_status ON ghost_node FIELDS status;
DEFINE INDEX idx_ghost_confidence ON ghost_node FIELDS confidence;

// ==================== STRUCTURAL SPINE ====================

DEFINE TABLE structural_spine SCHEMALESS;
DEFINE FIELD roots ON structural_spine TYPE array<string> DEFAULT [];
DEFINE FIELD total_nodes ON structural_spine TYPE int DEFAULT 0;
DEFINE FIELD lint_score ON structural_spine TYPE float DEFAULT 0.0;

"#;

/// Initialize database schema
pub async fn init_schema(db: &surrealdb::Surreal<surrealdb::engine::local::Db>) -> crate::NexusResult<()> {
    tracing::info!("Initializing database schema v{}", SCHEMA_VERSION);

    db.query(SCHEMA)
        .await
        .map_err(|e| crate::NexusError::Database(e.to_string()))?;

    tracing::info!("Schema initialized successfully");

    Ok(())
}
