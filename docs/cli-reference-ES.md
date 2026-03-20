# PKM-AI Referencia de Comandos CLI

> Referencia completa de la interfaz de linea de comandos para PKM-AI PKM

## Tabla de Contenidos

1. [Banderas Globales](#1-banderas-globales)
2. [Comando Init](#2-comando-init)
3. [Comandos de Bloques](#3-comandos-de-bloques)
   - [create - Crear un nuevo bloque](#31-create)
   - [quick - Captura rapida](#32-quick)
   - [list - Listar bloques](#33-list)
   - [search - Busqueda difusa](#34-search)
   - [grep - Buscar contenido](#35-grep)
   - [show - Mostrar detalles del bloque](#36-show)
   - [lint - Validar integridad](#37-lint)
   - [promote - Cambiar tipo de bloque](#38-promote)
4. [Comandos de Enlaces](#4-comandos-de-enlaces)
   - [link - Crear enlaces](#41-link)
5. [Comandos de Control de Versiones](#5-comandos-de-control-de-versiones)
   - [status - Estado del arbol de trabajo](#51-status)
   - [log - Historial de commits](#52-log)
   - [diff - Mostrar cambios](#53-diff)
   - [add - Preparar cambios](#54-add)
   - [commit - Crear commit](#55-commit)
   - [branch - Operaciones de ramas](#56-branch)
   - [checkout - Cambiar ramas](#57-checkout)
   - [merge - Fusionar ramas](#58-merge)
   - [tag - Operaciones de etiquetas](#59-tag)
   - [log-grep - Buscar en commits](#510-log-grep)
   - [orphan - Listar huerfanos](#511-orphan)
   - [reset - Restablecer HEAD](#512-reset)
   - [rebase - Rebase de rama](#513-rebase)
   - [push - Enviar refs](#514-push)
   - [pull - Traer cambios](#515-pull)
   - [fetch - Obtener remoto](#516-fetch)
   - [clone - Clonar repositorio](#517-clone)
   - [remote-list - Listar remotos](#518-remote-list)
   - [remote-add - Agregar remoto](#519-remote-add)
6. [Comandos de Base de Datos](#6-comandos-de-base-de-datos)
   - [db init - Inicializar base de datos](#61-db-init)
   - [db stats - Mostrar estadisticas](#62-db-stats)
   - [db export - Exportar base de datos](#63-db-export)
   - [db import - Importar base de datos](#64-db-import)
7. [Comandos de Estructura](#7-comandos-de-estructura)
   - [traverse - Recorrer spine](#71-traverse)
   - [toc - Generar TOC](#72-toc)
   - [synthesize - Sintetizar documento](#73-synthesize)
   - [gravity-check - Verificar agrupacion](#74-gravity-check)
8. [Comandos de Fantasmas](#8-comandos-de-fantasmas)
   - [ghost list - Listar fantasmas](#81-ghost-list)
   - [ghost show - Mostrar fantasma](#82-ghost-show)
   - [ghost fill - Llenar fantasma](#83-ghost-fill)
   - [ghost dismiss - Descartar fantasma](#84-ghost-dismiss)
9. [Comandos Interactivos](#9-comandos-interactivos)
   - [architect - TUI interactiva](#91-architect)

---

## 1. Banderas Globales

Las banderas globales pueden usarse con cualquier comando y deben colocarse antes del subcomando.

| Bandera | Corta | Descripcion | Variable de Entorno |
|---------|-------|-------------|---------------------|
| `--db-path <path>` | `-d` | Ruta de la base de datos (por defecto `~/.pkmai/data.db`) | `NEXUS_DB_PATH` |
| `--verbose` | `-v` | Habilitar salida detallada (mostrar logs de info) | - |
| `--stage` | - | Habilitar preparacion automatica despues de create/link (por defecto: true) | - |
| `--no-stage` | - | Deshabilitar preparacion automatica despues de create/link | - |

### Ejemplos

```bash
# Usar modo detallado
pkmai --verbose create -t permanent --title "Mi Nota"

# Especificar ruta personalizada de base de datos
pkmai --db-path /tmp/pkmai.db list

# Deshabilitar preparacion automatica para un solo comando
pkmai create -t permanent --title "Nota" --no-stage

# Deshabilitar preparacion automatica globalmente (habilitar con --stage)
pkmai link <src> <dst> --no-stage
```

### Resolucion de Ruta de Base de Datos

La ruta de la base de datos se resuelve en el siguiente orden de prioridad:
1. Bandera `--db-path` explicita
2. `.pkmai/config.toml` en el directorio actual
3. `~/.pkmai/config.toml` en el directorio home
4. Por defecto: `~/.pkmai/data.db`

---

## 2. Comando Init

### init - Inicializar PKM-AI

Inicializa la configuracion PKM en el sistema de archivos.

```bash
pkmai init [OPCIONES]
```

#### Opciones

| Bandera | Descripcion |
|---------|-------------|
| `--home` | Inicializar en el directorio home (`~/.pkmai/`) en lugar del directorio actual |
| `--force` | Forzar sobrescritura de config existente (AUN NO IMPLEMENTADO) |

#### Ejemplos

```bash
# Inicializar en el directorio actual
pkmai init

# Inicializar en el directorio home
pkmai init --home

# Salida:
# PKM-AI inicializado en /home/usuario/proyecto/
# Archivo de config: /home/usuario/proyecto/.pkmai/config.toml
# Base de datos: /home/usuario/proyecto/.pkmai/data.db
```

#### Notas

- Crea `.pkmai/config.toml` con la configuracion por defecto
- El archivo de configuracion almacena la ruta de la base de datos relativa a la ubicacion del config
- Falla si la config ya existe (usar `--force` para sobrescribir cuando este implementado)

---

## 3. Comandos de Bloques

### 3.1 create

Crear un nuevo bloque en la base de conocimiento.

```bash
pkmai create [OPCIONES]
pkmai create -t <tipo> --title <titulo> [--content <contenido>] [--tags <etiquetas>]
```

#### Sintaxis

```
pkmai create -t <block_type> --title <titulo> [--content <contenido>] [-T <etiquetas>] [-i]
```

#### Opciones

| Bandera | Corta | Requerido | Descripcion |
|---------|-------|-----------|-------------|
| `--title <titulo>` | - | Si | Titulo del bloque |
| `--type <tipo>` | `-t` | Si | Tipo de bloque (ver abajo) |
| `--content <contenido>` | - | No | Contenido (lee de stdin si no se proporciona) |
| `--tags <etiquetas>` | `-T` | No | Etiquetas (separadas por comas) |
| `--interactive` | `-i` | No | Habilitar modo interactivo AI pre-vuelo con sugerencias |

#### Tipos de Bloques

| Tipo | Valor de Bandera | Descripcion |
|------|------------------|-------------|
| Fleeting | `fleeting` o `f` | Captura rapida, notas temporales |
| Literature | `literature` o `l` | Notas de fuentes externas |
| Permanent | `permanent` o `p` | Notas de conocimiento atomico |
| Structure | `structure` o `s` | Raiz de documento, organizacion (MOC) |
| Hub | `hub` o `h` | Resumen de tema, indice |
| Task | `task` o `t` | Elementos de accion, todos |
| Reference | `reference` o `r` | Enlaces externos, citas |
| Outline | `outline` o `o` | Estructura jerarquica |

#### Ejemplos

```bash
# Crear una nota permanente
pkmai create -t permanent --title "Fundamentos del Modelo Actor" \
  --content "El modelo actor trata a los actores como primitivas universales."

# Crear un bloque de tarea
pkmai create -t task --title "TODO: Corregir bug de autenticacion" -T "bug,auth"

# Crear con etiquetas
pkmai create -t literature --title "Notas del Libro de Rust" \
  --content "Capitulo 5: Propiedad" -T "rust,programacion"

# Modo interactivo con sugerencias de IA
pkmai create -t permanent --title "Concurrencia en Rust" --interactive

# Crear desde stdin
echo "Contenido desde stdin" | pkmai create -t fleeting --title "Nota Stdin"
```

#### Pre-Vuelo AI (con `--verbose`)

Cuando `--verbose` esta habilitado, se muestran las sugerencias de IA:

```
🤖 Pre-Vuelo AI:
⚠️  Notas similares encontradas (posibles duplicados):
   - "Modelo de Propiedad de Rust" (0.94)
📍 Ubicacion sugerida: "Programacion Rust" (afinidad: 0.72)
🏷️  Etiquetas sugeridas: rust, memoria, propiedad
🔗 Enlaces sugeridos: 3 notas
```

#### Auto-Deteccion

El tipo de bloque puede auto-detectarse desde palabras clave del titulo:

| Palabras Clave | Tipo Detectado |
|----------------|----------------|
| `TODO`, `FIX`, `IMPLEMENT`, `COMPLETE` | `task` |
| `IDEA`, `NOTE`, `OBSERVATION` | `literature` |
| `INDEX`, `MOC`, `OVERVIEW`, `SUMMARY` | `structure` |

#### Notas

- Los bloques creados se prepararan automaticamente por defecto (usar `--no-stage` para deshabilitar)
- El modo interactivo (`-i`) solicita confirmacion de duplicado si similitud > 0.95
- Cuando no se proporciona contenido, lee de stdin

---

### 3.2 quick

Captura rapida: crear + preparar + commit en un comando.

```bash
pkmai quick <contenido> [OPCIONES]
pkmai q <contenido> [OPCIONES]
```

#### Sintaxis

```
pkmai quick <contenido> [-t <tipo>] [-T <etiquetas>]
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<contenido>` | Si | Contenido de la nota |

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--type <tipo>` | `-t` | `fleeting` | Tipo de bloque (fleeting, literature, permanent, task, reference) |
| `--tags <etiquetas>` | `-T` | - | Etiquetas (separadas por comas) |

#### Ejemplos

```bash
# Captura rapida (fleeting por defecto)
pkmai quick "Mi idea rapida"

# Con tipo especifico
pkmai quick "Referencia de libro" -t literature

# Con etiquetas
pkmai quick "Tarea importante" -t task -T "trabajo,urgente"

# Usando alias
pkmai q "Nota rapida"
```

#### Comportamiento

El comando `quick` automaticamente:
1. Crea el bloque con el tipo especificado
2. Lo agrega a preparacion
3. Hace commit con el mensaje: "Captura rapida: [primeros 50 caracteres del contenido]"

---

### 3.3 list

Listar bloques con filtrado opcional.

```bash
pkmai list [OPCIONES]
pkmai list [-t <tipo>] [-T <etiquetas>] [-s <busqueda>] [-n <limite>] [-o <formato>]
```

#### Sintaxis

```
pkmai list [-t <tipo>] [-T <etiquetas>] [-s <busqueda>] [-n <limite>] [-o <formato>]
```

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--type <tipo>` | `-t` | - | Filtrar por tipo de bloque |
| `--tags <etiquetas>` | `-T` | - | Filtrar por etiquetas (separadas por comas, logica OR) |
| `--search <busqueda>` | `-s` | - | Busqueda difusa por titulo |
| `--limit <limite>` | `-n` | `50` | Limitar numero de resultados |
| `--output <formato>` | `-o` | `table` | Formato de salida: `table`, `json`, `simple` |

#### Ejemplos

```bash
# Listar todos los bloques (formato tabla)
pkmai list

# Listar solo bloques permanentes
pkmai list -t permanent

# Listar bloques con etiqueta rust O python
pkmai list -T rust,python

# Busqueda difusa en titulos
pkmai list -s ownership

# Limitar a 10 resultados como JSON
pkmai list -n 10 -o json

# Formato de salida simple
pkmai list -o simple
```

#### Formatos de Salida

**table** (por defecto):
```
┌─────────────────────────────────────┬───────────────────────────┬──────────┐
│ ID                                  │ Titulo                    │ Tipo     │
├─────────────────────────────────────┼───────────────────────────┼──────────┤
│ 01ARZ3NDEKTSV4RRFFQ69G5FAV         │ Modelo de Propiedad Rust  │ permanent│
│ 01ARZ3NDEKTSV4RRFFQ69G5FAW         │ Basicos del Modelo Actor  │ permanent│
└─────────────────────────────────────┴───────────────────────────┴──────────┘
```

**json**:
```json
[
  {
    "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "title": "Modelo de Propiedad Rust",
    "type": "permanent",
    "tags": ["rust", "memoria"],
    "created_at": "2024-01-15T10:30:00Z"
  }
]
```

---

### 3.4 search

Busqueda difusa de bloques por titulo.

```bash
pkmai search <consulta> [OPCIONES]
pkmai f <consulta> [OPCIONES]
```

#### Sintaxis

```
pkmai search <consulta> [-n <limite>]
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<consulta>` | Si | Consulta de busqueda (soporta coincidencia difusa) |

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--limit <limite>` | `-n` | `50` | Limitar numero de resultados |

#### Ejemplos

```bash
# Busqueda difusa basica
pkmai search "rust own"

# Busqueda con limite
pkmai search "modelo actor" -n 10

# Usando alias
pkmai f "concurr"
```

#### Notas

- Usa coincidencia difusa, por lo que "rust own" encuentra "Modelo de Propiedad Rust"
- No distingue mayusculas de minusculas
- Mas permisivo que `--search` en el comando `list`

---

### 3.5 grep

Buscar contenido de bloques usando patrones regex.

```bash
pkmai grep <patron> [OPCIONES]
pkmai g <patron> [OPCIONES]
```

#### Sintaxis

```
pkmai grep <patron> [-c] [-i] [-n <limite>]
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<patron>` | Si | Patron de busqueda (regex) |

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--content-only` | `-c` | false | Buscar solo en contenido (no en titulos) |
| `--case-sensitive` | `-i` | false | Busqueda sensible a mayusculas (por defecto: insensible) |
| `--limit <limite>` | `-n` | `50` | Limitar numero de resultados |

#### Ejemplos

```bash
# Grep basico
pkmai grep "TODO"

# Busqueda sensible a mayusculas
pkmai grep "Rust" -i

# Buscar solo en contenido
pkmai grep "propiedad" -c

# Limitar resultados
pkmai grep "error" -n 20
```

---

### 3.6 show

Mostrar informacion detallada de un bloque.

```bash
pkmai show <id> [OPCIONES]
pkmai s <id> [OPCIONES]
```

#### Sintaxis

```
pkmai show <id> [--related]
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<id>` | Si | ID del bloque (ULID) |

#### Opciones

| Bandera | Corta | Descripcion |
|---------|-------|-------------|
| `--related` | `-r` | Mostrar bloques relacionados |

#### Ejemplos

```bash
# Mostrar detalles del bloque
pkmai show 01ARZ3NDEKTSV4RRFFQ69G5FAV

# Mostrar con bloques relacionados
pkmai show 01ARZ3NDEKTSV4RRFFQ69G5FAV --related
```

#### Ejemplo de Salida

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 Bloque: 01ARZ3NDEKTSV4RRFFQ69G5FAV
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 Titulo:   Modelo de Propiedad de Rust
 Tipo:     permanent
 Etiquetas: rust, memoria, propiedad
 Creado:   2024-01-15 10:30:00
 Modificado: 2024-01-15 14:22:00

 Contenido:
 ─────────────────────────────────────────────────────────
 El sistema de propiedad de Rust es un conjunto de reglas
 que el compilador aplica en tiempo de compilacion. Gestiona
 la memoria sin recolector de basura.

 Enlaces:
 ─────────────────────────────────────────────────────────
 → 01ARZ3NDEKTSV4RRFFQ69G5FAW (apoya)
 → 01ARZ3NDEKTSV4RRFFQ69G5FAX (extiende)

 ← 01ARZ3NDEKTSV4RRFFQ69G5FAY (apoyado_por)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

### 3.7 lint

Validar integridad estructural de la base de conocimiento.

```bash
pkmai lint [OPCIONES]
```

#### Sintaxis

```
pkmai lint [--fix]
```

#### Opciones

| Bandera | Corta | Descripcion |
|---------|-------|-------------|
| `--fix` | `-f` | Corregir problemas automaticamente |

#### Ejemplos

```bash
# Verificar problemas
pkmai lint

# Auto-corregir problemas
pkmai lint --fix
```

#### Notas

- Verifica problemas de integridad estructural
- Con `--fix`, intenta resolver problemas automaticamente
- Reporta bloques huerfanos, enlaces rotos, inconsistencias de tipo

---

### 3.8 promote

Promover un bloque a un tipo de orden superior.

```bash
pkmai promote <id> [OPCIONES]
```

#### Sintaxis

```
pkmai promote <id> [-t <tipo>] [--stage]
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<id>` | Si | ID del bloque a promover |

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--type <tipo>` | `-t` | `permanent` | Tipo de bloque destino |
| `--stage` | - | false | Agregar a preparacion automaticamente |

#### Transiciones Validas

| Desde | Hacia |
|-------|-------|
| `fleeting` | `literature`, `permanent`, `ghost` |
| `literature` | `permanent`, `ghost` |
| `ghost` | `permanent` |
| `structure`, `hub`, `task`, `reference`, `outline` | `permanent` |

#### Ejemplos

```bash
# Promover a permanent (por defecto)
pkmai promote 01ARZ3NDEKTSV4RRFFQ69G5FAV

# Promover a literature
pkmai promote 01ARZ3NDEKTSV4RRFFQ69G5FAV -t literature

# Promover con auto-preparacion
pkmai promote 01ARZ3NDEKTSV4RRFFQ69G5FAV -t permanent --stage
```

---

## 4. Comandos de Enlaces

### 4.1 link

Crear enlaces semanticos entre bloques.

```bash
pkmai link <desde> <hacia> [OPCIONES]
pkmai ln <desde> <hacia> [OPCIONES]
```

#### Sintaxis

```
pkmai link <source_id> <target_id> [-t <tipo>] [-w <peso>] [-c <contexto>]
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<desde>` | Si | ID del bloque origen |
| `<hacia>` | Si | ID del bloque destino |

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--type <tipo>` | `-t` | `related` | Tipo de enlace (ver abajo) |
| `--weight <peso>` | `-w` | `0.0` | Peso de secuencia (para enlaces ordenados) |
| `--context <contexto>` | `-c` | - | Contexto/descripcion del enlace |

#### Tipos de Enlaces

| Tipo | Descripcion | Caso de Uso |
|------|-------------|-------------|
| `section_of` | Bloque es una seccion del destino | Capitulo en documento |
| `supports` | Evidencia de apoyo | Citas, ejemplos |
| `extends` | Extension de otro bloque | Elaboraciones |
| `refines` | Version mas especifica | Explicacion detallada |
| `contradicts` | Vista opuesta | Debates, alternativas |
| `questions` | Plantea preguntas sobre | Cuestionar supuestos |
| `references` | Citacion externa | Enlaces a fuentes |
| `related` | Contenido relacionado | Asociacion general |
| `similar_to` | Contenido similar | Conceptos paralelos |
| `next` | Relacion secuencial | Siguiente bloque en spine |
| `gravity` | Atraccion semantica | Relacionado pero no secuencial |

#### Ejemplos

```bash
# Enlace basico
pkmai link 01ARZ3NDEKTSV4RRFFQ69G5FAV 01ARZ3NDEKTSV4RRFFQ69G5FAW

# Relacion de seccion
pkmai link 01ARZ3NDEKTSV4RRFFQ69G5FAV 01ARZ3NDEKTSV4RRFFQ69G5FAW -t section_of

# Con peso para ordenamiento
pkmai link 01ARZ3NDEKTSV4RRFFQ69G5FAV 01ARZ3NDEKTSV4RRFFQ69G5FAW \
  -t next -w 1.5

# Con contexto
pkmai link 01ARZ3NDEKTSV4RRFFQ69G5FAV 01ARZ3NDEKTSV4RRFFQ69G5FAW \
  -t supports -c "Esto proporciona evidencia para la afirmacion"
```

#### Notas

- Los enlaces se prepararan automaticamente por defecto (usar `--no-stage` para deshabilitar)
- El peso se usa para el ordenamiento estructural del spine
- Mayor peso = aparece despues en la secuencia

---

## 5. Comandos de Control de Versiones

Los comandos de control de versiones usan semantica similar a Git para bloques.

### 5.1 status

Mostrar estado del arbol de trabajo.

```bash
pkmai version status [OPCIONES]
pkmai status [OPCIONES]
```

#### Sintaxis

```
pkmai version status [-r <repo>]
```

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--repo <repo>` | `-r` | directorio actual | Ruta del repositorio |

#### Ejemplos

```bash
# Verificar estado
pkmai version status

# Verificar repositorio especifico
pkmai version status -r /ruta/a/repo
```

#### Ejemplo de Salida

```
En rama: main
Estado:
  Preparados:
    + 01ARZ3NDEKTSV4RRFFQ69G5FAV (nuevo)
    ~ 01ARZ3NDEKTSV4RRFFQ69G5FAW (modificado)
  Sin preparar:
    ~ 01ARZ3NDEKTSV4RRFFQ69G5FAY (eliminado)
```

---

### 5.2 log

Mostrar historial de commits.

```bash
pkmai version log [OPCIONES]
pkmai log [OPCIONES]
```

#### Sintaxis

```
pkmai version log [-r <repo>] [--oneline] [--graph] [-n <limite>]
```

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--repo <repo>` | `-r` | directorio actual | Ruta del repositorio |
| `--oneline` | - | false | Mostrar una linea por commit |
| `--graph` | - | false | Mostrar grafico ASCII |
| `--limit <limite>` | `-n` | `50` | Limitar numero de commits |

#### Ejemplos

```bash
# Log completo
pkmai version log

# Compacto oneline
pkmai version log --oneline

# Con grafico
pkmai version log --graph

# Limitar resultados
pkmai version log -n 20
```

---

### 5.3 diff

Mostrar cambios entre commits o arbol de trabajo.

```bash
pkmai version diff [OPCIONES]
pkmai diff [OPCIONES]
```

#### Sintaxis

```
pkmai version diff [-r <repo>] [-b <block_id>]
```

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--repo <repo>` | `-r` | directorio actual | Ruta del repositorio |
| `--block <block_id>` | `-b` | - | ID del bloque a comparar |

#### Ejemplos

```bash
# Mostrar todos los cambios
pkmai version diff

# Comparar bloque especifico
pkmai version diff -b 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

---

### 5.4 add

Preparar cambios para commit.

```bash
pkmai version add [OPCIONES]
pkmai add <block_id>
```

#### Sintaxis

```
pkmai version add [-r <repo>] <block_id>
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<block_id>` | Si | ID del bloque a preparar |

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--repo <repo>` | `-r` | directorio actual | Ruta del repositorio |

#### Ejemplos

```bash
# Preparar un bloque
pkmai version add 01ARZ3NDEKTSV4RRFFQ69G5FAV

# Preparar en repositorio especifico
pkmai version add -r /ruta/a/repo 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

---

### 5.5 commit

Crear un nuevo commit.

```bash
pkmai version commit [OPCIONES]
pkmai commit [OPCIONES]
```

#### Sintaxis

```
pkmai version commit [-r <repo>] -m <mensaje> [-a <autor>] [--amend] [--no-edit]
```

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--repo <repo>` | `-r` | directorio actual | Ruta del repositorio |
| `--message <mensaje>` | `-m` | - | Mensaje de commit |
| `--author <autor>` | `-a` | `user` | Nombre del autor |
| `--amend` | - | false | Enmendar el ultimo commit |
| `--no-edit` | - | false | Enmendar sin cambiar mensaje |

#### Ejemplos

```bash
# Commit basico
pkmai version commit -m "Agregar notas de concurrencia en Rust"

# Con autor
pkmai version commit -m "Actualizar" -a "desarrollador"

# Enmendar ultimo commit
pkmai version commit --amend --no-edit
```

#### Notas

- `--amend` y `--no-edit` son mutuamente excluyentes
- `--no-edit` usa el ultimo mensaje de commit

---

### 5.6 branch

Listar, crear o eliminar ramas.

```bash
pkmai version branch [OPCIONES]
pkmai branch [OPCIONES]
```

#### Sintaxis

```
pkmai version branch [-r <repo>] [nombre] [--delete] [--force-delete]
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<nombre>` | No | Nombre de la rama (para crear) |

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--repo <repo>` | `-r` | directorio actual | Ruta del repositorio |
| `--delete` | - | false | Eliminar una rama |
| `--force-delete` | - | false | Forzar eliminacion (incluso si no esta fusionada) |

#### Ejemplos

```bash
# Listar ramas
pkmai version branch

# Crear rama
pkmai version branch nueva-caracteristica

# Eliminar rama
pkmai version branch caracteristica-antigua --delete

# Forzar eliminacion
pkmai version branch legacy --force-delete
```

---

### 5.7 checkout

Cambiar ramas o crear nueva rama.

```bash
pkmai version checkout [OPCIONES]
pkmai checkout <nombre>
```

#### Sintaxis

```
pkmai version checkout [-r <repo>] <nombre> [-b]
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<nombre>` | Si | Nombre de la rama |

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--repo <repo>` | `-r` | directorio actual | Ruta del repositorio |
| `--create-new` | `-b` | false | Crear y cambiar a nueva rama |

#### Ejemplos

```bash
# Cambiar a rama existente
pkmai version checkout main

# Crear y cambiar a nueva rama
pkmai version checkout -b rama-caracteristica
```

---

### 5.8 merge

Fusionar una rama en HEAD actual.

```bash
pkmai version merge [OPCIONES]
pkmai merge <nombre>
```

#### Sintaxis

```
pkmai version merge [-r <repo>] -n <nombre> [-s <estrategia>]
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<nombre>` | Si | Nombre de la rama a fusionar |

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--repo <repo>` | `-r` | directorio actual | Ruta del repositorio |
| `--strategy <estrategia>` | `-s` | `merge` | Estrategia de fusion: `ours`, `theirs`, `merge` |

#### Ejemplos

```bash
# Fusion estandar
pkmai version merge rama-caracteristica

# Estrategia theirs (tomar sus cambios)
pkmai version merge rama-caracteristica -s theirs
```

---

### 5.9 tag

Operaciones de etiquetas (listar, crear, eliminar).

```bash
pkmai version tag [OPCIONES]
pkmai tag [OPCIONES]
```

#### Sintaxis

```
pkmai version tag [-r <repo>] [nombre] [-c <commit>] [-m <mensaje>] [--delete]
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<nombre>` | No | Nombre de la etiqueta |

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--repo <repo>` | `-r` | directorio actual | Ruta del repositorio |
| `--commit <commit>` | `-c` | HEAD | ID del commit a etiquetar |
| `--message <mensaje>` | `-m` | - | Mensaje de etiqueta (para etiquetas anotadas) |
| `--delete` | - | false | Eliminar una etiqueta |

#### Ejemplos

```bash
# Listar etiquetas
pkmai version tag

# Crear etiqueta
pkmai version tag v1.0.0

# Etiqueta anotada
pkmai version tag -m "Lanzamiento 1.0.0" v1.0.0

# Etiquetar commit especifico
pkmai version tag -c abc123 v0.9.0

# Eliminar etiqueta
pkmai version tag v0.9.0 --delete
```

---

### 5.10 log-grep

Buscar en mensajes de commits.

```bash
pkmai version log-grep [OPCIONES]
```

#### Sintaxis

```
pkmai version log-grep [-r <repo>] <patron> [-n <limite>]
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<patron>` | Si | Patron a buscar |

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--repo <repo>` | `-r` | directorio actual | Ruta del repositorio |
| `--limit <limite>` | `-n` | `50` | Limitar numero de commits |

#### Ejemplos

```bash
# Buscar en mensajes de commits
pkmai version log-grep "fix bug"

# Con limite
pkmai version log-grep "feat" -n 20
```

---

### 5.11 orphan

Listar bloques huerfanos (bloques sin bordes entrantes).

```bash
pkmai version orphan [OPCIONES]
pkmai orphan [OPCIONES]
```

#### Sintaxis

```
pkmai version orphan [-r <repo>]
```

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--repo <repo>` | `-r` | directorio actual | Ruta del repositorio |

#### Ejemplos

```bash
# Listar bloques huerfanos
pkmai version orphan
```

---

### 5.12 reset

Restablecer HEAD a un commit anterior.

```bash
pkmai version reset [OPCIONES]
pkmai reset [OPCIONES]
```

#### Sintaxis

```
pkmai version reset [-r <repo>] [--soft] [--hard] [-c <commit>]
```

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--repo <repo>` | `-r` | directorio actual | Ruta del repositorio |
| `--soft` | - | false | Reset suave: mantener cambios preparados |
| `--hard` | - | false | Reset duro: descartar todos los cambios |
| `--commit <commit>` | `-c` | HEAD~1 | Commit al que restablecer |

#### Ejemplos

```bash
# Reset suave (mantener cambios preparados)
pkmai version reset --soft

# Reset duro (descartar cambios)
pkmai version reset --hard

# Restablecer a commit especifico
pkmai version reset -c abc123
```

#### Notas

- `--soft` y `--hard` son mutuamente excluyentes
- Por defecto es `--soft` con `--commit` por defecto en HEAD~1

---

### 5.13 rebase

Hacer rebase de la rama actual sobre otra rama.

```bash
pkmai version rebase [OPCIONES]
pkmai rebase <rama>
```

#### Sintaxis

```
pkmai version rebase [-r <repo>] <rama>
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<rama>` | Si | Rama sobre la que hacer rebase |

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--repo <repo>` | `-r` | directorio actual | Ruta del repositorio |

#### Ejemplos

```bash
# Rebase sobre main
pkmai version rebase main
```

---

### 5.14 push

Enviar refs a un remoto.

```bash
pkmai version push [OPCIONES]
pkmai push [OPCIONES]
```

#### Sintaxis

```
pkmai version push [-r <repo>] [-m <remoto>] [-r <refs>] [-f]
```

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--repo <repo>` | `-r` | directorio actual | Ruta del repositorio |
| `--remote <remoto>` | `-m` | `origin` | Nombre del remoto |
| `--refs <refs>` | `-r` | todos | Refs a enviar |
| `--force` | `-f` | false | Forzar envio (saltar verificacion fast-forward) |

#### Ejemplos

```bash
# Enviar a origin
pkmai version push

# Enviar rama especifica
pkmai version push -m origin main

# Forzar envio
pkmai version push --force
```

---

### 5.15 pull

Traer de un remoto.

```bash
pkmai version pull [OPCIONES]
pkmai pull [OPCIONES]
```

#### Sintaxis

```
pkmai version pull [-r <repo>] [-m <remoto>] [-b <rama>] [-s <estrategia>]
```

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--repo <repo>` | `-r` | directorio actual | Ruta del repositorio |
| `--remote <remoto>` | `-m` | `origin` | Nombre del remoto |
| `--branch <rama>` | `-b` | - | Rama a traer |
| `--strategy <estrategia>` | `-s` | `merge` | Estrategia de fusion: `ours`, `theirs`, `merge` |

#### Ejemplos

```bash
# Traer de origin
pkmai version pull

# Traer con estrategia
pkmai version pull -s theirs
```

---

### 5.16 fetch

Obtener de un remoto sin aplicar cambios.

```bash
pkmai version fetch [OPCIONES]
pkmai fetch [OPCIONES]
```

#### Sintaxis

```
pkmai version fetch [-r <repo>] [-m <remoto>]
```

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--repo <repo>` | `-r` | directorio actual | Ruta del repositorio |
| `--remote <remoto>` | `-m` | `origin` | Nombre del remoto |

#### Ejemplos

```bash
# Obtener de origin
pkmai version fetch
```

---

### 5.17 clone

Clonar un repositorio.

```bash
pkmai version clone <fuente> [OPCIONES]
pkmai clone <fuente> [OPCIONES]
```

#### Sintaxis

```
pkmai version clone <fuente> [-d <destino>] [-b <rama>] [-D <profundidad>]
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<fuente>` | Si | Ruta del repositorio fuente |

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--destination <dest>` | `-d` | nombre del directorio fuente | Ruta de destino |
| `--branch <rama>` | `-b` | `main` | Rama a clonar |
| `--depth <profundidad>` | `-D` | - | Profundidad del clone (para clone superficial) |

#### Ejemplos

```bash
# Clonar a destino por defecto
pkmai version clone /ruta/a/fuente

# Clonar a destino especifico
pkmai version clone /ruta/a/fuente -d mi-clon

# Clone superficial
pkmai version clone /ruta/a/fuente -D 10
```

---

### 5.18 remote-list

Listar remotos configurados.

```bash
pkmai version remote list [OPCIONES]
pkmai remote list [OPCIONES]
```

#### Sintaxis

```
pkmai version remote list [-r <repo>]
```

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--repo <repo>` | `-r` | directorio actual | Ruta del repositorio |

#### Ejemplos

```bash
# Listar remotos
pkmai version remote list
```

---

### 5.19 remote-add

Agregar un remoto.

```bash
pkmai version remote add [OPCIONES]
pkmai remote add <nombre> <url>
```

#### Sintaxis

```
pkmai version remote add [-r <repo>] <nombre> <url>
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<nombre>` | Si | Nombre del remoto |
| `<url>` | Si | Ruta o URL del remoto |

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--repo <repo>` | `-r` | directorio actual | Ruta del repositorio |

#### Ejemplos

```bash
# Agregar remoto
pkmai version remote add origin /ruta/a/remoto
```

---

## 6. Comandos de Base de Datos

### 6.1 db init

Inicializar la base de datos.

```bash
pkmai db init
```

#### Sintaxis

```
pkmai db init
```

#### Ejemplos

```bash
# Inicializar base de datos
pkmai db init
```

#### Notas

- Crea el archivo de base de datos si no existe
- Falla si la base de datos ya esta inicializada

---

### 6.2 db stats

Mostrar estadisticas de la base de datos.

```bash
pkmai db stats
```

#### Sintaxis

```
pkmai db stats
```

#### Ejemplos

```bash
# Mostrar estadisticas
pkmai db stats
```

#### Ejemplo de Salida

```
Estadisticas de Base de Datos:
  Bloques totales:      1,234
  Por tipo:
    - fleeting:        200
    - literature:      350
    - permanent:       500
    - structure:       50
    - hub:             30
    - task:            80
    - reference:       20
    - outline:         4
  Enlaces totales:      3,456
  Nodos fantasma:       12
  Ultimo commit:        2024-01-15T14:22:00Z
```

---

### 6.3 db export

Exportar base de datos a un archivo.

```bash
pkmai db export [OPCIONES]
```

#### Sintaxis

```
pkmai db export --format <formato>
```

#### Opciones

| Bandera | Requerido | Descripcion |
|---------|-----------|-------------|
| `--format <formato>` | Si | Formato de exportacion |

#### Ejemplos

```bash
# Exportar a JSON
pkmai db export --format json > respaldo.json
```

---

### 6.4 db import

Importar base de datos desde un archivo.

```bash
pkmai db import <archivo>
```

#### Sintaxis

```
pkmai db import <archivo>
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<archivo>` | Si | Archivo a importar |

#### Ejemplos

```bash
# Importar desde respaldo
pkmai db import respaldo.json
```

---

## 7. Comandos de Estructura

### 7.1 traverse

Recorrer el spine estructural.

```bash
pkmai traverse [OPCIONES]
```

#### Sintaxis

```
pkmai traverse [--from <id>] [-d <profundidad>] [-t <tipo_enlace>] [-c]
```

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--from <id>` | - | raiz del spine | ID del bloque inicial |
| `--depth <profundidad>` | `-d` | `10` | Profundidad maxima |
| `--type <tipo>` | `-t` | - | Filtrar por tipo de enlace |
| `--content` | `-c` | false | Mostrar contenido |

#### Ejemplos

```bash
# Recorrer desde la raiz
pkmai traverse

# Desde bloque especifico
pkmai traverse --from 01ARZ3NDEKTSV4RRFFQ69G5FAV

# Profundidad limitada
pkmai traverse -d 3

# Mostrar contenido
pkmai traverse -c
```

---

### 7.2 toc

Generar Tabla de Contenidos para un bloque de estructura.

```bash
pkmai toc <id>
```

#### Sintaxis

```
pkmai toc <id>
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<id>` | Si | ID del bloque de estructura |

#### Ejemplos

```bash
# Generar TOC
pkmai toc 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

#### Ejemplo de Salida

```
Tabla de Contenidos: Guia de Programacion Rust
==========================================

1. Introduccion
   1.1 Que es Rust?
   1.2 Por que Rust?
2. Basicos
   2.1 Variables
   2.2 Funciones
   2.3 Control de Flujo
3. Propiedad
   3.1 Reglas de Propiedad
   3.2 Prestamos
```

---

### 7.3 synthesize

Sintetizar un documento desde un bloque de estructura.

```bash
pkmai synthesize <id> [OPCIONES]
```

#### Sintaxis

```
pkmai synthesize <id> [-o <output>] [-t <plantilla>] [-f <archivo>]
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<id>` | Si | ID del bloque de estructura |

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--output <output>` | `-o` | `pdf` | Formato de salida: `pdf`, `html`, `markdown`, `typst` |
| `--template <plantilla>` | `-t` | - | Nombre de plantilla |
| `--file <archivo>` | `-f` | - | Archivo de salida |

#### Ejemplos

```bash
# Sintetizar a Markdown
pkmai synthesize 01ARZ3NDEKTSV4RRFFQ69G5FAV -o markdown

# Sintetizar a PDF con plantilla
pkmai synthesize 01ARZ3NDEKTSV4RRFFQ69G5FAV \
  -o pdf --template technical-whitepaper

# Guardar en archivo
pkmai synthesize 01ARZ3NDEKTSV4RRFFQ69G5FAV -f salida.pdf
```

#### Notas

- La salida PDF requiere la caracteristica `typst`
- Si no se especifica `--file`, salida a stdout

---

### 7.4 gravity-check

Verificar ganchos de gravedad y agrupacion semantica.

```bash
pkmai gravity-check <id> [OPCIONES]
```

#### Sintaxis

```
pkmai gravity-check <id> [-t <umbral>]
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<id>` | Si | ID del bloque a verificar |

#### Opciones

| Bandera | Corta | Por Defecto | Descripcion |
|---------|-------|-------------|-------------|
| `--threshold <umbral>` | `-t` | `0.7` | Umbral de similitud (0.0-1.0) |

#### Ejemplos

```bash
# Verificar gravedad para bloque
pkmai gravity-check 01ARZ3NDEKTSV4RRFFQ69G5FAV

# Con umbral personalizado
pkmai gravity-check 01ARZ3NDEKTSV4RRFFQ69G5FAV -t 0.8
```

#### Notas

- Umbral mas alto = requisito de similitud mas estricto
- Muestra bloques con atraccion semantica por encima del umbral

---

## 8. Comandos de Fantasmas

Los nodos fantasma son brechas detectadas por IA en la base de conocimiento.

### 8.1 ghost list

Listar todos los nodos fantasma.

```bash
pkmai ghost list
```

#### Sintaxis

```
pkmai ghost list
```

#### Ejemplos

```bash
# Listar nodos fantasma
pkmai ghost list
```

#### Ejemplo de Salida

```
👻 Nodos Fantasma:
   01ARZ3NDEKTSV4RRFFQ69G5FAV "Explicacion faltante" (confianza: 0.85)
   01ARZ3NDEKTSV4RRFFQ69G5FAW "Pensamiento incompleto" (confianza: 0.72)
```

---

### 8.2 ghost show

Mostrar informacion detallada de un nodo fantasma.

```bash
pkmai ghost show <id>
```

#### Sintaxis

```
pkmai ghost show <id>
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<id>` | Si | ID del nodo fantasma |

#### Ejemplos

```bash
# Mostrar detalles del fantasma
pkmai ghost show 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

---

### 8.3 ghost fill

Llenar un nodo fantasma con contenido real.

```bash
pkmai ghost fill <id> --content <contenido>
```

#### Sintaxis

```
pkmai ghost fill <id> --content <contenido>
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<id>` | Si | ID del nodo fantasma |

#### Opciones

| Bandera | Requerido | Descripcion |
|---------|-----------|-------------|
| `--content <contenido>` | Si | Contenido para llenar |

#### Ejemplos

```bash
# Llenar nodo fantasma
pkmai ghost fill 01ARZ3NDEKTSV4RRFFQ69G5FAV \
  --content "El modelo actor es un modelo de programacion concurrente."
```

---

### 8.4 ghost dismiss

Descartar un nodo fantasma (marcar como no necesario).

```bash
pkmai ghost dismiss <id>
```

#### Sintaxis

```
pkmai ghost dismiss <id>
```

#### Argumentos

| Argumento | Requerido | Descripcion |
|-----------|-----------|-------------|
| `<id>` | Si | ID del nodo fantasma |

#### Ejemplos

```bash
# Descartar fantasma
pkmai ghost dismiss 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

---

## 9. Comandos Interactivos

### 9.1 architect

Iniciar TUI interactiva para exploracion del grafo de conocimiento.

```bash
pkmai architect
```

#### Sintaxis

```
pkmai architect
```

#### Ejemplos

```bash
# Iniciar TUI
pkmai architect
```

#### Navegacion

| Tecla | Accion |
|-------|--------|
| `j` / `↓` | Mover hacia abajo |
| `k` / `↑` | Mover hacia arriba |
| `h` / `←` | Contraer |
| `l` / `→` | Expandir |
| `g` | Ir al inicio |
| `G` | Ir al final |
| `PageUp` / `PageDown` | Paginar resultados |
| `Enter` | Ver detalle |
| `Esc` | Volver |

#### Modo Detalle

| Tecla | Accion |
|-------|--------|
| `b` | Navegar al primer backlink |
| `o` | Navegar al primer enlace saliente |

#### Comandos

| Comando | Accion |
|---------|--------|
| `:` | Entrar en modo comando |
| `:search <consulta>` | Busqueda difusa |
| `:filter <tipo>` | Filtrar por tipo |
| `:filter` | Limpiar filtro |
| `:new` | Nueva nota |
| `:all` | Mostrar todo |
| `:quit` | Salir |
| `q` | Salir |

#### Atajos Adicionales

| Tecla | Accion |
|-------|--------|
| `?` | Mostrar ayuda |
| `Tab` | Ciclar entre filtros |
| `r` | Recargar datos |

---

## Apendice A: Referencia de Tipos de Bloques

| Tipo | Bandera | Descripcion |
|------|---------|-------------|
| `fleeting` | `f` | Captura rapida, notas temporales |
| `literature` | `l` | Notas de fuentes externas |
| `permanent` | `p` | Notas de conocimiento atomico |
| `structure` | `s` | Raiz de documento, MOC |
| `hub` | `h` | Resumen de tema |
| `task` | `t` | Elementos de accion |
| `reference` | `r` | Enlaces externos, citas |
| `outline` | `o` | Estructura jerarquica |
| `ghost` | `g` | Brechas detectadas por IA |

## Apendice B: Referencia de Tipos de Enlaces

| Tipo | Descripcion |
|------|-------------|
| `section_of` | Bloque es una seccion del destino |
| `supports` | Evidencia de apoyo |
| `extends` | Extension de otro bloque |
| `refines` | Version mas especifica |
| `contradicts` | Vista opuesta |
| `questions` | Plantea preguntas |
| `references` | Citacion externa |
| `related` | Contenido relacionado |
| `similar_to` | Contenido similar |
| `next` | Relacion secuencial |
| `gravity` | Atraccion semantica |

## Apendice C: Referencia de Formatos de Salida

| Formato | Valor de Bandera | Caso de Uso |
|---------|------------------|-------------|
| `table` | `-o table` | Visualizacion por defecto |
| `json` | `-o json` | Uso programatico |
| `simple` | `-o simple` | Visualizacion compacta |
