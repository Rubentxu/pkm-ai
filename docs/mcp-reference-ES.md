# Referencia MCP para PKM

**Estado:** Implementado
**Version:** 1.0
**Fecha:** 2026-03-20

---

## Descripcion General

El servidor MCP de PKM proporciona a los agentes de IA acceso a un grafo de conocimiento estilo Zettelkasten a traves del [Model Context Protocol](https://modelcontextprotocol.io/). Expone **15 herramientas** en 5 categorias para gestionar notas atomicas, enlaces semanticos y documentos estructurados.

**Transporte:** stdio (JSON-RPC)
**Implementacion:** `src/ai/mcp.rs` usando el crate `rmcp`

---

## Inicio Rapido

### Ejecutar el Servidor

```bash
# Via cargo
cargo run --release --bin pkm-ai -- mcp

# O via alias de CLI
pkm-ai mcp

---

## Resumen de Herramientas

| Categoria | Herramientas | Cantidad |
|-----------|--------------|----------|
| Bloque | `create_block`, `get_block`, `search_blocks`, `update_block` | 4 |
| Enlace | `create_link`, `get_links`, `suggest_links` | 3 |
| Espina | `traverse_spine`, `gravity_check`, `reorder_block` | 3 |
| Estructura | `get_section_map`, `detect_gaps`, `list_ghosts` | 3 |
| Sintesis | `synthesize`, `get_toc` | 2 |

---

## Herramientas de Bloque

### create_block

Crea un nuevo bloque en el grafo de conocimiento.

**Parametros:**

| Parametro | Tipo | Obligatorio | Descripcion |
|-----------|------|-------------|-------------|
| `block_type` | string | Si | Tipo: `fleeting`, `literature`, `permanent`, `structure`, `hub`, `task`, `reference`, `outline`, `ghost` |
| `title` | string | No | Titulo del bloque (predeterminado: "Sin titulo") |
| `content` | string | No | Contenido en formato Markdown |
| `tags` | string[] | No | Etiquetas para clasificacion |

**Respuesta:**
```json
{
  "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "block_type": "permanent",
  "title": "Mi Zettel",
  "created_at": "2026-03-20T10:00:00Z"
}
```

---

### get_block

Recupera un bloque por su ULID.

**Parametros:**

| Parametro | Tipo | Obligatorio | Predeterminado | Descripcion |
|-----------|------|-------------|----------------|-------------|
| `block_id` | string | Si | - | ULID del bloque |
| `include_content` | boolean | No | `true` | Incluir contenido completo |

**Respuesta:**
```json
{
  "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "block_type": "permanent",
  "title": "Mi Zettel",
  "content": "# Nota Atomica\n\nEsta es mi idea atomica.",
  "tags": ["idea"],
  "metadata": {},
  "created_at": "2026-03-20T10:00:00Z",
  "updated_at": "2026-03-20T10:00:00Z"
}
```

---

### search_blocks

Busca bloques por consulta, tipo o devuelve todos los bloques.

**Parametros:**

| Parametro | Tipo | Obligatorio | Predeterminado | Descripcion |
|-----------|------|-------------|----------------|-------------|
| `query` | string | No | - | Consulta de busqueda de texto completo |
| `block_type` | string | No | - | Filtrar por tipo de bloque |
| `limit` | integer | No | `20` | Maximo de resultados (max 100) |

**Respuesta:**
```json
{
  "blocks": [
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "block_type": "permanent",
      "title": "Mi Zettel",
      "content": "Primeros 200 caracteres...",
      "tags": ["idea"],
      "created_at": "2026-03-20T10:00:00Z"
    }
  ],
  "count": 1
}
```

---

### update_block

Actualiza el contenido o las propiedades de un bloque.

**Parametros:**

| Parametro | Tipo | Obligatorio | Descripcion |
|-----------|------|-------------|-------------|
| `block_id` | string | Si | ULID del bloque |
| `content` | string | No | Nuevo contenido |
| `properties` | object | No | Pares clave-valor para metadatos |

**Respuesta:** Confirmacion en texto plano

```
Bloque 01ARZ3NDEKTSV4RRFFQ69G5FAV actualizado correctamente
```

---

## Herramientas de Enlace

### create_link

Crea un enlace dirigido entre dos bloques.

**Parametros:**

| Parametro | Tipo | Obligatorio | Descripcion |
|-----------|------|-------------|-------------|
| `source_id` | string | Si | ULID del bloque origen |
| `target_id` | string | Si | ULID del bloque destino |
| `link_type` | string | Si | Tipo de enlace |

**Tipos de Enlaces:**
- Estructurales: `section_of`, `subsection_of`, `ordered_child`, `next`, `next_sibling`, `first_child`, `contains`, `parent`
- Semanticos: `extends`, `refines`, `contradicts`, `questions`, `supports`, `references`
- De Similitud: `related`, `similar_to`, `ai_suggested`

**Respuesta:**
```json
{
  "id": "01ARZ3NDEKTSV4RRFFQ69G5FAW",
  "source_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "target_id": "01ARZ3NDEKTSV4RRFFQ69G5FAZ",
  "link_type": "supports"
}
```

---

### get_links

Obtiene todos los enlaces desde o hacia un bloque.

**Parametros:**

| Parametro | Tipo | Obligatorio | Predeterminado | Descripcion |
|-----------|------|-------------|----------------|-------------|
| `block_id` | string | Si | - | ULID del bloque |
| `link_types` | string[] | No | - | Filtrar por tipos de enlace |
| `direction` | string | No | `both` | Direccion: `outgoing`, `incoming`, `both` |

**Respuesta:**
```json
{
  "edges": [
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAW",
      "from": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "to": "01ARZ3NDEKTSV4RRFFQ69G5FAZ",
      "link_type": "supports",
      "sequence_weight": 1.0,
      "created_at": "2026-03-20T10:00:00Z"
    }
  ],
  "count": 1
}
```

---

### suggest_links

Sugiere enlaces para un bloque usando analisis impulsado por IA.

**Parametros:**

| Parametro | Tipo | Obligatorio | Predeterminado | Descripcion |
|-----------|------|-------------|----------------|-------------|
| `block_id` | string | Si | - | ULID del bloque |
| `confidence_threshold` | number | No | `0.5` | Umbral de confianza minimo (0.0-1.0) |
| `limit` | integer | No | `10` | Maximo de sugerencias |

**Respuesta:**
```json
{
  "suggestions": [
    {
      "target_id": "01ARZ3NDEKTSV4RRFFQ69G5FAZ",
      "link_type": "related",
      "confidence": 0.85,
      "reason": "Ambas notas discuten arquitecturas de redes neuronales"
    }
  ],
  "count": 1
}
```

---

## Herramientas de Espina

### traverse_spine

Recorre la espina estructural desde un bloque raiz.

**Parametros:**

| Parametro | Tipo | Obligatorio | Predeterminado | Descripcion |
|-----------|------|-------------|----------------|-------------|
| `root_id` | string | No | - | ULID del bloque estructura raiz (null para espina completa) |
| `max_depth` | integer | No | `0` | Profundidad maxima de recorrido (0 = ilimitado) |

**Respuesta:**
```json
{
  "root_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "blocks": [
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "block_type": "structure",
      "title": "Mi Documento"
    }
  ],
  "total_count": 2,
  "depth": 1
}
```

---

### gravity_check

Verifica la conectividad (gravedad) de un bloque en el grafo de conocimiento.

**Parametros:**

| Parametro | Tipo | Obligatorio | Descripcion |
|-----------|------|-------------|-------------|
| `block_id` | string | Si | ULID del bloque |

**Respuesta:**
```json
{
  "block_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "gravity_score": 15.0,
  "outgoing_links": 8,
  "incoming_links": 7,
  "total_connections": 15
}
```

---

### reorder_block

Reordena un bloque en la espina estructural.

**Parametros:**

| Parametro | Tipo | Obligatorio | Descripcion |
|-----------|------|-------------|-------------|
| `block_id` | string | Si | ULID del bloque a reordenar |
| `after_id` | string | No* | ULID del bloque a colocar despues |
| `before_id` | string | No* | ULID del bloque a colocar antes |

*Se debe proporcionar al menos `after_id` o `before_id`.

**Respuesta:**
```json
{
  "block_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "after_id": "01ARZ3NDEKTSV4RRFFQ69G5FAY",
  "before_id": "01ARZ3NDEKTSV4RRFFQ69G5FAZ",
  "sequence_weight": 1.5,
  "message": "Bloque reordenado correctamente"
}
```

---

## Herramientas de Estructura

### get_section_map

Obtiene la jerarquia de secciones desde un bloque estructura raiz.

**Parametros:**

| Parametro | Tipo | Obligatorio | Descripcion |
|-----------|------|-------------|-------------|
| `root_id` | string | Si | ULID del bloque estructura raiz |

**Respuesta:**
```json
{
  "root_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "sections": [
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAZ",
      "title": "Capitulo 1",
      "block_type": "permanent",
      "sequence_weight": 1.0
    }
  ],
  "count": 1
}
```

---

### detect_gaps

Detecta huecos (nodos fantasma) en una seccion usando analisis de IA.

**Parametros:**

| Parametro | Tipo | Obligatorio | Descripcion |
|-----------|------|-------------|-------------|
| `section_id` | string | Si | ULID del bloque seccion |

**Respuesta:**
```json
{
  "section_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "detected_gaps": [
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FB0",
      "description": "Seccion faltante sobre optimizacion de rendimiento",
      "confidence": 0.85,
      "status": "detected",
      "ai_rationale": "Tema mencionado en la introduccion pero nunca expandido",
      "position_hint": {
        "after": "01ARZ3NDEKTSV4RRFFQ69G5FAZ",
        "before": null,
        "parent_section": "01ARZ3NDEKTSV4RRFFQ69G5FAV"
      }
    }
  ],
  "count": 1
}
```

---

### list_ghosts

Lista los nodos fantasma (contenedores de contenido) en el grafo de conocimiento.

**Parametros:**

| Parametro | Tipo | Obligatorio | Descripcion |
|-----------|------|-------------|-------------|
| `status` | string | No | Filtrar: `detected`, `acknowledged`, `in_progress`, `filled`, `dismissed` |
| `confidence_below` | number | No | Filtrar por umbral de confianza |

**Respuesta:**
```json
{
  "ghosts": [
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FB0",
      "title": "Optimizacion de Rendimiento",
      "content": "",
      "ai_confidence": 0.85,
      "status": "detected",
      "created_at": "2026-03-20T10:00:00Z"
    }
  ],
  "count": 1
}
```

---

## Herramientas de Sintesis

### synthesize

Sintetiza un documento desde una estructura.

**Parametros:**

| Parametro | Tipo | Obligatorio | Descripcion |
|-----------|------|-------------|-------------|
| `structure_id` | string | Si | ULID del bloque estructura |
| `template` | string | No | Nombre de plantilla (predeterminado: "default") |
| `output_path` | string | No | Ruta del archivo de salida |

**Respuesta:**
```json
{
  "structure_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "title": "Mi Documento",
  "format": "markdown",
  "blocks_used": 5,
  "blocks_total": 7,
  "content": "# Mi Documento\n\nContenido sintetizado...",
  "message": "Sintesis completada correctamente"
}
```

---

### get_toc

Obtiene la tabla de contenidos de una estructura.

**Parametros:**

| Parametro | Tipo | Obligatorio | Descripcion |
|-----------|------|-------------|-------------|
| `structure_id` | string | Si | ULID del bloque estructura |

**Respuesta:**
```json
{
  "structure_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "toc": [
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAZ",
      "title": "Capitulo 1",
      "level": 1
    },
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FB0",
      "title": "Seccion 1.1",
      "level": 2
    }
  ],
  "count": 2
}
```

---

## Referencia de Tipos de Bloque

| Tipo | Descripcion | Alias |
|------|-------------|-------|
| `fleeting` | Notas temporales (capturadas rapidamente) | `f` |
| `literature` | Material de referencia de fuentes externas | `l` |
| `permanent` | Notas atomicas de Zettelkasten | `p` |
| `structure` | Contenedores estructurales (documentos, libros) | `s`, `index`, `moc` |
| `hub` | Nodos centrales de temas | `h` |
| `task` | Elementos de accion y tareas | `t` |
| `reference` | Referencias externas | `r` |
| `outline` | Esquemas jerarquicos | `o` |
| `ghost` | Marcador de posicion para contenido faltante | `g` |

---

## Codigos de Error

| Codigo | Nombre | Descripcion |
|--------|--------|-------------|
| `-32600` | Error de Analisis | JSON invalido |
| `-32601` | Metodo No Encontrado | Nombre de herramienta desconocido |
| `-32602` | Parametros Invalidos | Parametros invalidos |
| `-32603` | Error Interno | Error del lado del servidor |

---

## Ejemplos de Configuracion para Agentes

### Claude Desktop (claude_desktop_config.json)

```json
{
  "mcpServers": {
    "pkm": {
      "command": "cargo",
      "args": ["run", "--release", "--bin", "pkm-ai", "--", "mcp"],
      "env": {},
      "workingDirectory": "/path/to/hodei-pkm"
    }
  }
}
```

### Claude Code (settings.json)

```json
{
  "mcpServers": {
    "pkm": {
      "command": "cargo",
      "args": ["run", "--release", "--bin", "pkm-ai", "--", "mcp"],
      "workingDirectory": "/path/to/hodei-pkm"
    }
  }
}
```

### Configuracion de Engram Memory

```json
{
  "memory": {
    "pkm": {
      "type": "mcp",
      "command": "cargo",
      "args": ["run", "--release", "--bin", "pkm-ai", "--", "mcp"],
      "workingDirectory": "/path/to/hodei-pkm"
    }
  }
}
```

---

## Estado de Implementacion

| Herramienta | Estado | Notas |
|-------------|--------|-------|
| `create_block` | Implementado | Totalmente funcional |
| `get_block` | Implementado | Totalmente funcional |
| `search_blocks` | Implementado | Totalmente funcional |
| `update_block` | Implementado | Totalmente funcional |
| `create_link` | Implementado | Totalmente funcional |
| `get_links` | Implementado | Totalmente funcional |
| `suggest_links` | Implementado | Usa LinkSuggester |
| `traverse_spine` | Implementado | Totalmente funcional |
| `gravity_check` | Implementado | Totalmente funcional |
| `reorder_block` | Implementado | Totalmente funcional |
| `get_section_map` | Implementado | Totalmente funcional |
| `detect_gaps` | Implementado | Usa GhostDetector |
| `list_ghosts` | Implementado | Totalmente funcional |
| `synthesize` | Implementado | Totalmente funcional |
| `get_toc` | Implementado | Totalmente funcional |

---

## Documentacion Relacionada

- [MCP README](docs/mcp/README.md) - Documentacion completa del servidor
- [MCP API](docs/mcp/API.md) - Referencia detallada de la API
- [MCP SKILL](docs/mcp/SKILL.md) - Documentacion de habilidades para agentes
- [MCP Use Cases](docs/mcp/USE_CASES.md) - Ejemplos de uso practico

---

**Ultima actualizacion:** 2026-03-20
