# Guía del Flujo de Trabajo Inteligente MCP

## Tabla de Contenidos

1. [Visión General](#visión-general)
2. [Filosofía Central: Flujo de Trabajo Estilo Git](#filosofía-central-flujo-de-trabajo-estilo-git)
3. [Flujo de Trabajo Zettelkasten](#flujo-de-trabajo-zettelkasten)
4. [Referencia Completa de Herramientas MCP](#referencia-completa-de-herramientas-mcp)
   - [Operaciones de Conocimiento](#operaciones-de-conocimiento)
   - [Operaciones de Enlaces](#operaciones-de-enlaces)
   - [Operaciones de Estructura](#operaciones-de-estructura)
   - [Operaciones de Staging y Commit](#operaciones-de-staging-y-commit)
5. [Tipos de Bloques](#tipos-de-bloques)
6. [Datos de Enriquecimiento](#datos-de-enriquecimiento)
7. [Principios de Diseño](#principios-de-diseño)

---

## Visión General

El servidor MCP proporciona a los agentes de IA un flujo de trabajo inteligente para la captura de conocimiento que refleja la filosofía de Git: nada se confirma hasta que esté realmente listo. Este enfoque permite a los agentes de IA tomar decisiones informadas sobre la estructura del grafo de conocimiento antes de finalizar los cambios.

---

## Filosofía Central: Flujo de Trabajo Estilo Git

```
┌─────────────────────────────────────────────────────────────┐
│                 Ciclo de Vida del Conocimiento              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   ┌──────────┐    ┌──────────┐    ┌──────────┐             │
│   │  Bloque  │───▶│  Stage   │───▶│  Commit  │             │
│   │ Creado   │    │ (Working │    │ (Snap-   │             │
│   │          │    │   Set)   │    │  shot)   │             │
│   └──────────┘    └──────────┘    └──────────┘             │
│        │               │               │                    │
│        ▼               ▼               ▼                    │
│   ┌──────────────────────────────────────────┐             │
│   │  Análisis IA: Sugerencias de enlaces,    │             │
│   │  Gravedad, Detección de tipo, Tags      │             │
│   └──────────────────────────────────────────┘             │
│                                                              │
│  Principio Clave: Nada se stagea automáticamente.          │
│                   La IA decide cuándo está listo.          │
└─────────────────────────────────────────────────────────────┘
```

---

## Flujo de Trabajo Zettelkasten

El flujo de trabajo MCP implementa el método Zettelkasten para construir un grafo de conocimiento conectado:

### Paso 1: Capturar (Crear Bloque)

Crea notas atómicas con datos de enriquecimiento para enlaces inteligentes:

```json
{
  "name": "create_block",
  "arguments": {
    "block_type": "fleeting",
    "title": "Idea sobre redes neuronales",
    "content": "Las arquitecturas Transformer son potentes...",
    "enrich": true
  }
}
```

### Paso 2: Reflexionar (Analizar Enriquecimiento)

La IA analiza los datos de enriquecimiento devueltos:
- **link_suggestions**: Bloques existentes que se relacionan con este contenido
- **tag_suggestions**: Tags relevantes para categorización
- **gravity_info**: Métricas de conectividad (detección de hubs)
- **type_suggestion**: Recomendación de tipo de bloque

### Paso 3: Conectar (Crear Enlaces)

Enlaza bloques relacionados usando tipos de enlaces semánticos:

```json
{
  "name": "create_link",
  "arguments": {
    "from": "01HX5V8F3K7QV9BZEC4N6P0M",
    "to": "01HX5V8F3K7QV9BZEC4N6P0A",
    "link_type": "extends"
  }
}
```

### Paso 4: Organizar (Operaciones de Estructura)

Usa herramientas de estructura para mantener la jerarquía:

```json
{
  "name": "reorder_block",
  "arguments": {
    "block_id": "01HX5V8F3K7QV9BZEC4N6P0M",
    "after_block_id": "01HX5V8F3K7QV9BZEC4N6P0B",
    "position": "after"
  }
}
```

### Paso 5: Revisar (Detectar Vacíos)

Identifica conexiones y secciones faltantes:

```json
{
  "name": "detect_gaps",
  "arguments": {
    "root_block_id": "01HX5V8F3K7QV9BZEC4N6P0Z"
  }
}
```

### Paso 6: Stagear (Working Set)

Añade bloques procesados al área de staging:

```json
{
  "name": "stage_block",
  "arguments": {
    "block_id": "01HX5V8F3K7QV9BZEC4N6P0M"
  }
}
```

### Paso 7: Confirmar (Snapshot Permanente)

Finaliza los cambios cuando esté verdaderamente listo:

```json
{
  "name": "commit_changes",
  "arguments": {
    "message": "Añadir notas de arquitectura transformer con enlaces a mecanismos de atención"
  }
}
```

---

## Referencia Completa de Herramientas MCP

### Operaciones de Conocimiento

#### 1. create_block

Crea un nuevo bloque de conocimiento con enriquecimiento IA opcional.

**Parámetros:**

| Parámetro | Tipo | Requerido | Descripción |
|-----------|------|-----------|-------------|
| block_type | string | Sí | Tipo: fleeting, literature, permanent, structure, hub, task, reference, outline, ghost |
| title | string | Sí | Título del bloque |
| content | string | No | Contenido del bloque |
| tags | string[] | No | Tags iniciales |
| enrich | boolean | No | Si es true, devuelve datos de enriquecimiento IA (default: false) |

**Respuesta (con enrich=true):**

```json
{
  "id": "01HX5V8F3K7QV9BZEC4N6P0M",
  "block_type": "fleeting",
  "title": "Idea sobre redes neuronales",
  "content": "Las arquitecturas Transformer son potentes...",
  "tags": [],
  "created_at": "2026-03-20T10:30:00Z",
  "updated_at": "2026-03-20T10:30:00Z",

  "// Datos de Enriquecimiento IA (CAMPOS DE NIVEL SUPERIOR, NO anidados bajo 'enrichment')",

  "link_suggestions": [
    {
      "target_id": "01HX5V8F3K7QV9BZEC4N6P0A",
      "link_type": "extends",
      "confidence": 0.85,
      "reason": "El contenido extiende la discusión sobre mecanismos de atención"
    }
  ],
  "tag_suggestions": ["machine-learning", "transformers", "deep-learning"],
  "gravity_info": {
    "gravity_score": 3.2,
    "outgoing_links": 5,
    "incoming_links": 2
  },
  "type_suggestion": {
    "suggested_type": "literature",
    "confidence": 0.72,
    "reason": "El contenido tiene estructura de tipo referencia"
  }
}
```

---

#### 2. get_block

Recupera un bloque por ID con contenido opcional.

**Parámetros:**

| Parámetro | Tipo | Requerido | Descripción |
|-----------|------|-----------|-------------|
| block_id | string | Sí | ULID del bloque a recuperar |
| include_content | boolean | No | Incluir contenido completo (default: false) |

**Ejemplo:**

```json
{
  "name": "get_block",
  "arguments": {
    "block_id": "01HX5V8F3K7QV9BZEC4N6P0M",
    "include_content": true
  }
}
```

---

#### 3. search_blocks

Búsqueda de texto completo en bloques por query, tipo o tags.

**Parámetros:**

| Parámetro | Tipo | Requerido | Descripción |
|-----------|------|-----------|-------------|
| query | string | No | Texto de búsqueda |
| block_type | string | No | Filtrar por tipo de bloque |
| tags | string[] | No | Filtrar por tags (lógica AND) |
| limit | number | No | Resultados máximos (default: 50) |
| offset | number | No | Offset de paginación (default: 0) |

**Ejemplo:**

```json
{
  "name": "search_blocks",
  "arguments": {
    "query": "arquitectura transformer",
    "block_type": "literature",
    "tags": ["machine-learning"],
    "limit": 10
  }
}
```

**Respuesta:**

```json
{
  "blocks": [
    {
      "id": "01HX5V8F3K7QV9BZEC4N6P0M",
      "block_type": "literature",
      "title": "Attention is All You Need",
      "tags": ["machine-learning", "transformers"],
      "created_at": "2026-03-20T10:30:00Z"
    }
  ],
  "total": 1,
  "limit": 10,
  "offset": 0
}
```

---

#### 4. update_block

Actualiza el contenido y propiedades de un bloque.

**Parámetros:**

| Parámetro | Tipo | Requerido | Descripción |
|-----------|------|-----------|-------------|
| block_id | string | Sí | ULID del bloque a actualizar |
| title | string | No | Nuevo título |
| content | string | No | Nuevo contenido |
| tags | string[] | No | Reemplazar todos los tags |
| block_type | string | No | Cambiar tipo de bloque |

**Ejemplo:**

```json
{
  "name": "update_block",
  "arguments": {
    "block_id": "01HX5V8F3K7QV9BZEC4N6P0M",
    "title": "Actualizado: Arquitectura Transformer",
    "tags": ["machine-learning", "transformers", "nlp"],
    "block_type": "literature"
  }
}
```

---

### Operaciones de Enlaces

#### 5. create_link

Crea un enlace dirigido entre dos bloques con un tipo de enlace semántico.

**Parámetros:**

| Parámetro | Tipo | Requerido | Descripción |
|-----------|------|-----------|-------------|
| from | string | Sí | ULID del bloque origen |
| to | string | Sí | ULID del bloque destino |
| link_type | string | Sí | Tipo: extends, refines, contradicts, questions, supports, references, related, similar_to, section_of, next |

**Ejemplo:**

```json
{
  "name": "create_link",
  "arguments": {
    "from": "01HX5V8F3K7QV9BZEC4N6P0M",
    "to": "01HX5V8F3K7QV9BZEC4N6P0A",
    "link_type": "extends"
  }
}
```

**Respuesta:**

```json
{
  "success": true,
  "link_id": "link_01HX5V8F3K7QV9BZEC4N6P0Z",
  "from": "01HX5V8F3K7QV9BZEC4N6P0M",
  "to": "01HX5V8F3K7QV9BZEC4N6P0A",
  "link_type": "extends"
}
```

---

#### 6. get_links

Consulta enlaces desde o hacia un bloque específico.

**Parámetros:**

| Parámetro | Tipo | Requerido | Descripción |
|-----------|------|-----------|-------------|
| block_id | string | Sí | ULID del bloque |
| direction | string | No | "outgoing", "incoming", o "both" (default: "both") |
| link_type | string | No | Filtrar por tipo de enlace |

**Ejemplo:**

```json
{
  "name": "get_links",
  "arguments": {
    "block_id": "01HX5V8F3K7QV9BZEC4N6P0M",
    "direction": "outgoing",
    "link_type": "extends"
  }
}
```

---

#### 7. suggest_links

Sugerencias de enlaces impulsadas por IA basadas en similitud de contenido y análisis semántico.

**Parámetros:**

| Parámetro | Tipo | Requerido | Descripción |
|-----------|------|-----------|-------------|
| block_id | string | Sí | ULID del bloque para obtener sugerencias |
| limit | number | No | Sugerencias máximas (default: 10) |

**Ejemplo:**

```json
{
  "name": "suggest_links",
  "arguments": {
    "block_id": "01HX5V8F3K7QV9BZEC4N6P0M",
    "limit": 5
  }
}
```

**Respuesta:**

```json
{
  "block_id": "01HX5V8F3K7QV9BZEC4N6P0M",
  "suggestions": [
    {
      "target_id": "01HX5V8F3K7QV9BZEC4N6P0A",
      "link_type": "extends",
      "confidence": 0.85,
      "reason": "Similitud semántica en la discusión de mecanismos de atención"
    }
  ]
}
```

---

### Operaciones de Estructura

#### 8. traverse_spine

Recorre la estructura jerárquica espinal de los bloques.

**Parámetros:**

| Parámetro | Tipo | Requerido | Descripción |
|-----------|------|-----------|-------------|
| root_block_id | string | Sí | ULID del bloque inicial |
| direction | string | No | "forward" o "backward" (default: "forward") |
| depth | number | No | Profundidad máxima a recorrer (default: ilimitado) |

**Ejemplo:**

```json
{
  "name": "traverse_spine",
  "arguments": {
    "root_block_id": "01HX5V8F3K7QV9BZEC4N6P0Z",
    "direction": "forward",
    "depth": 3
  }
}
```

---

#### 9. gravity_check

Verifica las métricas de conectividad de un bloque (detección de hubs).

**Parámetros:**

| Parámetro | Tipo | Requerido | Descripción |
|-----------|------|-----------|-------------|
| block_id | string | Sí | ULID del bloque a analizar |
| depth | number | No | Profundidad de análisis (default: 1) |

**Ejemplo:**

```json
{
  "name": "gravity_check",
  "arguments": {
    "block_id": "01HX5V8F3K7QV9BZEC4N6P0M"
  }
}
```

**Respuesta:**

```json
{
  "block_id": "01HX5V8F3K7QV9BZEC4N6P0M",
  "gravity_score": 3.2,
  "direct_links": {
    "outgoing": 5,
    "incoming": 2
  },
  "network_metrics": {
    "betweenness_centrality": 0.15,
    "page_rank": 0.08
  }
}
```

---

#### 10. reorder_block

Reordena bloques dentro de la jerarquía espinal.

**Parámetros:**

| Parámetro | Tipo | Requerido | Descripción |
|-----------|------|-----------|-------------|
| block_id | string | Sí | ULID del bloque a reordenar |
| after_block_id | string | No | ULID del bloque a colocar después |
| before_block_id | string | No | ULID del bloque a colocar antes |
| parent_id | string | No | Nuevo ID del bloque padre |

**Ejemplo:**

```json
{
  "name": "reorder_block",
  "arguments": {
    "block_id": "01HX5V8F3K7QV9BZEC4N6P0M",
    "after_block_id": "01HX5V8F3K7QV9BZEC4N6P0B",
    "position": "after"
  }
}
```

---

#### 11. get_section_map

Obtiene el árbol de sección jerárquica comenzando desde un bloque.

**Parámetros:**

| Parámetro | Tipo | Requerido | Descripción |
|-----------|------|-----------|-------------|
| root_block_id | string | No | Bloque inicial (default: raíz) |
| max_depth | number | No | Profundidad máxima del árbol (default: ilimitado) |

**Ejemplo:**

```json
{
  "name": "get_section_map",
  "arguments": {
    "root_block_id": "01HX5V8F3K7QV9BZEC4N6P0Z",
    "max_depth": 3
  }
}
```

**Respuesta:**

```json
{
  "root_id": "01HX5V8F3K7QV9BZEC4N6P0Z",
  "sections": [
    {
      "id": "01HX5V8F3K7QV9BZEC4N6P0M",
      "title": "Título de Sección",
      "children": []
    }
  ]
}
```

---

#### 12. detect_gaps

Identifica secciones faltantes en el grafo de conocimiento.

**Parámetros:**

| Parámetro | Tipo | Requerido | Descripción |
|-----------|------|-----------|-------------|
| root_block_id | string | No | Bloque raíz a analizar |
| expected_sections | string[] | No | Títulos de secciones esperados |

**Ejemplo:**

```json
{
  "name": "detect_gaps",
  "arguments": {
    "root_block_id": "01HX5V8F3K7QV9BZEC4N6P0Z",
    "expected_sections": ["Introducción", "Métodos", "Resultados", "Conclusión"]
  }
}
```

**Respuesta:**

```json
{
  "root_block_id": "01HX5V8F3K7QV9BZEC4N6P0Z",
  "gaps": [
    {
      "expected": "Métodos",
      "status": "missing",
      "suggestion": "Considera añadir una sección de Métodos"
    }
  ]
}
```

---

#### 13. list_ghosts

Lista nodos fantasma con filtrado opcional por estado.

**Parámetros:**

| Parámetro | Tipo | Requerido | Descripción |
|-----------|------|-----------|-------------|
| status | string | No | Filtrar: predicted, confirmed, resolved |
| block_type | string | No | Filtrar por tipo de bloque |

**Ejemplo:**

```json
{
  "name": "list_ghosts",
  "arguments": {
    "status": "predicted",
    "block_type": "outline"
  }
}
```

---

#### 14. synthesize

Genera un documento desde la estructura de bloques.

**Parámetros:**

| Parámetro | Tipo | Requerido | Descripción |
|-----------|------|-----------|-------------|
| root_block_id | string | Sí | Bloque raíz para síntesis |
| format | string | No | Formato de salida: markdown, html, json (default: markdown) |
| include_metadata | boolean | No | Incluir metadatos en salida (default: true) |

**Ejemplo:**

```json
{
  "name": "synthesize",
  "arguments": {
    "root_block_id": "01HX5V8F3K7QV9BZEC4N6P0Z",
    "format": "markdown",
    "include_metadata": true
  }
}
```

---

#### 15. get_toc

Obtiene la tabla de contenidos de una jerarquía de bloques.

**Parámetros:**

| Parámetro | Tipo | Requerido | Descripción |
|-----------|------|-----------|-------------|
| root_block_id | string | No | Bloque raíz (default: raíz del documento) |
| max_depth | number | No | Profundidad máxima de encabezados (default: 6) |

**Ejemplo:**

```json
{
  "name": "get_toc",
  "arguments": {
    "root_block_id": "01HX5V8F3K7QV9BZEC4N6P0Z",
    "max_depth": 3
  }
}
```

**Respuesta:**

```json
{
  "root_id": "01HX5V8F3K7QV9BZEC4N6P0Z",
  "toc": [
    {
      "id": "01HX5V8F3K7QV9BZEC4N6P0M",
      "title": "Introducción",
      "level": 1,
      "children": [
        {
          "id": "01HX5V8F3K7QV9BZEC4N6P0N",
          "title": "Contexto",
          "level": 2,
          "children": []
        }
      ]
    }
  ]
}
```

---

### Operaciones de Staging y Commit

#### 16. stage_block

Añade un bloque al área de staging del working set.

**Parámetros:**

| Parámetro | Tipo | Requerido | Descripción |
|-----------|------|-----------|-------------|
| block_id | string | Sí | ULID del bloque a stagear |

**Ejemplo:**

```json
{
  "name": "stage_block",
  "arguments": {
    "block_id": "01HX5V8F3K7QV9BZEC4N6P0M"
  }
}
```

**Respuesta:**

```json
{
  "success": true,
  "message": "Bloque 01HX5V8F3K7QV9BZEC4N6P0M staged para commit"
}
```

---

#### 17. commit_changes

Crea un commit desde todos los cambios staged.

**Parámetros:**

| Parámetro | Tipo | Requerido | Descripción |
|-----------|------|-----------|-------------|
| message | string | Sí | Mensaje de commit describiendo los cambios |
| author | string | No | Nombre del autor (default: "user") |

**Ejemplo:**

```json
{
  "name": "commit_changes",
  "arguments": {
    "message": "Añadir notas de arquitectura transformer con enlaces a mecanismos de atención",
    "author": "ai-agent-1"
  }
}
```

**Respuesta:**

```json
{
  "success": true,
  "commit_id": "01HX5V8F3K7QV9BZEC4N6P0Z",
  "message": "Añadir notas de arquitectura transformer con enlaces a mecanismos de atención",
  "blocks_staged": 1,
  "blocks_committed": 1
}
```

---

#### 18. get_working_set_status

Devuelve el estado actual del área de staging.

**Parámetros:**

Ninguno.

**Ejemplo:**

```json
{
  "name": "get_working_set_status",
  "arguments": {}
}
```

**Respuesta:**

```json
{
  "staged_blocks": ["01HX5V8F3K7QV9BZEC4N6P0M"],
  "staged_edges": [],
  "removed_blocks": [],
  "removed_edges": [],
  "is_empty": false
}
```

---

## Tipos de Bloques

| Tipo | Propósito | Auto-stage? |
|------|-----------|------------|
| fleeting | Capturas rápidas, notas temporales | No |
| literature | Notas procesadas de fuentes | No |
| permanent | Conocimiento final, pulido | No |
| structure | Bloques de organización (TOC, índice) | No |
| hub | Conectores de tema central | No |
| task | Elementos de acción, TODOs | No |
| reference | Referencias externas, citas | No |
| outline | Esquemas jerárquicos | No |
| ghost | Predicciones placeholder | No |

---

## Datos de Enriquecimiento

Cuando `enrich=true` se pasa a `create_block`, los siguientes campos de nivel superior son devueltos (NO anidados bajo "enrichment"):

### link_suggestions

```json
{
  "target_id": "01HX...",
  "link_type": "extends|refines|contradicts|questions|supports|references|related|similar_to|section_of|next",
  "confidence": 0.0-1.0,
  "reason": "Explicación legible por humanos"
}
```

### tag_suggestions

Array de tags sugeridos ordenados por relevancia.

### gravity_info

```json
{
  "gravity_score": 0.0-10.0,
  "outgoing_links": número,
  "incoming_links": número
}
```

Mayor gravedad = más conectado (tipo hub).

### type_suggestion

```json
{
  "suggested_type": "literature|permanent|...",
  "confidence": 0.0-1.0,
  "reason": "Por qué se sugiere este tipo"
}
```

---

## Principios de Diseño

1. **Explícito sobre Implícito**: Nada sucede automáticamente
2. **Enriquecimiento bajo demanda**: La IA elige cuándo obtener contexto
3. **Commits Atómicos**: Los cambios se stagean hasta estar verdaderamente listos
4. **Contexto Completo**: La IA obtiene sugerencias de enlaces, gravedad, hints de tipo antes de confirmar
5. **Reversible**: El working set permite descartar antes del commit
6. **Método Zettelkasten**: Notas atómicas, enlaces semánticos, crecimiento orgánico
7. **Nodos Fantasma**: Placeholders predictivos para conexiones futuras

---

## Ejemplo: Construcción Completa de Conocimiento por IA

```python
# 1. Crear una nota fleeting con enriquecimiento
result = mcp.call_tool("create_block", {
    "block_type": "fleeting",
    "title": "Attention is all you need",
    "content": "La arquitectura Transformer...",
    "enrich": True
})

# 2. La IA recibe sugerencias en NIVEL SUPERIOR (no bajo "enrichment"):
# - result.link_suggestions
# - result.tag_suggestions
# - result.gravity_info
# - result.type_suggestion

# 3. La IA crea enlaces semánticos
mcp.call_tool("create_link", {
    "from": result.id,
    "to": "bloque_nn_existente_id",
    "link_type": "extends"
})

# 4. Verificar estructura y detectar vacíos
gaps = mcp.call_tool("detect_gaps", {
    "root_block_id": result.id,
    "expected_sections": ["Resumen", "Introducción", "Métodos"]
})

# 5. Stagear el bloque
mcp.call_tool("stage_block", {"block_id": result.id})

# 6. Verificar estado de staging
status = mcp.call_tool("get_working_set_status", {})

# 7. Hacer commit cuando esté verdaderamente lista
mcp.call_tool("commit_changes", {
    "message": "Añadir notas del paper Transformer con enlaces a redes neuronales"
})
```
