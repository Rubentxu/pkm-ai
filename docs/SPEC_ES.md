# PKM-AI Specification

**Version:** 1.0
**Fecha:** 2026-03-19
**Estado:** Canonico
**Proyecto:** hodei-pkm

---

## Tabla de Contenidos

1. [Vision y Principios](#1-vision-y-principios)
2. [Modelos de Dominio](#2-modelos-de-dominio)
3. [Columna Vertebral Estructural](#3-columna-vertebral-estructural)
4. [Secciones Inteligentes](#4-secciones-inteligentes)
5. [Nodos Fantasma](#5-nodos-fantasma)
6. [API Estilo Git](#6-api-estilo-git)
7. [Comandos CLI](#7-comandos-cli)
8. [Herramientas MCP](#8-herramientas-mcp)
9. [API REST](#9-api-rest)
10. [Arquitectura](#10-arquitectura)
11. [Estado de Implementacion](#11-estado-de-implementacion)

---

## 1. Vision y Principios

### 1.1 Vision Central

PKM-AI es un **Sistema Operativo de Conocimiento** para equipos que trabajan con enjambres de agentes de IA. Mientras que herramientas como Obsidian o Logseq son zoológicos individuales de notas, PKM-AI esta construido para:

- Multiples agentes de IA operando concurrentemente en la misma base de conocimiento
- Sintesis automatica de fragmentos Zettelkasten en documentos tecnicos profesionales
- Ordenamiento determinista emergente de la estructura, no impuesto jerarquicamente
- Alto rendimiento (objetivo: 65,000 bloques con operaciones de grafo en <16ms)

### 1.2 Principios Fundamentales

| Principio | Descripcion |
|-----------|-------------|
| **Modelo de Bloques-Atomo** | Cada pieza de conocimiento es un bloque direccionable con ULID |
| **Columna Vertebral Estructural Primero** | Orden y estructura son ciudadanos de primera clase |
| **Separacion Semantica/Estructural** | Aristas semanticas (enlaces) vs aristas estructurales (ordenamiento) |
| **Nodos Fantasma como Predicados** | Los huecos son restricciones que describen contenido ideal |
| **Rendimiento como Requisito** | O(Delta bloques) no O(N) para ganchos de gravedad |

### 1.3 Relacion entre Proyectos

PKM-AI y Nexus-WASM son proyectos hermanos bajo "hodei-pkm":

| Aspecto | Nexus-WASM | PKM-AI |
|---------|-----------|---------|
| **Dominio** | Runtime WASM de alto rendimiento | Sistema operativo de conocimiento |
| **Objetivo** | 65,536 entidades @ 60 FPS | 65,000+ bloques con agentes concurrentes |
| **Arquitectura** | Modelo de actores sobre SAB | Modelo de actores sobre SurrealDB daemon |
| **Coordinacion** | SharedArrayBuffer + Atomics | Unix socket + LeaseManager |

---

## 2. Modelos de Dominio

### 2.1 Bloque

```rust
// Canonico: src/models/block.rs
pub struct Block {
    pub id: Ulid,                           // Ordenable cronologicamente
    pub block_type: BlockType,
    pub content: String,                    // Contenido Markdown
    pub properties: Map<String, Value>,    // Metadatos flexibles
    pub embedding_bloom: Option<[u128; 1]>, // Filtro de busqueda semantica
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Tipos de Bloque:**

| Tipo | Alias | Descripcion |
|------|-------|-------------|
| `fleeting` | `f` | Notas de captura temporal |
| `literature` | `l` | Material de referencia de fuentes externas |
| `permanent` | `p` | Notas atomicas Zettelkasten |
| `structure` | `s`, `moc` | Contenedores estructurales |
| `hub` | `h` | Puntos de entrada de tema central |
| `task` | `t` | Elementos de accion |
| `reference` | `r` | Referencias externas |
| `outline` | `o` | Esquemas jerarquicos |
| `ghost` | `g` | Marcador de posicion para contenido faltante |

### 2.2 Arista

```rust
pub struct Edge {
    pub id: Ulid,
    pub link_type: LinkType,
    pub from: Ulid,    // Bloque origen
    pub to: Ulid,      // Bloque destino
    pub properties: Map<String, Value>,
    pub sequence_weight: FractionalIndex,  // Clave de posicion (NO f32!)
    pub updated_at: DateTime<Utc>,
}
```

**Tipos de Enlace:**

| Tipo | Categoria | Descripcion |
|------|-----------|-------------|
| `extends` | Semantico | Bloque extiende otro |
| `refines` | Semantico | Bloque refina otro |
| `contradicts` | Semantico | Bloque contradice otro |
| `questions` | Semantico | Bloque cuestiona otro |
| `supports` | Semantico | Bloque soporta otro |
| `references` | Semantico | Bloque referencia otro |
| `related` | Semantico | Relacionado (por defecto) |
| `similar_to` | Semantico | Similar a otro |
| `section_of` | Estructural | Bloque es seccion de una Estructura |
| `subsection_of` | Estructural | Subseccion |
| `ordered_child` | Estructural | Hijo ordenado |
| `next` | Estructural | Siguiente en secuencia |
| `next_sibling` | Estructural | Hermano siguiente |
| `first_child` | Estructural | Primer hijo |
| `contains` | Estructural | Contiene otro |
| `parent` | Estructural | Padre de |
| `ai_suggested` | IA | Enlace sugerido por IA |

### 2.3 FractionalIndex (CRITICO: NO f32)

**NO usar `f32` para ordenamiento de secuencias.** Usar indexacion fraccionaria lexicografica.

```rust
pub struct FractionalIndex(String);

impl FractionalIndex {
    pub fn first() -> Self;
    pub fn after(last: &FractionalIndex) -> Self;
    pub fn between(before: &FractionalIndex, after: &FractionalIndex) -> Self;
}
```

### 2.4 SmartSection

```rust
pub struct SmartSection {
    pub block: Block,
    pub intent: String,
    pub boundary_constraints: Vec<Constraint>,
    pub semantic_centroid: Vec<f32>,
    pub medoid_id: Option<Ulid>,
    pub vacancy_status: Vacancy,
    pub coherence_score: f32,
}

pub enum Vacancy {
    Full,       // >90%
    NearlyFull, // 70-90%
    Partial,    // 30-70%
    Sparse,     // 10-30%
    Empty,      // <10%
}
```

### 2.5 GhostNode

```rust
pub struct GhostNode {
    pub id: Ulid,
    pub expected_keywords: Vec<String>,
    pub confidence: f32,
    pub parent_id: Ulid,
    pub suggested_position: FractionalIndex,
    pub status: GhostStatus,
}

pub enum GhostStatus {
    Pending,
    Filled,
    Dismissed,
}
```

---

## 3. Columna Vertebral Estructural

### 3.1 Definicion

La **Columna Vertebral Estructural** es el eje ordenado de un documento, implementado como bloques enlazados via aristas `NEXT`.

```
Bloque A (peso: "a") -> Bloque B (peso: "am") -> Bloque C (peso: "b")
```

### 3.2 Reglas de Recorrido

1. Iniciar desde el bloque raiz
2. Seguir aristas `NEXT` en orden
3. Respetar limite de profundidad (por defecto: 100)
4. Detectar y manejar ciclos

### 3.3 Algoritmo de Recorrido

```rust
async fn traverse_spine(
    db: &SurrealDb,
    root: Ulid,
    max_depth: usize,
) -> Result<Vec<Block>> {
    let mut visited = HashSet::new();
    traverse_recursive(db, root, max_depth, 0, &mut visited).await
}

#[async_recursion::async_recursion]
async fn traverse_recursive(
    db: &SurrealDb,
    node: Ulid,
    max_depth: usize,
    current_depth: usize,
    visited: &mut HashSet<Ulid>,
) -> Result<Vec<Block>> {
    if current_depth >= max_depth || visited.contains(&node) {
        return Ok(vec![]);
    }
    visited.insert(node);

    let children = db.query("
        SELECT out.*, sequence_weight
        FROM edge
        WHERE in = $node AND link_type = 'next'
        ORDER BY sequence_weight ASC
    ")
    .bind(("node", node))
    .await?;

    let mut result = Vec::new();
    for child in children {
        if child.block_type == BlockType::Structure(_) {
            let nested = traverse_recursive(db, child.id, max_depth, current_depth + 1, visited).await?;
            result.extend(nested);
        } else {
            result.push(child);
        }
    }
    Ok(result)
}
```

---

## 4. Secciones Inteligentes

### 4.1 Centroide Semantico

Calcular usando **media ponderada** por importancia (enlaces entrantes + 1):

```rust
pub fn calculate_weighted_centroid(
    blocks: &[Block],
    weights: &[f32],
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

### 4.2 Medoide

El **medoide** es el bloque mas cercano al centroide (bloque mas representativo).

---

## 5. Nodos Fantasma

### 5.1 Algoritmo de Deteccion

1. Obtener todos los bloques en una Estructura
2. Calcular centroides semanticos para cada seccion
3. Para cada par consecutivo, calcular la distancia
4. Si la distancia > umbral, insertar GhostNode

---

## 6. API Estilo Git

### 6.1 Commit

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

### 6.2 View (Ref)

```rust
pub enum View {
    Branch {
        name: ViewName,
        target: Ulid,
        is_head: bool,
    },
    Tag {
        name: ViewName,
        target: Ulid,
        message: String,
    },
}
```

### 6.3 WorkingSet (Indice)

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

## 7. Comandos CLI

### 7.1 Comandos Principales

| Comando | Estado | Descripcion |
|---------|--------|-------------|
| `pkmai create` | ✅ | Crear un nuevo bloque |
| `pkmai list` | ✅ | Listar bloques con filtrado |
| `pkmai show` | ✅ | Mostrar detalles del bloque |
| `pkmai link` | ✅ | Crear enlaces entre bloques |
| `pkmai grep` | ✅ | Buscar contenido de bloques |
| `pkmai traverse` | ✅ | Recorrer la columna vertebral estructural |
| `pkmai gravity-check` | ✅ | Verificar conectividad de bloques |
| `pkmai toc` | ✅ | Generar tabla de contenidos |
| `pkmai synthesize` | ✅ | Sintetizar documento desde estructura |
| `pkmai ghost` | ✅ | Gestionar nodos fantasma |
| `pkmai architect` | ✅ | Lanzar TUI interactiva |
| `pkmai lint` | ✅ | Validar integridad estructural |
| `pkmai db` | ✅ | Gestion de base de datos |
| `pkmai api` | ✅ | Iniciar servidor API REST |

### 7.2 Comandos de Control de Versiones

| Comando | Equivalente Git | Estado |
|---------|----------------|--------|
| `version status` | `git status` | ✅ |
| `version log` | `git log` | ✅ |
| `version diff` | `git diff` | ✅ |
| `version add` | `git add` | ✅ |
| `version commit` | `git commit` | ✅ |
| `version branch` | `git branch` | ✅ |
| `version checkout` | `git checkout` | ✅ |
| `version merge` | `git merge` | ✅ |
| `version tag` | `git tag` | ✅ |
| `version push` | `git push` | ✅ |
| `version pull` | `git pull` | ✅ |

**Total Comandos CLI: 34 (100% implementados)**

---

## 8. Herramientas MCP

### 8.1 Herramientas de Bloque (4)

| Herramienta | Parametros | Descripcion |
|------------|------------|-------------|
| `create_block` | `block_type`, `title`, `content?`, `tags?` | Crear bloque |
| `get_block` | `id` | Obtener bloque por ULID |
| `search_blocks` | `query?`, `block_type?`, `tags?`, `limit?` | Buscar |
| `update_block` | `id`, `title?`, `content?`, `tags?` | Actualizar |

### 8.2 Herramientas de Enlace (3)

| Herramienta | Parametros | Descripcion |
|------------|------------|-------------|
| `create_link` | `from_id`, `to_id`, `link_type`, `weight?`, `context?` | Crear enlace |
| `get_links` | `block_id`, `direction?` | Obtener enlaces |
| `suggest_links` | `block_id`, `limit?` | Sugerencias de IA |

### 8.3 Herramientas de Columna Vertebral (3)

| Herramienta | Parametros | Descripcion |
|------------|------------|-------------|
| `traverse_spine` | `root_id?`, `depth?`, `link_type?` | Recorrer |
| `gravity_check` | `block_id`, `threshold?` | Verificar conectividad |
| `reorder_block` | `block_id`, `new_position`, `parent_id?` | Reordenar |

### 8.4 Herramientas de Estructura (3)

| Herramienta | Parametros | Descripcion |
|------------|------------|-------------|
| `get_section_map` | `root_id` | Obtener jerarquia |
| `detect_gaps` | `section_id` | Detectar faltantes |
| `list_ghosts` | `root_id?` | Listar marcadores |

### 8.5 Herramientas de Sintesis (2)

| Herramienta | Parametros | Descripcion |
|------------|------------|-------------|
| `synthesize` | `structure_id`, `format?`, `template?` | Generar doc |
| `get_toc` | `structure_id` | Obtener TOC |

**Total Herramientas MCP: 15 (100% implementadas)**

---

## 9. API REST

### 9.1 Endpoints

```
Base URL: /api/v1

HEALTH
  GET    /health              Verificacion de salud

BLOCKS
  GET    /blocks              Listar bloques
  GET    /blocks/:id          Obtener bloque
  POST   /blocks              Crear bloque
  PUT    /blocks/:id          Actualizar bloque
  DELETE /blocks/:id          Eliminar bloque
  GET    /blocks/:id/history  Historial del bloque

STRUCTURES
  GET    /structures              Listar estructuras
  GET    /structures/:id          Obtener estructura
  POST   /structures              Crear estructura
  PUT    /structures/:id          Actualizar estructura
  DELETE /structures/:id          Eliminar estructura
  GET    /structures/:id/spine    Obtener columna vertebral

COMMITS
  GET    /commits                  Listar commits
  GET    /commits/:id              Obtener commit
  POST   /commits                  Crear commit
  GET    /commits/:id/diff         Ver diff

VIEWS (REFS)
  GET    /views               Listar vistas
  GET    /views/:name         Obtener vista
  POST   /views               Crear vista
  PUT    /views/:name         Actualizar vista
  DELETE /views/:name         Eliminar vista

WORKINGSET (INDEX)
  GET    /working-set             Obtener conjunto de trabajo
  POST   /working-set/stage       Agregar a preparacion
  POST   /working-set/unstage     Quitar de preparacion
  POST   /working-set/commit      Crear commit desde preparacion
  DELETE /working-set             Descartar

SYNC
  POST   /sync/push           Empujar a remoto
  POST   /sync/pull           Traer de remoto
  POST   /sync/fetch          Obtener metadatos
  GET    /sync/status         Estado de sincronizacion
```

**Estado: ~70% implementada (todos los endpoints principales)**

---

## 10. Arquitectura

### 10.1 Alto Nivel

```
┌─────────────────────────────────────────────────────────────┐
│                     CLI / TUI / MCP / API                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      PKM-AI CORE LIBRARY                     │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐  │
│  │ Block CRUD  │ │ Edge Manager│ │ FractionalIndex     │  │
│  └─────────────┘ └─────────────┘ └─────────────────────┘  │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐  │
│  │ Spine       │ │ Lint Engine │ │ Ghost System        │  │
│  │ Traversal   │ │             │ │                     │  │
│  └─────────────┘ └─────────────┘ └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   SURREALDB DAEMON                           │
│   Unix socket: /tmp/pkmai-surreal.sock                     │
│   Tables: block, edge, commit, view, working_set           │
└─────────────────────────────────────────────────────────────┘
```

### 10.2 Patron Modo Daemon

SurrealDB se ejecuta como un proceso servidor, no embebido, para soportar acceso concurrente multi-proceso.

```rust
pub async fn get_daemon_connection() -> Result<Surreal<Unix>> {
    let socket_path = "/tmp/pkmai-surreal.sock";

    // Intentar conectar al daemon existente
    if Path::new(socket_path).exists() {
        if let Ok(db) = connect_to_socket(socket_path).await {
            if db.health().await.is_ok() {
                return Ok(db);
            }
        }
    }

    // Crear nuevo daemon
    Command::new("surrealdb")
        .args(["start", "--bind", &format!("unix:{}", socket_path), ...])
        .spawn()?;

    wait_for_socket(socket_path, Duration::from_secs(5)).await?;
    connect_to_socket(socket_path).await
}
```

---

## 11. Estado de Implementacion

### 11.1 Cobertura de Pruebas

| Metrica | Valor |
|---------|-------|
| Total de Pruebas | 229 |
| Aprobadas | 229 |
| Fallidas | 0 |

### 11.2 Estado de Modulos

| Modulo | Pruebas | Estado |
|--------|---------|--------|
| FractionalIndex | 3 | ✅ |
| Block Model | 5 | ✅ |
| GhostNode | 5+ | ✅ |
| SmartSection + Bloom | 10 | ✅ |
| GravityHooks | 10 | ✅ |
| Commit | 8 | ✅ |
| View | 10 | ✅ |
| WorkingSet | 12 | ✅ |
| LinkSuggester | 3 | ✅ |
| Synthesis | 5 | ✅ |
| MCP | 16 | ✅ |
| Traverse | 4 | ✅ |
| CLI Create | 11 | ✅ |
| CLI Link | 18 | ✅ |
| Embeddings | 5 | ✅ |
| Delta | 6 | ✅ |

### 11.3 Problemas Conocidos

| Problema | Severidad | Solucion |
|----------|-----------|----------|
| 11 warnings del compilador | Menor | Limpieza antes de v1.0 |
| Validacion de API REST incompleta | Medio | Agregar manejo de errores |
| Centroide ponderado no implementado | Medio | Usar media simple |

---

## Apendice A: Decisiones de Diseno Criticas

### A.1 FractionalIndex sobre f32

Usar `f32` para ordenamiento de secuencias causa degradacion de precision. FractionalIndex con cadenas lexicograficas nunca se degrada.

### A.2 Direccion de section_of

**CORRECTO:** Zettel -> Estructura (contenido apunta al contenedor)
```sql
RELATE block:01HABC1->edge:section_of->block:01HSTRUCT
```

### A.3 traverse_spine Debe Ser Async

```rust
async fn traverse_spine(
    db: &SurrealDb,
    root: Ulid,
    max_depth: usize,
) -> Result<Vec<Block>>
```

---

**Ultima actualizacion:** 2026-03-19
**Fuente canonica:** `src/models/` y este documento