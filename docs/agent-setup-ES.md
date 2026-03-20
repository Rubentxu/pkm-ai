# Configuracion de Agentes — Protocolo de Memoria PKM-AI

> Configurar agentes de IA para persistir conocimiento entre sesiones usando PKM-AI.

**Version:** 1.0
**Fecha:** 2026-03-20

---

## Referencia Rapida

| Agente | Complejidad de Configuracion | Soporte MCP | Protocolo de Memoria | Auto-Guardado |
|--------|------------------------------|-------------|---------------------|---------------|
| **Claude Code** | Facil | Nativo | Completo | Hooks de sesion |
| **OpenCode** | Facil | Nativo | Completo | Hooks de sesion |
| **Gemini CLI** | Facil | Nativo | Completo | Hooks de sesion |
| **Codex** | Medio | Solo MCP | Parcial | Manual |
| **VS Code (Continue)** | Medio | Solo MCP | Parcial | Manual |
| **Cursor** | Medio | Solo MCP | Parcial | Manual |
| **Windsurf** | Medio | Solo MCP | Parcial | Manual |
| **Otros Agentes MCP** | Medio | Nativo | Completo | Depende |

---

## 1. Requisitos Previos

### 1.1 Requisitos del Sistema

- **PKM-AI instalado** (binario `pkmai`)
- **Servidor MCP en ejecucion** (`pkmai mcp` o `pkm-ai mcp`)
- **Contexto del proyecto**: Base de conocimiento inicializada en tu proyecto

### 1.2 Verificar Instalacion

```bash
# Verificar que PKM-AI esta instalado
pkmai --version

# Verificar que el servidor MCP inicia
pkmai mcp --help

# Inicializar base de conocimiento del proyecto (si no existe)
pkmai init
```

---

## 2. Configuracion de Claude Code

### Requisitos Previos

- Claude Code instalado (`npm install -g @anthropic/claude-code`)
- Servidor MCP de PKM-AI accesible

### Comando de Configuracion

```bash
# Agregar PKM-AI como caracteristica de Claude Code
claude code --add-feature pkm-memory

# O via variable de entorno
export PKM_MCP_SERVER="pkmai mcp"
```

### Configuracion Manual

1. Crear archivo de configuracion: `~/.claude/settings.json`

```json
{
  "mcpServers": {
    "pkm": {
      "command": "pkmai",
      "args": ["mcp"],
      "env": {
        "PKM_PROJECT": "/ruta/a/tu/proyecto"
      }
    }
  },
  "memory": {
    "provider": "pkm",
    "autoSave": true,
    "sessionHooks": true
  }
}
```

### Integracion del Protocolo de Memoria

```bash
# Iniciar sesion con seguimiento automatico
pkmai session start --agent claude-code --project myproject

# Claude Code ahora:
# - Guardara decisiones arquitectonicas automaticamente
# - Buscara conocimiento antes de hacer cambios importantes
# - Vinculard Nuevos hallazgos a conceptos existentes
```

### Ganchos del Ciclo de Vida de Sesion

```rust
// .claude/hooks/session_start.sh
#!/bin/bash
pkmai session start \
  --agent "claude-code" \
  --project "$(basename $(pwd))" \
  --session-id "${CLAUDE_SESSION_ID}"

// .claude/hooks/session_end.sh
#!/bin/bash
pkmai session end \
  --session-id "${CLAUDE_SESSION_ID}" \
  --summary "$(cat /dev/stdin)"
```

---

## 3. Configuracion de OpenCode

### Requisitos Previos

- OpenCode instalado
- Servidor MCP de PKM-AI en ejecucion

### Comando de Configuracion

```bash
# Inicializar OpenCode con memoria PKM
opencode --setup-memory pkm

# O configurar manualmente
opencode config set memory.provider pkm
opencode config set mcp.enabled true
```

### Configuracion Manual

1. Crear `~/.opencode/config.yaml`:

```yaml
memory:
  provider: pkm
  autoSave: true
  searchOnStart: true

mcp:
  servers:
    - name: pkm
      command: pkmai
      args: [mcp]
      autoConnect: true

agent:
  session:
    trackDecisions: true
    linkToContext: true
```

### Integracion del Protocolo de Memoria

```bash
# OpenCode automaticamente:
# - Guardara patrones de codigo descubiertos durante la sesion
# - Vinculard errores corregidos a la base de conocimiento
# - Buscara soluciones pasadas similares antes de intentar correcciones
```

---

## 4. Configuracion de Gemini CLI

### Requisitos Previos

