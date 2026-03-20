# Ciclo de Vida de Sesión y Flujo de Trabajo

**Versión:** 1.0.0
**Fecha:** 2026-03-20
**Proyecto:** hodei-pkm
**Estado:** Activo

---

## Tabla de Contenidos

1. [Descripción General](#1-descripción-general)
2. [Ciclo de Vida de Sesión](#2-ciclo-de-vida-de-sesión)
3. [Flujo de Trabajo Estilo Git](#3-flujo-de-trabajo-estilo-git)
4. [Flujo de Trabajo Zettelkasten](#4-flujo-de-trabajo-zettelkasten)
5. [Flujo de Trabajo Asistido por IA](#5-flujo-de-trabajo-asistido-por-ia)
6. [Transiciones de Tipos de Bloque](#6-transiciones-de-tipos-de-bloque)
7. [Diagramas ASCII](#7-diagramas-ascii)
8. [Mejores Prácticas](#8-mejores-prácticas)
9. [Ejemplos](#9-ejemplos)

---

## 1. Descripción General

Este documento describe el ciclo de vida de sesión y los flujos de trabajo en hodei-pkm, un sistema de gestión de conocimiento personal con semántica de versionado estilo Git. El sistema combina la metodología Zettelkasten con la gestión de grafos de conocimiento asistida por IA.

### 1.1 Conceptos Fundamentales

| Concepto | Descripción |
|---------|-------------|
| **Bloque** | Unidad atómica de conocimiento (como un blob en Git) |
| **WorkingSet** | Área de preparación para cambios pendientes (como el índice de Git) |
| **Commit** | Instantánea inmutable del estado del conocimiento |
| **Vista** | Puntero nominativo a commits (rama/etiqueta) |
| **Estructura** | Colección ordenada de bloques (como un árbol en Git) |

### 1.2 Principios de Diseño

```
┌─────────────────────────────────────────────────────────────────┐
│                    PRINCIPIOS DE DISEÑO                          │
├─────────────────────────────────────────────────────────────────┤
│  1. Todo es un Commit        - Instantáneas atómicas de conocimiento│
│  2. Las Ramas son Vistas    - Punteros nominativos a commits  │
│  3. El Índice es WorkingSet - Área de preparación de cambios  │
│  4. Distribuido es Normal   - Cada agente tiene estado local  │
│  5. ULID para Identificación- Identificadores únicos ordenados │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Ciclo de Vida de Sesión

Una **Sesión** representa un período continuo de trabajo de conocimiento. Rastrea la evolución de ideas desde la captura hasta el refinamiento y el almacenamiento permanente.

### 2.1 Estados de Sesión

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   ACTIVA    │───▶│  PREPARANDO  │───▶│  CONFIRMADO │───▶│  ARCHIVADO  │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
      │                  │                  │                  │
      │                  │                  │                  │
      ▼                  ▼                  ▼                  ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  Creando    │    │  Revisando   │    │  Instantánea│    │  Referencia │
│  bloques    │    │  cambios     │    │  guardada   │    │  Histórica  │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

### 2.2 Flujo del Ciclo de Vida de Sesión

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                         CICLO DE VIDA DE SESIÓN                              │
└──────────────────────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────────────┐
    │                         NUEVA SESIÓN                                  │
    │  ┌───────────────────────────────────────────────────────────────┐  │
    │  │  1. Crear WorkingSet (área de preparación)                    │  │
    │  │  2. Cargar WorkingSet ID (basado en ULID)                    │  │
    │  │  3. Inicializar log de operaciones                            │  │
    │  └───────────────────────────────────────────────────────────────┘  │
    └─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
    ┌─────────────────────────────────────────────────────────────────────┐
    │                         TRABAJO EN PROGRESO                           │
    │  ┌───────────────────────────────────────────────────────────────┐  │
    │  │  • Crear/editar/eliminar bloques                             │  │
    │  │  • Cada cambio genera un Delta (BlockDelta/EdgeDelta)        │  │
    │  │  • Deltas preparados en WorkingSet                          │  │
    │  │  • Log de operaciones registra todos los cambios             │  │
    │  └───────────────────────────────────────────────────────────────┘  │
    └─────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┴───────────────┐
                    │                               │
                    ▼                               ▼
    ┌───────────────────────────┐    ┌───────────────────────────────┐
    │     STASH (opcional)      │    │         COMMIT                │
    │  ┌─────────────────────┐  │    │  ┌─────────────────────────┐  │
    │  │  save_state()       │  │    │  │  create_commit()       │  │
    │  │  - Pausar sesión    │  │    │  │  - Instantánea de estructura│  │
    │  │  - Mantener WorkingSet│  │    │  │  - Registrar autor     │  │
    │  │  - Puede reanudarse  │  │    │  │  - Añadir mensaje     │  │
    │  └─────────────────────┘  │    │  │  - Limpiar WorkingSet │  │
    └───────────────────────────┘    │  └─────────────────────────┘  │
                                     └───────────────────────────────┘
                                                        │
                                                        ▼
                                     ┌───────────────────────────────┐
                                     │      ACTUALIZAR VISTA (HEAD)  │
                                     │  ┌─────────────────────────┐  │
                                     │  │ Vista apunta ahora al    │  │
                                     │  │ nuevo commit             │  │
                                     │  └─────────────────────────┘  │
                                     └───────────────────────────────┘
```

### 2.3 Estructuras de Datos de Sesión

#### WorkingSet (Área de Preparación)

```rust
pub struct WorkingSet {
    pub id: WorkingSetId,              // Identificador único basado en ULID
    pub author: AgentId,               // Agente propietario de este working set
    pub staged_blocks: BTreeMap<Ulid, BlockDelta>,   // Cambios de bloques pendientes
    pub staged_edges: BTreeMap<(Ulid, Ulid), EdgeDelta>, // Cambios de aristas pendientes
    pub removed_blocks: Vec<Ulid>,      // Bloques marcados para eliminación
    pub removed_edges: Vec<(Ulid, Ulid)>, // Aristas marcadas para eliminación
    pub operations: Vec<Operation>,     // Log de operaciones para reproducción/revertir
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### Entrada del Log de Operaciones

```rust
pub struct Operation {
    pub id: Ulid,                       // Identificador único de operación
    pub delta: OperationDelta,         // El delta aplicado
    pub timestamp: DateTime<Utc>,       // Cuándo se registró
}

pub enum OperationDelta {
    Block(BlockDelta),
    Edge(EdgeDelta),
}
```

---

## 3. Flujo de Trabajo Estilo Git

El PKM sigue un flujo de trabajo inspirado en Git para la gestión del conocimiento.

### 3.1 Diagrama de Estados

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         ESTADOS DEL CONOCIMIENTO                            │
└─────────────────────────────────────────────────────────────────────────────┘

                         ┌──────────────────┐
                         │      BORRADOR    │
                         │  (memoria/efímero)│
                         └────────┬─────────┘
                                  │ create_block()
                                  │ with_content()
                                  ▼
                         ┌──────────────────┐
          ┌──────────────│     PREPARADO     │──────────────┐
          │              │  (WorkingSet)    │              │
          │              └────────┬─────────┘              │
          │                       │ stage()                │
          │                       ▼                        │
          │              ┌──────────────────┐               │
          │              │    CONFIRMADO    │               │
          │              │  (Repositorio)   │               │
          │              └────────┬─────────┘               │
          │                       │                         │
          │    ┌────────────────────┼────────────────────┐   │
          │    │                    │                    │   │
          ▼    ▼                    ▼                    ▼   ▼
   ┌──────────────┐      ┌──────────────┐      ┌──────────────┐
   │   DESCARTADO  │      │    FUSIONADO │      │   RAMIFICADO │
   │  (unprepare)  │      │  (síntesis)  │      │   (vista)   │
   └──────────────┘      └──────────────┘      └──────────────┘
```

### 3.2 Comandos del Flujo de Trabajo

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         COMANDOS EQUIVALENTES                               │
├──────────────────────┬──────────────────────────────────────────────────────┤
│       Git           │                    PKM                                 │
├──────────────────────┼──────────────────────────────────────────────────────┤
│  git init           │  pkm init                                             │
│  git status         │  pkm status                                           │
│  git add <archivo>  │  pkm stage <block-id>                                 │
│  git reset <archivo>│  pkm unstage <block-id>                               │
│  git diff --cached  │  pkm diff --staged                                    │
│  git diff           │  pkm diff                                             │
│  git commit         │  pkm commit "mensaje"                                 │
│  git branch         │  pkm view list                                        │
│  git checkout -b    │  pkm view create <nombre>                            │
│  git checkout       │  pkm view switch <nombre>                             │
│  git tag            │  pkm tag create <nombre>                              │
│  git log            │  pkm log                                              │
│  git stash          │  pkm stash                                            │
│  git stash pop      │  pkm stash pop                                       │
│  git merge          │  pkm merge <vista>                                    │
└──────────────────────┴──────────────────────────────────────────────────────┘
```

### 3.3 Estados Detallados del Flujo de Trabajo

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         FLUJO DE TRABAJO DETALLADO                           │
└─────────────────────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────────────────┐
    │  PASO 1: CREAR (Estado Borrador)                                        │
    │                                                                         │
    │  ┌──────────────────────────────────────────────────────────────────┐   │
    │  │  block = Block::fleeting("Captura de idea en bruto")              │   │
    │  │  block.stage()  ──▶ WorkingSet.staged_blocks[block.id]            │   │
    │  └──────────────────────────────────────────────────────────────────┘   │
    └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
    ┌─────────────────────────────────────────────────────────────────────────┐
    │  PASO 2: PREPARAR (Estado Preparado)                                    │
    │                                                                         │
    │  ┌──────────────────────────────────────────────────────────────────┐   │
    │  │  WorkingSet.stage_block(BlockDelta::Created {                     │   │
    │  │      block_id: block.id,                                         │   │
    │  │      title: block.title,                                         │   │
    │  │      content: block.content,                                     │   │
    │  │      block_type: block.block_type,                               │   │
    │  │  })                                                               │   │
    │  │                                                                   │   │
    │  │  # Deltas registrados en log de operaciones                       │   │
    │  │  # Puede deshacerse con unstage_block(block_id)                   │   │
    │  └──────────────────────────────────────────────────────────────────┘   │
    └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
    ┌─────────────────────────────────────────────────────────────────────────┐
    │  PASO 3: CONFIRMAR (Estado Confirmado)                                  │
    │                                                                         │
    │  ┌──────────────────────────────────────────────────────────────────┐   │
    │  │  commit = repo.commit(                                            │   │
    │  │      working_set,                                                 │   │
    │  │      "Añadir nota efímera sobre async de Rust"                    │   │
    │  │  )                                                                 │   │
    │  │                                                                   │   │
    │  │  # Crea StructureSnapshot                                          │   │
    │  │  # Registra commits padre                                         │   │
    │  │  # Actualiza Vista (HEAD)                                         │   │
    │  └──────────────────────────────────────────────────────────────────┘   │
    └─────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Flujo de Trabajo Zettelkasten

La metodología Zettelkasten se implementa a través de transiciones de tipos de bloque.

### 4.1 Tipos de Bloque

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       TAXONOMÍA DE TIPOS DE BLOQUE                           │
└─────────────────────────────────────────────────────────────────────────────┘

                    ┌─────────────────────┐
                    │   ENTRADA (Captura)  │
                    └──────────┬────────────┘
                               │
              ┌────────────────┼────────────────┐
              ▼                ▼                ▼
    ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
    │    EFÍMERA      │ │   LITERATURA    │ │    REFERENCIA    │
    │  (ideas en bruto)│ │ (notas de fuente)│ │ (refs externas) │
    └────────┬────────┘ └────────┬────────┘ └─────────────────┘
             │                   │
             │   process()       │   process()
             ▼                   ▼
    ┌─────────────────┐ ┌─────────────────┐
    │   PERMANENTE    │ │    PERMANENTE    │
    │  (cristalizado) │ │   (sintetizado)  │
    └────────┬────────┘ └────────┬────────┘
             │                   │
             └─────────┬─────────┘
                       │
                       ▼
             ┌─────────────────┐
             │    ESTRUCTURA    │
             │   (MOC/Índice)  │
             └─────────────────┘
```

### 4.2 Definiciones de Tipos de Bloque

| Tipo | Propósito | Características |
|------|---------|-----------------|
| **Efímera** | Captura rápida | Ideas en bruto, sin procesar |
| **Literatura** | Notas de fuente | Citas, resúmenes |
| **Permanente** | Conocimiento atómico | Denso, enlazado, independiente |
| **Estructura** | Índice/MOC | Puntos de entrada, agrupaciones |
| **Hub** | Entrada de dominio | Anclas de tema |
| **Tarea** | Acciones | Elementos ejecutables |
| **Referencia** | Enlaces externos | Citas, URLs |
| **Esquema** | Esqueleto de documento | Estructuras TOC |
| **Fantasma** | Marcador predictivo | Conceptos futuros |

### 4.3 Flujo Zettelkasten

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      FLUJO DE TRABAJO ZETTELKASTEN                           │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  NOTAS EFÍMERAS (Bandeja de entrada)                                          │
│  ─────────────────────────────────                                           │
│  • Captura rápida sin estructura                                             │
│  • Ideas en bruto capturadas por voz o escritura rápida                      │
│  • Con marca temporal ULID                                                    │
│                                                                             │
│  Ejemplo:                                                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  Efímera: "Quizás async/await en Rust podría simplificar el patrón  │    │
│  │            de worker mencionado en la arquitectura Nexus"           │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ process()
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  NOTAS DE LITERATURA (Procesamiento)                                          │
│  ────────────────────────────────────                                        │
│  • Expandir y clarificar notas efímeras                                      │
│  • Añadir contexto de fuentes externas                                       │
│  • Mantener enlace al material fuente                                        │
│                                                                             │
│  Ejemplo:                                                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  Literatura: "Rust async/await simplifica los patrones de workers     │    │
│  │              concurrentes                                                                     │    │
│  │              - Runtime Tokio para tareas async                       │    │
│  │              - Estado compartido vía Arc<Mutex<T>>                    │    │
│  │              - Fuente: Rust Async Book Capítulo 3                    │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ synthesize()
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  NOTAS PERMANENTES (Conocimiento)                                            │
│  ───────────────────────────────                                             │
│  • Unidades de conocimiento atómicas, autocontenidas                        │
│  • Enlazadas a otras notas permanentes mediante aristas                      │
│  • Con palabras propias, comprensión sintetizada                             │
│                                                                             │
│  Ejemplo:                                                                   │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  Permanente: "Patrón de Worker Async en Rust                        │    │
│  │                                                                          │    │
│  │  La sintaxis async/await en Rust permite un patrón de worker limpio  │    │
│  │  donde Tokio gestiona la planificación. Idea clave: paso de mensajes │    │
│  │  acotado con canales previene el agotamiento de recursos.           │    │
│  │                                                                          │    │
│  │  Enlaces: → Patrón de Worker Async de Rust (self)                    │    │
│  │          → Modelo Actor Nexus (relacionado)                          │    │
│  │          → Rust Async Book (fuente)                                   │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ organize()
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  NOTAS DE ESTRUCTURA (MOC/Índice)                                            │
│  ─────────────────────────────────                                         │
│  • Hub para notas permanentes relacionadas                                  │
│  • Proporciona puntos de entrada de navegación                              │
│  • Mantiene visión general del área de tema                                  │
│                                                                             │
│  Ejemplo:                                                                   │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  Estructura: "Patrones de Concurrencia en Rust"                      │    │
│  │                                                                          │    │
│  │  ## Índice                                                            │    │
│  │  • [[Patrón de Worker Async]]      - Patrón listo para producción     │    │
│  │  • [[Modelo Actor]]               - Base de arquitectura Nexus     │    │
│  │  • [[Estructuras de Datos Lock-Free]]  - Optimización avanzada     │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 5. Flujo de Trabajo Asistido por IA

El subsistema de IA mejora la gestión del conocimiento mediante embeddings, sugerencias de enlaces y detección de fantasmas.

### 5.1 Puntos de Integración de IA

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                  FLUJO DE TRABAJO ASISTIDO POR IA                            │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                           CAPTURA                                            │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  Asistencia de IA:                                                      │  │
│  │  • Auto-clasificar tipo de bloque según contenido                     │  │
│  │  • Sugerir etiquetas iniciales                                         │  │
│  │  • Detectar ideas duplicadas                                           │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          PROCESAR                                           │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  Asistencia de IA:                                                      │  │
│  │  • Agrupación semántica (agrupar bloques relacionados)                  │  │
│  │  • Búsqueda por similitud basada en embeddings                         │  │
│  │  • Sugerencia de enlaces (encontrar bloques relacionados)              │  │
│  │  • Detección de fantasmas (identificar marcadores predictivos)        │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        SINTETIZAR                                           │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  Asistencia de IA:                                                      │  │
│  │  • Generación de estructura (crear MOC desde bloques relacionados)   │  │
│  │  • Generación de resúmenes para notas de literatura                   │  │
│  │  • Puntuación de confianza para contenido generado por IA             │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        RECUPERAR                                            │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  Asistencia de IA:                                                      │  │
│  │  • Búsqueda semántica (consultas en lenguaje natural)                  │  │
│  │  • Recuperación consciente del contexto                               │  │
│  │  • Sugerencias de recorrido de grafo                                  │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 Capacidades de IA

| Componente | Función | Descripción |
|-----------|----------|-------------|
| `Embeddings` | Vectores semánticos | Generar embeddings para bloques |
| `LinkSuggester` | Descubrimiento de relaciones | Encontrar potenciales enlaces entre bloques |
| `SemanticClustering` | Agrupación de temas | Agrupar bloques por similitud semántica |
| `GhostDetector` | Identificación de marcadores | Detectar bloques incompletos o predictivos |
| `StructureGenerator` | Creación de MOC | Generar notas de estructura desde bloques relacionados |

---

## 6. Transiciones de Tipos de Bloque

### 6.1 Matriz de Transición

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    TRANSICIONES DE TIPOS DE BLOQUE                           │
└─────────────────────────────────────────────────────────────────────────────┘

    DESDE \ A    │ Efímera │ Literatura │ Permanente │ Estructura │ Hub │ Tarea │
    ───────────┼──────────┼────────────┼───────────┼───────────┼─────┼──────│
    Efímera    │    ─     │    ✓       │    ✓      │     ✓     │  ✓  │  ✓   │
    Literatura │    ✗     │    ─       │    ✓      │     ✓     │  ✓  │  ✓   │
    Permanente │    ✗     │    ✗       │    ─      │     ✓     │  ✓  │  ✓   │
    Estructura │    ✗     │    ✗       │    ✗      │     ─     │  ✗  │  ✗   │
    Hub        │    ✗     │    ✗       │    ✗      │     ✓     │  ✗  │  ✗   │
    Fantasma   │    ✓     │    ✓       │    ✓      │     ✓     │  ✓  │  ✓   │

    Leyenda: ✓ = transición permitida, ✗ = no permitida, ─ = mismo tipo
```

### 6.2 Reglas de Transición

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         REGLAS DE TRANSICIÓN                                 │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  REGLA 1: Efímera → Literatura                                              │
│  ─────────────────────────────────                                          │
│  Condición: Usuario expande idea con cita de fuente                         │
│  Acción:    Convertir bloque, añadir metadatos de fuente                    │
│  Ejemplo:   "Rust es rápido" → "Rust es rápido (Fuente: rust-lang.org)"   │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  REGLA 2: Efímera/Literatura → Permanente                                   │
│  ─────────────────────────────────────────                                  │
│  Condición: Conocimiento sintetizado, atómico, autocontenido               │
│  Acción:    Convertir a Permanente, crear enlaces a bloques relacionados    │
│  Validación: Debe tener al menos una arista saliente                        │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  REGLA 3: Cualquiera → Fantasma                                            │
│  ───────────────────                                                        │
│  Condición: Contenido de bloque es especulativo o marcador                 │
│  Acción:    Convertir a Fantasma, marcar como predictivo                   │
│  Ejemplo:   "Futuro: Soporte multi-región" → Bloque Fantasma               │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  REGLA 4: Múltiples Permanentes → Estructura                                │
│  ────────────────────────────────────────                                   │
│  Condición: Indexando notas permanentes relacionadas                        │
│  Acción:    Crear Bloque Estructura con enlaces a bloques relacionados      │
│  Asistencia IA: StructureGenerator puede sugerir estructura desde clusters   │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 6.3 Tipos de Delta para Transiciones

```rust
// Variantes de BlockDelta para transiciones de tipo
pub enum BlockDelta {
    Created { block_id, title, content, block_type },
    Modified { block_id, old_title, new_title, old_content, new_content },
    Deleted { block_id, title },
    Reorganized { block_id, old_predecessor, new_predecessor },
    TypeChanged { block_id, old_type, new_type },  // Para transiciones de tipo
    TagAdded { block_id, tag },
    TagRemoved { block_id, tag },
}
```

---

## 7. Diagramas ASCII

### 7.1 Diagrama Completo del Flujo de Trabajo PKM

```
┌─────────────────────────────────────────────────────────────────────────────┐
│               DIAGRAMA COMPLETO DEL FLUJO DE TRABAJO PKM                     │
└─────────────────────────────────────────────────────────────────────────────┘

    ╔═══════════════════════════════════════════════════════════════════════╗
    ║                         ENTRADA DEL USUARIO                             ║
    ╠═══════════════════════════════════════════════════════════════════════╣
    ║                                                                        ║
    ║   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                ║
    ║   │  Efímera    │    │  Literatura │    │  Referencia  │                ║
    ║   │   Nota      │    │    Nota     │    │   Nota      │                ║
    ║   └──────┬──────┘    └──────┬──────┘    └──────┬──────┘                ║
    ║          │                  │                  │                       ║
    ║          │ process()        │ process()        │ link()                ║
    ║          ▼                  ▼                  ▼                       ║
    ║   ┌──────────────────────────────────────────────────────┐               ║
    ║   │              NOTAS PERMANENTES                        │               ║
    ║   │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  │               ║
    ║   │  │ Bloque 1│  │ Bloque 2│  │ Bloque 3│  │ Bloque N│  │               ║
    ║   │  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘  │               ║
    ║   │       │    enlaces  │    enlaces  │    enlaces  │        │               ║
    ║   │       └────────────┼────────────┼────────────┘        │               ║
    ║   │                    │            │                    │               ║
    ║   │              ┌──────┴────────────┴──────┐              │               ║
    ║   │              │                         │              │               ║
    ║   │              ▼                         ▼              │               ║
    ║   │       ┌───────────┐             ┌───────────┐           │               ║
    ║   │       │   Hub     │             │ Estructura │           │               ║
    ║   │       │ (dominio) │             │   (MOC)   │           │               ║
    ║   │       └───────────┘             └───────────┘           │               ║
    ║   └──────────────────────────────────────────────────────┘               ║
    ║                               │                                         ║
    ║                               │ synthesize()                            ║
    ║                               ▼                                         ║
    ║   ┌──────────────────────────────────────────────────────┐               ║
    ║   │                 SALIDA DE SÍNTESIS                   │               ║
    ║   │  • Exportaciones de documento  • Renderizados Typst │               ║
    ║   │  • Visualizaciones de grafo    • Mapas de conocimiento│               ║
    ║   └──────────────────────────────────────────────────────┘               ║
    ║                                                                       ║
    ╚═══════════════════════════════════════════════════════════════════════╝
```

### 7.2 Flujo de Control de Versionado

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    FLUJO DE CONTROL DE VERSIONADO                            │
└─────────────────────────────────────────────────────────────────────────────┘

        ┌─────────────────────────────────────────────────────────────────┐
        │                     DIRECTORIO DE TRABAJO                        │
        │  ┌─────────────────────────────────────────────────────────┐    │
        │  │  Bloques en memoria / siendo editados                   │    │
        │  │  • Creados vía Block::fleeting(), Block::permanent()     │    │
        │  │  • Modificados vía block.with_content(), block.with_tag()│    │
        │  └─────────────────────────┬───────────────────────────────┘    │
        └────────────────────────────┼────────────────────────────────────┘
                                     │ stage()
                                     ▼
        ┌─────────────────────────────────────────────────────────────────┐
        │                         WORKING SET                             │
        │  ┌─────────────────────────────────────────────────────────┐    │
        │  │  CAMBIOS PREPARADOS                                      │    │
        │  │  ┌────────────────┐  ┌────────────────┐                │    │
        │  │  │ BlockDelta    │  │ EdgeDelta     │                │    │
        │  │  │ - Creado      │  │ - Creado      │                │    │
        │  │  │ - Modificado  │  │ - Eliminado  │                │    │
        │  │  │ - Eliminado   │  │               │                │    │
        │  │  └────────────────┘  └────────────────┘                │    │
        │  │                                                        │    │
        │  │  PENDIENTES DE ELIMINACIÓN                              │    │
        │  │  • removed_blocks: Vec<Ulid>                           │    │
        │  │  • removed_edges: Vec<(Ulid, Ulid)>                    │    │
        │  │                                                        │    │
        │  │  LOG DE OPERACIONES                                     │    │
        │  │  • Vec<Operation> para reproducción/revertir           │    │
        │  └─────────────────────────────────────────────────────────┘    │
        └────────────────────────────┬────────────────────────────────────┘
                                     │ commit()
                                     ▼
        ┌─────────────────────────────────────────────────────────────────┐
        │                           COMMIT                               │
        │  ┌─────────────────────────────────────────────────────────┐    │
        │  │  Commit {                                                │    │
        │  │    id: CommitId(Ulid),                                   │    │
        │  │    structure_snapshot: StructureSnapshot,                │    │
        │  │    parents: Vec<CommitId>,                                │    │
        │  │    author: AgentId,                                      │    │
        │  │    message: String,                                       │    │
        │  │    created_at: DateTime,                                  │    │
        │  │    blocks_added/modified/removed: Vec<Ulid>,            │    │
        │  │  }                                                        │    │
        │  └─────────────────────────┬───────────────────────────────┘    │
        └────────────────────────────┼────────────────────────────────────┘
                                     │ actualizar Vista
                                     ▼
        ┌─────────────────────────────────────────────────────────────────┐
        │                         REPOSITORIO                             │
        │  ┌─────────────────────────────────────────────────────────┐    │
        │  │  VISTAS (Ramas/Etiquetas)                                 │    │
        │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐                │    │
        │  │  │ main     │  │ work     │  │ v1.0.0  │ (etiqueta)    │    │
        │  │  │   ↓      │  │   ↓     │  │   ↓     │                │    │
        │  │  │ Commit   │  │ Commit  │  │ Commit  │                │    │
        │  │  └──────────┘  └──────────┘  └──────────┘                │    │
        │  │                                                        │    │
        │  │  COMMITS (Cadena)                                       │    │
        │  │  ●────●────●────●────● (HEAD/main)                      │    │
        │  │                                                                    │    │
        │  │  INSTANTÁNEA DE ESTRUCTURA                                 │    │
        │  │  • block_order: Vec<Ulid> (orden FOLLOWZETTEL)           │    │
        │  │  • edges: Vec<EdgeSnapshot>                              │    │
        │  └─────────────────────────────────────────────────────────┘    │
        └─────────────────────────────────────────────────────────────────┘
```

### 7.3 Ciclo de Sesión

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          CICLO DE SESIÓN                                     │
└─────────────────────────────────────────────────────────────────────────────┘

                              ┌───────────────────┐
                              │  INICIO DE SESIÓN │
                              │ Crear WorkingSet  │
                              └─────────┬─────────┘
                                        │
                                        ▼
        ┌───────────────────────────────────────────────────────────────────┐
        │                        SESIÓN ACTIVA                              │
        │                                                                    │
        │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐               │
        │   │   CREAR     │  │   EDITAR    │  │   ELIMINAR   │               │
        │   │   BLOQUE    │──│   BLOQUE    │──│   BLOQUE    │               │
        │   └─────────────┘  └─────────────┘  └─────────────┘               │
        │         │                │                │                        │
        │         └────────────────┼────────────────┘                        │
        │                          │                                         │
        │                          ▼                                         │
        │   ┌─────────────────────────────────────────────────────────┐     │
        │   │                   PREPARAR CAMBIOS                       │     │
        │   │  • stage_block(delta)     • stage_edge(delta)           │     │
        │   │  • mark_block_removed(id) • mark_edge_removed(s,t)     │     │
        │   └─────────────────────────────────────────────────────────┘     │
        │                          │                                         │
        │           ┌──────────────┴──────────────┐                         │
        │           │                             │                         │
        │           ▼                             ▼                         │
        │   ┌───────────────┐             ┌───────────────┐                │
        │   │    COMMIT      │             │     STASH     │                │
        │   │  create_commit │             │   save_state  │                │
        │   └───────┬───────┘             └───────────────┘                │
        │           │                                                          │
        │           ▼                                                          │
        │   ┌───────────────┐                                                 │
        │   │ ACTUALIZAR VISTA│                                                │
        │   │ (HEAD se mueve) │                                                │
        │   └───────────────┘                                                 │
        │                                                                    │
        └────────────────────────────────────────────────────────────────────┘
                                        │
                                        ▼
                              ┌───────────────────┐
                              │    FIN DE SESIÓN  │
                              │  Cerrar/Archivar  │
                              └───────────────────┘
```

---

## 8. Mejores Prácticas

### 8.1 Gestión de Sesiones

| Práctica | Descripción |
|----------|-------------|
| **Commits Atómicos** | Agrupar cambios relacionados en un solo commit con mensaje descriptivo |
| **Preparación Frecuente** | Preparar cambios regularmente para evitar perder trabajo |
| **Disciplina de Vistas** | Mantener vista main limpia; usar vistas de característica para trabajo experimental |
| **Mensajes Significativos** | Escribir mensajes de commit que expliquen "por qué", no solo "qué" |

### 8.2 Creación de Bloques

| Práctica | Descripción |
|----------|-------------|
| **Una Idea Por Bloque** | Mantener bloques atómicos; si el contenido crece más de 2-3 párrafos, considerar dividir |
| **Usar Timestamps ULID** | Aprovechar el orden temporal de ULID para clasificación cronológica automática |
| **Enlazar Generosamente** | Crear aristas entre bloques relacionados para construir el grafo de conocimiento |
| **Etiquetar Consistentemente** | Establecer un vocabulario controlado para etiquetas |

### 8.3 Principios Zettelkasten

| Práctica | Descripción |
|----------|-------------|
| **Procesar Notas Efímeras** | Revisar y procesar notas efímeras dentro de 24-48 horas |
| **Escribir con Palabras Propias** | Las notas de literatura deben estar sintetizadas, no copiadas |
| **Permanente = Enlazado** | Las notas permanentes deben tener aristas salientes hacia otras notas |
| **Construir Estructuras al Final** | Crear bloques MOC/Hub después de acumular notas permanentes relacionadas |

### 8.4 Control de Versiones

| Práctica | Descripción |
|----------|-------------|
| **Commit Temprano, Commit Frecuente** | Commits pequeños son más fáciles de entender y revertir |
| **Referenciar Contexto Externo** | Incluir enlaces a issues, PRs o documentos en mensajes de commit |
| **Rama para Trabajo Mayor** | Usar vistas para reorganizaciones mayores o restructuración experimental |
| **Etiquetar Hitos** | Etiquetar estados de conocimiento significativos (ej: "curso-completado-2026") |

---

## 9. Ejemplos

### 9.1 Ejemplo Completo de Flujo de Trabajo

```rust
use hodei_pkm::{
    versioning::{WorkingSet, Commit, View},
    models::{Block, BlockType},
    ai::{Embeddings, LinkSuggester},
};

// 1. Iniciar una nueva sesión
let author = AgentId::new("investigador");
let mut working_set = WorkingSet::new(author.clone());

// 2. Crear nota efímera
let mut fleeting = Block::fleeting(
    "Los patrones async de Rust podrían simplificar el diseño de workers Nexus"
);
fleeting = fleeting.with_tag("rust").with_tag("async");

// 3. Preparar el bloque
let delta = BlockDelta::Created {
    block_id: fleeting.id,
    title: fleeting.title.clone(),
    content: fleeting.content.clone(),
    block_type: "fleeting".to_string(),
};
working_set.stage_block(delta);

println!("Preparados {} bloque(s)", working_set.staged_blocks_count());

// 4. Procesar: expandir a nota de literatura
let literature = Block::new(BlockType::Literature, "Patrones de Workers Async en Rust");
let literature_content = r#"
De: Rust Async Book

Patrones clave:
1. Runtime Tokio para tareas async
2. Estado compartido vía Arc<Mutex<T>>
3. Canales para paso de mensajes
4. Canales acotados previenen agotamiento de recursos

Ver también: Modelo Actor en arquitectura Nexus
"#;
let literature = literature.with_content(literature_content);

// 5. Preparar nota de literatura
working_set.stage_block(BlockDelta::Created {
    block_id: literature.id,
    title: literature.title.clone(),
    content: literature.content.clone(),
    block_type: "literature".to_string(),
});

// 6. Preparar la arista que enlaza literatura con efímera
working_set.stage_edge(EdgeDelta::Created {
    source: literature.id,
    target: fleeting.id,
    relation: "processes".to_string(),
});

// 7. Confirmar la sesión
let commit = Commit::new(
    StructureSnapshot {
        id: Ulid::new(),
        block_order: vec![fleeting.id, literature.id],
        edges: vec![
            EdgeSnapshot {
                source: literature.id,
                target: fleeting.id,
                relation: "processes".to_string(),
            }
        ],
    },
    author.clone(),
    "Procesar nota efímera a nota de literatura".to_string(),
    Vec::new(),
    vec![fleeting.id, literature.id],
    Vec::new(),
    Vec::new(),
);

// 8. Actualizar vista main
let main_view = View::branch_head("main", commit.id);
```

### 9.2 Ejemplo de Progresión Zettelkasten

```rust
// PASO 1: Efímera (captura)
let fleeting_note = Block::fleeting(
    "Patrones de supervisión de agentes en sistemas distribuidos"
);

// PASO 2: Literatura (expandir con fuentes)
let literature_note = Block::new(BlockType::Literature, "Patrones de Supervisión de Agentes");
let literature_content = r#"
Conceptos clave de "Designing Distributed Systems" de Brendan Burns:

ÁRBOLES DE SUPERVISIÓN:
- Uno-por-uno: Reiniciar solo el hijo fallido
- Uno-por-todos: Reiniciar todos los hijos si uno falla
- Estrategias personalizadas para modos de fallo específicos

APLICACIÓN NEXUS:
La arquitectura Nexus usa supervisión para el ciclo de vida de workers:
- SensorWorker supervisado por hilo principal
- LogicWorker supervisado independientemente
- RenderWorker tiene su propio dominio de recuperación

Fuente: Designing Distributed Systems, Brendan Burns
"#;
let literature = literature_note.with_content(literature_content);

// PASO 3: Permanente (sintetizar conocimiento atómico)
let permanent_note = Block::permanent(
    "Patrón de Supervisión de Agentes Nexus",
    r#"
La arquitectura WASM Nexus implementa supervisión jerárquica:

SUPERVISIÓN EN RUNTIME:
Hilo Principal (React UI)
    └── SensorWorker (actor)
    └── LogicWorker (actor)
    └── RenderWorker (actor)

POLÍTICA DE SUPERVISIÓN:
- Monitoreo de heartbeats (control-plane.sab)
- Recuperación condicionada por época
- Semántica de reemplazo-al-caer
- Sin estado compartido; SAB como única fuente de verdad

Esto permite:
- Aislamiento de fallos entre workers
- Recuperación independiente sin reinicio total del sistema
- Comportamiento de reinicio determinista

Relacionado: [[Modelo Actor]], [[Plano de Control Nexus]]
"#
).with_tag("nexus").with_tag("arquitectura").with_tag("tolerancia-a-fallos");

// PASO 4: Estructura (crear MOC)
let structure_note = Block::structure("Índice de Arquitectura Nexus");
let structure_content = r#"
# Temas de Arquitectura Nexus

## Conceptos Fundamentales
- [[Patrón de Supervisión de Agentes Nexus]] - Gestión de ciclo de vida de workers
- [[Plano de Control Nexus]] - Sincronización basada en SAB
- [[Modelo Actor Nexus]] - Semántica de actor sobre memoria compartida

## Patrones
- [[Determinismo Zarfiano]] - Ordenamiento por especificidad de reglas
- [[Principio Fantasma]] - Recuperación de contexto efímero de worker

## Implementación
- [[Arquitectura Nexus Draw]] - Aplicación de referencia
- [[Workers WASM Nexus]] - Implementación de workers
"#;
let moc = structure_note.with_content(structure_content);
```

### 9.3 Ejemplo de Stash/Reanudación de Sesión

```rust
// INTERRUPCIÓN: Cambiar contexto a mitad de sesión
let stash_id = repo.stash_working_set(working_set, "WIP: investigando supervisión")?;
println!("Working set guardado en stash: {}", stash_id);

// ... más tarde, reanudar trabajo ...

let (resumed_working_set, stash) = repo.pop_stash(stash_id)?;
println!("Stash reanudado: {}", stash.message());
// Continuar editando...
```

---

## Apéndice: Referencia de Comandos

| Comando CLI | Descripción |
|-------------|-------------|
| `pkm init` | Inicializar nuevo repositorio PKM |
| `pkm status` | Mostrar estado del working set |
| `pkm stage <id>` | Preparar bloque para commit |
| `pkm unstage <id>` | Quitar de preparación |
| `pkm diff [--staged]` | Mostrar cambios |
| `pkm commit <msg>` | Confirmar cambios preparados |
| `pkm log [--graph]` | Mostrar historial de commits |
| `pkm view list` | Listar todas las vistas |
| `pkm view create <nombre>` | Crear nueva vista |
| `pkm view switch <nombre>` | Cambiar HEAD a vista |
| `pkm tag create <nombre>` | Crear etiqueta anotada |
| `pkm merge <vista>` | Fusionar vista en actual |
| `pkm stash` | Guardar working set actual en stash |
| `pkm stash pop` | Restaurar working set del stash |

---

*Versión del documento: 1.0.0 | Última actualización: 2026-03-20*
