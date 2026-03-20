# PKM-AI Conceptos

**Version:** 1.0
**Fecha:** 2026-03-19
**Estado:** Referencia Canonica

---

## Descripcion General

Este documento proporciona definiciones canonicas para todos los conceptos de PKM-AI. Cuando la documentacion y la implementacion discrepan, este documento establece la verdad.

---

## 1. Conceptos del Dominio Core

### 1.1 Block

La unidad fundamental de contenido en PKM-AI.

```rust
// Definicion canonica en src/models/block.rs
pub struct Block {
    pub id: Ulid,                           // Timestamp + aleatorio, ordenable cronologicamente
    pub block_type: BlockType,              // Tipo semantico
    pub content: String,                     // Contenido en Markdown
    pub properties: Map<String, Value>,      // Metadatos flexibles
    pub embedding_bloom: Option<[u128; 1]>,  // Filtro Bloom para busqueda semantica
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

| Propiedad | Tipo | Descripcion |
|----------|------|-------------|
| `id` | `Ulid` | Identificador unico, ordenable por tiempo de creacion |
| `block_type` | `BlockType` | Clasificacion semantica |
| `content` | `String` | Contenido formateado en Markdown |
| `properties` | `Map<String, Value>` | Metadatos clave-valor arbitrarios |
| `embedding_bloom` | `Option<[u128; 1]>` | Filtro Bloom para busqueda semantica rapida |
| `created_at` | `DateTime<Utc>` | Timestamp de creacion |
| `updated_at` | `DateTime<Utc>` | Timestamp de ultima modificacion |

**Block Types (Tipos de Block):**

| Tipo | Alias | Descripcion |
|------|-------|-------------|
| `fleeting` | `f` | Notas de captura temporales |
| `literature` | `l` | Material de referencia de fuentes externas |
| `permanent` | `p` | Notas atomicas Zettelkasten (tipo core) |
| `structure` | `s`, `index`, `moc` | Contenedores estructurales |
| `hub` | `h` | Nodos centrales de tema (puntos de entrada) |
| `task` | `t` | Elementos de accion |
| `reference` | `r` | Referencias externas |
| `outline` | `o` | Esquemas jerarquicos |
| `ghost` | `g` | Marcador de posicion para contenido faltante |

### 1.2 Structure

Un Block contenedor que organiza otros Blocks.

```rust
pub struct Structure {
    pub id: Ulid,
    pub name: String,
    pub root_blocks: Vec<Ulid>,                    // Blocks de nivel superior
    pub block_tree: Map<Ulid, Vec<Ulid>>,          // mapeo padre -> hijos
    pub spine_order: Vec<(Ulid, FractionalIndex)>, // Secuencia ordenada
    pub properties: Map<String, Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Distincion Clave:**
- Un `Block` con `block_type = Structure` representa un documento o coleccion
- Una struct `Structure` representa el ordenamiento interno de un documento

### 1.3 Edge

Una relacion tipada entre dos Blocks.

```rust
pub struct Edge {
    pub id: Ulid,
    pub link_type: LinkType,
    pub from: Ulid,    // Block origen
    pub to: Ulid,      // Block destino
    pub properties: Map<String, Value>,
    pub sequence_weight: FractionalIndex,  // Posicion en la secuencia
}
```

**Link Types (Tipos de Enlace):**

| Tipo | Categoria | Descripcion |
|------|----------|-------------|
| `extends` | Semantico | Block extiende a otro |
| `refines` | Semantico | Block refina a otro |
| `contradicts` | Semantico | Block contradice a otro |
| `questions` | Semantico | Block cuestiona a otro |
| `supports` | Semantico | Block soporta a otro |
| `references` | Semantico | Block referencia a otro |
| `related` | Semantico | Blocks estan relacionados (predeterminado) |
| `similar_to` | Semantico | Blocks son similares |
| `section_of` | Estructural | Block es seccion de una Structure |
| `subsection_of` | Estructural | Block es subseccion |
| `ordered_child` | Estructural | Hijo ordenado en jerarquia |
| `next` | Estructural | Siguiente en secuencia (Structural Spine) |
| `next_sibling` | Estructural | Hermano siguiente |
| `first_child` | Estructural | Primer hijo |
| `contains` | Estructural | Block contiene a otro |
| `parent` | Estructural | Block es padre |
| `ai_suggested` | AI | Enlace sugerido por IA |

---

## 2. Structural Spine

La **Structural Spine** es el eje ordenado de un documento, implementado como Blocks enlazados via bordes `NEXT`.

### 2.1 Definicion

```
Block A (peso: "a") → Block B (peso: "am") → Block C (peso: "b")
```

**Principios:**
- Cada Block en una Spine tiene exactamente un borde `next` (excepto el ultimo)
- La navegacion es deterministica y preserva el orden
- La inserccion entre dos Blocks genera una clave punto medio

### 2.2 FractionalIndex

**NO usar `f32` para ordenamiento de secuencia.** Usar indexacion fraccional lexografica.

```rust
// Implementacion correcta
pub struct FractionalIndex(String);

impl FractionalIndex {
    pub fn between(before: &FractionalIndex, after: &FractionalIndex) -> Self {
        // Punto medio lexografico
    }

    pub fn first() -> Self {
        FractionalIndex("a".to_string())
    }

    pub fn after(last: &FractionalIndex) -> Self {
        // Agregar 'a' para extender
    }
}
```

### 2.3 Reglas de Navegacion

1. Iniciar desde el Block raiz
2. Seguir bordes `NEXT` en orden
3. Respetar limite de profundidad (predeterminado: 100)
4. Detectar y manejar ciclos

---

## 3. Smart Sections

Un Block `Structure` mejorado con conciencia semantica.

```rust
pub struct SmartSection {
    pub block: Block,              // El Block estructura subyacente
    pub intent: String,            // Descripcion del proposito
    pub boundary_constraints: Vec<Constraint>,
    pub semantic_centroid: Vec<f32>,  // Embedding promedio
    pub medoid_id: Option<Ulid>,   // Block mas representativo
    pub vacancy_status: Vacancy,   // Indicador de capacidad
    pub coherence_score: f32,       // 0.0 - 1.0
}

pub enum Vacancy {
    Full,       // >90% capacidad
    NearlyFull, // 70-90%
    Partial,    // 30-70%
    Sparse,     // 10-30%
    Empty,      // <10%
}
```

### 3.1 Calculo del Semantic Centroid

```rust
pub fn calculate_weighted_centroid(
    blocks: &[Block],
    weights: &[f32],  // Basado en enlaces entrantes + 1
) -> Vec<f32> {
    let total: f32 = weights.iter().sum();
    let mut centroid = vec![0.0; EMBEDDING_DIM];

    for (block, weight) in blocks.iter().zip(weights) {
        for (i, val) in block.embedding.iter().enumerate() {
            centroid[i] += val * weight / total;
        }
    }
    centroid
}
```

---

## 4. Ghost Nodes

Marcadores de posicion predictivos para contenido faltante.

```rust
pub struct GhostNode {
    pub id: Ulid,
    pub expected_keywords: Vec<String>,
    pub confidence: f32,           // 0.0 - 1.0
    pub parent_id: Ulid,           // Structure que contiene este hueco
    pub suggested_position: FractionalIndex,
    pub status: GhostStatus,
}

pub enum GhostStatus {
    Pending,   // Aun no abordado
    Filled,    // Contenido ha sido agregado
    Dismissed, // Intencionalmente dejado vacio
}
```

### 4.1 Algoritmo de Deteccion

1. Obtener todos los Blocks en una Structure
2. Calcular centros semanticos para cada seccion
3. Para cada par consecutivo, calcular distancia
4. Si distancia > umbral, insertar GhostNode

---

## 5. Versionado (API estilo Git)

### 5.1 Commit

```rust
pub struct Commit {
    pub id: CommitId,
    pub structure_snapshot: Structure,
    pub parents: Vec<CommitId>,
    pub author: AgentId,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub blocks_added: Vec<Ulid>,
    pub blocks_removed: Vec<Ulid>,
    pub blocks_modified: Vec<Ulid>,
}
```

### 5.2 View (Ref)

```rust
pub enum View {
    Branch {
        name: ViewName,
        target: Ulid,      // Commit ID
        is_head: bool,
    },
    Tag {
        name: ViewName,
        target: Ulid,
        message: String,   // Mensaje de etiqueta anotada
    },
}
```

### 5.3 WorkingSet (Index)

```rust
pub struct WorkingSet {
    pub id: WorkingSetId,
    pub author: AgentId,
    pub staged_blocks: BTreeMap<Ulid, BlockDelta>,
    pub staged_edges: BTreeMap<(Ulid, Ulid), EdgeDelta>,
    pub removed_blocks: Vec<Ulid>,
    pub removed_edges: Vec<(Ulid, Ulid)>,
    pub operations: Vec<Operation>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

---

## 6. Conceptos de Agente

### 6.1 AgentId

```rust
pub struct AgentId(String);

impl AgentId {
    pub fn new(id: String) -> Self;
    pub fn as_str(&self) -> &str;
}
```

### 6.2 Flujo de Trabajo del Agente

1. **Capture**: El Agente captura notas temporales
2. **Crystallize**: Convertir a notas permanentes
3. **Link**: Crear bordes semanticos
4. **Structure**: Agregar a Structural Spine
5. **Synthesize**: Generar documentos

---

## 7. Capa de Base de Datos

### 7.1 Tablas de Esquema

```sql
-- Tabla Block (SCHEMAFULL)
DEFINE TABLE block SCHEMAFULL;
DEFINE FIELD id ON block TYPE ulid PRIMARY KEY;
DEFINE FIELD block_type ON block TYPE string;
DEFINE FIELD content ON block TYPE string;
DEFINE FIELD properties ON block TYPE object;
DEFINE FIELD embedding_bloom ON block TYPE option<array>;
DEFINE FIELD created_at ON block TYPE datetime;
DEFINE FIELD updated_at ON block TYPE datetime;

-- Tabla Edge
DEFINE TABLE edge SCHEMAFULL;
DEFINE FIELD id ON edge TYPE ulid PRIMARY KEY;
DEFINE FIELD link_type ON edge TYPE string;
DEFINE FIELD from ON edge TYPE ulid;
DEFINE FIELD to ON edge TYPE ulid;
DEFINE FIELD properties ON edge TYPE object;
DEFINE FIELD sequence_weight ON edge TYPE string;  -- FractionalIndex como string

-- Indices
DEFINE INDEX idx_block_type ON block FIELDS block_type;
DEFINE INDEX idx_edge_from ON edge FIELDS from;
DEFINE INDEX idx_edge_link_type ON edge FIELDS link_type;
```

---

## 8. Comandos CLI

### 8.1 Comandos Core

| Comando | Descripcion |
|---------|-------------|
| `pkmai create` | Crear un nuevo Block |
| `pkmai list` | Listar Blocks con filtro |
| `pkmai show` | Mostrar detalles del Block |
| `pkmai link` | Crear enlaces entre Blocks |
| `pkmai grep` | Buscar contenido de Blocks |
| `pkmai traverse` | Navegar la Structural Spine |
| `pkmai gravity-check` | Verificar conectividad de Blocks |
| `pkmai toc` | Generar tabla de contenidos |
| `pkmai synthesize` | Sintetizar documento desde estructura |
| `pkmai ghost` | Gestionar Ghost Nodes |
| `pkmai architect` | Lanzar TUI interactivo |
| `pkmai lint` | Validar integridad estructural |
| `pkmai db` | Gestion de base de datos |
| `pkmai api` | Iniciar servidor REST API |

### 8.2 Comandos de Control de Versiones

| Comando | Equivalente Git | Descripcion |
|---------|---------------|-------------|
| `pkmai version status` | `git status` | Mostrar estado del arbol de trabajo |
| `pkmai version log` | `git log` | Mostrar registros de commits |
| `pkmai version diff` | `git diff` | Mostrar cambios |
| `pkmai version add` | `git add` | Preparar cambios |
| `pkmai version commit` | `git commit` | Crear commit |
| `pkmai version branch` | `git branch` | Gestion de ramas |
| `pkmai version checkout` | `git checkout` | Cambiar ramas |
| `pkmai version merge` | `git merge` | Fusionar ramas |
| `pkmai version tag` | `git tag` | Operaciones de etiquetas |
| `pkmai version push` | `git push` | Enviar a remoto |
| `pkmai version pull` | `git pull` | Traer de remoto |

---

## 9. Herramientas MCP

### 9.1 Herramientas de Block

| Herramienta | Parametros | Descripcion |
|------|------------|-------------|
| `create_block` | `block_type`, `title`, `content?`, `tags?` | Crear Block |
| `get_block` | `id` | Obtener Block por ULID |
| `search_blocks` | `query?`, `block_type?`, `tags?`, `limit?` | Buscar |
| `update_block` | `id`, `title?`, `content?`, `tags?` | Actualizar |

### 9.2 Herramientas de Enlace

| Herramienta | Parametros | Descripcion |
|------|------------|-------------|
| `create_link` | `from_id`, `to_id`, `link_type`, `weight?`, `context?` | Crear enlace |
| `get_links` | `block_id`, `direction?` | Obtener enlaces |
| `suggest_links` | `block_id`, `limit?` | Sugerencias de IA |

### 9.3 Herramientas de Spine

| Herramienta | Parametros | Descripcion |
|------|------------|-------------|
| `traverse_spine` | `root_id?`, `depth?`, `link_type?` | Navegar |
| `gravity_check` | `block_id`, `threshold?` | Verificar conectividad |
| `reorder_block` | `block_id`, `new_position`, `parent_id?` | Reordenar |

### 9.4 Herramientas de Structure

| Herramienta | Parametros | Descripcion |
|------|------------|-------------|
| `get_section_map` | `root_id` | Obtener jerarquia |
| `detect_gaps` | `section_id` | Detectar faltantes |
| `list_ghosts` | `root_id?` | Listar marcadores |

### 9.5 Herramientas de Sintesis

| Herramienta | Parametros | Descripcion |
|------|------------|-------------|
| `synthesize` | `structure_id`, `format?`, `template?` | Generar doc |
| `get_toc` | `structure_id` | Obtener TOC |

---

## 10. Glosario

| Termino | Definicion | Ubicacion Canonica |
|------|------------|-------------------|
| **Block** | Unidad atomica de contenido con ULID | `src/models/block.rs` |
| **Structure** | Block contenedor para organizar Blocks | `src/models/block.rs` |
| **Edge** | Relacion tipada entre Blocks | `src/models/edge.rs` |
| **Structural Spine** | Secuencia ordenada via bordes NEXT | `src/spine/` |
| **Smart Section** | Structure con conciencia semantica | `src/models/smart_section.rs` |
| **Ghost Node** | Marcador de posicion para contenido faltante | `src/models/ghost_node.rs` |
| **FractionalIndex** | Clave de posicion lexografica | `src/utils/fractional_index.rs` |
| **WorkingSet** | Indice de cambios pendientes | `src/models/working_set.rs` |
| **View** | Puntero nombrado a commit (branch/tag) | `src/models/view.rs` |
| **Commit** | Captura de estado de estructura | `src/models/commit.rs` |
| **AgentId** | Identificador unico de agente | `src/models/agent.rs` |
| **Semantic Centroid** | Embedding promedio ponderado por importancia | `smart_section.rs` |
| **Medoid** | Block mas cercano al centroide | `smart_section.rs` |

---

## 11. Referencias

- Modelos de Dominio: `src/models/`
- Comandos CLI: `src/cli/commands/`
- Herramientas MCP: `src/ai/mcp.rs`
- Esquema de Base de Datos: `src/db/schema.rs`
- Tests: `src/tests/`

---

**Ultima actualizacion:** 2026-03-19
**Fuente canonica:** `src/models/` y este documento