- Google Gemini CLI instalado
- Servidor MCP de PKM-AI accesible

### Comando de Configuracion

```bash
# Agregar memoria PKM a Gemini CLI
gemini --setup pkm-memory

# Configurar API key
export GEMINI_API_KEY="tu-api-key"
```

### Configuracion Manual

1. Crear `~/.geminirc`:

```bash
# Integracion PKM
GEMINI_MEMORY_PROVIDER=pkm
GEMINI_MCP_SERVER=pkmai:mcp
GEMINI_AUTO_MEMORY=true
```

2. O via `gemini config`:

```bash
gemini config set memory.provider pkm
gemini config set memory.autoSave true
gemini config set mcp.servers.pkm.command pkmai
gemini config set mcp.servers.pkm.args mcp
```

### Integracion del Protocolo de Memoria

```bash
# Gemini CLI ahora:
# - Persistira hallazgos de investigacion en PKM
# - Vinculard revisiones de codigo a decisiones arquitectonicas
# - Buscara sesiones pasadas para problemas similares
```

---

## 5. Configuracion de Codex

### Requisitos Previos

- Codex CLI instalado (`pip install openai-codex` o binario)
- Version de Codex compatible con MCP

### Comando de Configuracion

```bash
# Configurar Codex para usar el servidor MCP de PKM
codex config set mcp.servers.pkm.command "pkmai"
codex config set mcp.servers.pkm.args "mcp"
```

### Configuracion Manual

1. Crear `~/.codex/config.json`:

```json
{
  "mcpServers": {
    "pkm": {
      "command": "pkmai",
      "args": ["mcp"]
    }
  },
  "memory": {
    "enabled": true,
    "provider": "pkm"
  }
}
```

### Limitaciones

- Codex tiene soporte limitado de herramientas MCP
- Se requiere guardado manual de memoria: `pkmai save --message "..."`
- Compactacion de sesion no automatica

### Integracion del Protocolo de Memoria

```bash
# Guardados de memoria manuales
pkmai create --type permanent --title "Bug Fix: Expiracion de Token de Auth" --content "
**Problema**: Usuarios desconectados despues de 30 minutos
**Causa Raiz**: Expiracion de token no actualizada al refrescar
**Solucion**: Implementada expiracion con ventana deslizante
**Archivos**: src/auth/token.rs, src/middleware/auth.rs
"

# Buscar antes de comenzar
pkmai search "auth token refresh"
```

---

## 6. Configuracion de VS Code (Extension Continue)

### Requisitos Previos

- VS Code instalado
- Extension Continue instalada (u otra extension compatible con MCP)

### Comando de Configuracion

1. Instalar extension Continue desde el marketplace
2. Agregar servidor MCP de PKM en configuracion de VS Code:

```json
{
  "continue.serverConfigs": {
    "pkm": {
      "command": "pkmai",
      "args": ["mcp"],
      "env": {
        "PKM_PROJECT": "${workspaceFolder}"
      }
    }
  }
}
```

### Configuracion Manual

1. Crear `~/.continue/config.py`:

```python
from continuedev.src.continuedev.core.config import ContinueConfig

def modify_config(config: ContinueConfig):
    config.mcp_servers = [
        {
            "name": "pkm",
            "command": "pkmai",
            "args": ["mcp"],
            "env": {"PKM_PROJECT": "${workspace_folder}"}
        }
    ]
    return config
```

### Integracion del Protocolo de Memoria

- Click derecho en codigo seleccionado → "Guardar en PKM"
- Usar paleta de comandos: "PKM: Buscar Base de Conocimiento"
- Captura automatica del historial de chat (configurable)

---

## 7. Configuracion de Cursor

### Requisitos Previos

- Cursor IDE instalado
- Soporte MCP habilitado

### Comando de Configuracion

1. Abrir Configuracion de Cursor → MCP Servers
2. Agregar nuevo servidor MCP:

```
Nombre: PKM-AI
Comando: pkmai
Args: mcp
```

### Configuracion Manual

1. Editar `~/.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "pkm": {
      "command": "pkmai",
      "args": ["mcp"],
      "env": {
        "PKM_PROJECT": "${workspaceFolder}"
      }
    }
  }
}
```

### Integracion del Protocolo de Memoria

```bash
# Cursor usara PKM para:
# - Almacenar fragmentos de codigo con contexto
# - Vincular conceptos relacionados entre sesiones
# - Buscar soluciones pasadas
```

---

## 8. Configuracion de Windsurf

### Requisitos Previos

- IDE Windsurf instalado
- Version de Windsurf con soporte MCP

### Comando de Configuracion

1. Abrir Configuracion de Windsurf → External Tools
2. Agregar servidor MCP:

```
Nombre: PKM-AI Memory
Comando: pkmai
Args: mcp
```

### Configuracion Manual

1. Crear `~/.windsurf/mcp.json`:

```json
{
  "servers": {
    "pkm": {
      "command": "pkmai",
      "args": ["mcp"]
    }
  }
}
```

### Integracion del Protocolo de Memoria

- Windsurf usara PKM para:
  - Sugerencias de codigo conscientes del contexto
  - Seguimiento de decisiones arquitectonicas
  - Historial de correcciones de bugs con rationale

---

## 9. Otros Agentes MCP

### Configuracion Generica de MCP

Para cualquier agente compatible con MCP, agregar lo siguiente a su configuracion MCP:

```json
{
  "mcpServers": {
    "pkm": {
      "command": "pkmai",
      "args": ["mcp"],
      "env": {
        "PKM_PROJECT": "${WORKSPACE}",
        "PKM_AUTO_SAVE": "true"
      }
    }
  }
}
```

### Herramientas Disponibles para Todos los Agentes MCP

| Herramienta | Proposito | Parametros Clave |
|-------------|-----------|------------------|
| `create_block` | Crear bloque de conocimiento | `block_type`, `title`, `content`, `tags` |
| `search_blocks` | Buscar conocimiento | `query`, `block_type`, `limit` |
| `get_block` | Recuperar bloque | `block_id` |
| `update_block` | Actualizar bloque | `block_id`, `title`, `content`, `tags` |
| `create_link` | Vincular bloques | `from_id`, `to_id`, `link_type` |
| `get_links` | Obtener conexiones | `block_id`, `direction` |
| `suggest_links` | Sugerencias de vinculos IA | `block_id`, `limit` |
| `traverse_spine` | Navegar estructura | `root_id`, `depth` |
| `gravity_check` | Verificar conectividad | `block_id` |
| `synthesize` | Generar documento | `structure_id`, `format` |

---

## 10. Sobreviviendo a la Compactacion

Cuando los agentes compactan su ventana de contexto, los recuerdos almacenados en PKM sobreviven.

### Que Se Compacta

| Tipo de Contexto | Sobrevive? | Almacenamiento |
|------------------|------------|----------------|
| Historial de chat | No | PKM (si se guardo) |
| Memoria de trabajo | No | PKM (si se guardo) |
| Patrones aprendidos | **Si** | Bloques permanentes de PKM |
| Decisiones arquitectonicas | **Si** | Bloques permanentes de PKM |
| Correcciones de bugs y rationale | **Si** | Bloques permanentes de PKM |
| Patrones de codigo | **Si** | Bloques permanentes de PKM |
| Convenciones del proyecto | **Si** | Bloques de estructura de PKM |

### Protocolo de Recuperacion por Compactacion

```bash
# 1. Antes de la compactacion, el agente debe:
pkmai session checkpoint --session-id "${AGENT_SESSION_ID}"

# 2. Despues de la compactacion, restaurar contexto:
pkmai session restore --session-id "${AGENT_SESSION_ID}"

# 3. Buscar conocimiento pasado relevante:
pkmai search "problema similar" --limit 10
```

### Tipos de Bloques para Supervivencia a la Compactacion

| Tipo | Caso de Uso | TTL |
|------|-------------|-----|
| `permanent` | Conocimiento perenne | Infinito |
| `structure` | Arquitectura del proyecto | Infinito |
| `literature` | Referencias externas | Infinito |
| `fleeting` | Notas temporales | 7 dias |
| `task` | Elementos de accion | Hasta completar |

---

## 11. Protocolo de Memoria PKM

### Cuando Guardar Sesiones

Guardar en PKM cuando:

- [ ] **Decisiones arquitectonicas**: Elecciones de diseno, compromisos, rationale
- [ ] **Correcciones de bugs**: Causa raiz, solucion, archivos afectados
- [ ] **Patrones descubiertos**: Patrones de codigo, convenciones, idioms
- [ ] **Cambios de configuracion**: Entorno, construccion, despliegue
- [ ] **Nuevas dependencias**: Por que se agregaron, alternativas consideradas
- [ ] **Estrategias de prueba**: Que se probo, que paso, casos limite

### Cuando Buscar Conocimiento

Buscar en PKM antes de:

- [ ] **Iniciar nueva caracteristica**: "Como implementamos X similar?"
- [ ] **Depurar**: "Hemos visto este error antes?"
- [ ] **Refactorizar**: "Cual es el contexto alrededor de este codigo?"
- [ ] **Agregar dependencias**: "Por que usamos X en lugar de Y?"
- [ ] **Cambios mayores**: "Que decisiones se tomaron sobre esto?"

### Ganchos del Ciclo de Vida de Sesion

#### Ganchos Automaticos (Recomendado)

```bash
# .claude/hooks/session_start.sh
#!/bin/bash
export PKM_SESSION_ID="$(uuidgen)"
pkmai session start \
  --agent "claude-code" \
  --project "$(basename $(pwd))" \
  --session-id "$PKM_SESSION_ID" \
  --cwd "$(pwd)"

# .claude/hooks/session_end.sh
#!/bin/bash
pkmai session end \
  --session-id "$PKM_SESSION_ID" \
  --auto-summary true
```

#### Ganchos Manuales (Respaldo)

```bash
# Iniciar sesion manualmente
pkmai session start --agent opencode --project myapp

# Trabajar normalmente...

# Terminar sesion con resumen
pkmai session end --summary "Corregido bug de auth, refactorizado servicio de usuario"
```

### Tipos de Bloques Recomendados por Contenido

| Tipo de Contenido | Tipo de Bloque | Titulo de Ejemplo |
|-------------------|----------------|-------------------|
| Decision | `permanent` | `DECISION: Usar async/await para E/S` |
| Correccion de Bug | `permanent` | `FIX: Condicion de carrera en cache` |
| Patron | `permanent` | `PATRON: Repositorio con Unidad de Trabajo` |
| Convencion | `structure` | `CODE_STYLE: Manejo de errores` |
| Concepto | `permanent` | `COMPRENSION: Event Sourcing vs CRUD` |
| Referencia | `literature` | `REF: Guia de manejo de errores en Rust` |
| Tarea | `task` | `TODO: Migrar a nuevo proveedor de auth` |

### Ejemplos de Busqueda de Memoria

```bash
# Encontrar todas las correcciones de bugs
pkmai search --type permanent --query "FIX:" | head -20

# Encontrar decisiones arquitectonicas
pkmai search --type permanent --query "DECISION:" | head -20

# Encontrar patrones por lenguaje
pkmai search --type permanent --query "PATTERN: Rust" | head -20

# Encontrar todo del proyecto actual
pkmai search --project "$(basename $(pwd))" --limit 50
```

---

## 12. Solucion de Problemas

### Servidor MCP No Inicia

```bash
# Verificar que pkmai esta en PATH
which pkmai

# Iniciar servidor MCP manualmente
pkmai mcp

# Verificar errores
pkmai doctor
```

### Agente No Conecta a MCP

```bash
# Verificar que el servidor MCP responde
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | pkmai mcp

# Verificar configuracion MCP del agente
# Asegurar ruta correcta al binario pkmai
```

### Memoria No Persiste

```bash
# Verificar ubicacion de la base de datos PKM
pkmai config get database.path

# Verificar permisos de escritura
touch "$(pkmai config get database.path)/test"

# Verificar estado de sesion
pkmai session list
```

---

## 13. Referencia de Configuracion

### Variables de Entorno

| Variable | Valor Por Defecto | Descripcion |
|----------|-------------------|-------------|
| `PKM_PROJECT` | Directorio actual | Contexto del proyecto |
| `PKM_DB_PATH` | `~/.pkm/data.db` | Ruta de la base de datos |
| `PKM_AUTO_SAVE` | `false` | Auto-guardar sesion |
| `PKM_MCP_TIMEOUT` | `30000` | Timeout de MCP (ms) |

### Comandos CLI

| Comando | Descripcion |
|---------|-------------|
| `pkmai session start` | Iniciar sesion de conocimiento |
| `pkmai session end` | Terminar y resumir sesion |
| `pkmai session list` | Listar sesiones recientes |
| `pkmai session restore` | Restaurar contexto de sesion |
| `pkmai quick "mensaje"` | Captura rapida |
| `pkmai search "consulta"` | Buscar en base de conocimiento |

---

## 14. Ver Tambien

- [README de MCP](./mcp/README.md) — Documentacion del servidor MCP
- [Referencia de API](./mcp/API.md) — Referencia completa de herramientas
- [Conceptos](./CONCEPTS.md) — Tipos de bloques y tipos de enlaces
- [Manual de Usuario](./USER_MANUAL.md) — Uso general de PKM-AI
- [Analisis de Zettelkasten PKM](./pkm-zettelkasten-rust-analysis.md) — Fundamentos de diseno

---

**Ultima actualizacion:** 2026-03-20