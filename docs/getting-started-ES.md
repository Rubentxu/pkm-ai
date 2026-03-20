# Primeros Pasos con PKM-AI

> Tu Sistema de Gestion Personal de Conocimiento con Zettelkasten + IA

**Tiempo estimado:** 5 minutos para la primera captura, 30 minutos para ser productivo.

---

## Tabla de Contenidos

1. [Inicio Rapido (5 min)](#1-inicio-rapido-5-min)
2. [Instalacion](#2-instalacion)
3. [Primeros Pasos](#3-primeros-pasos)
4. [Flujo de Trabajo Basico](#4-flujo-de-trabajo-basico)
5. [Proximos Pasos](#5-proximos-pasos)

---

## 1. Inicio Rapido (5 min)

### 1.1 Que es PKM-AI?

PKM-AI es un **Sistema Operativo de Conocimiento** que trata el conocimiento como un grafo de bloques interconectados. A diferencia de las aplicaciones tradicionales de toma de notas, PKM-AI utiliza:

- **Modelo de Bloque-Atomo**: Cada pieza de conocimiento es un bloque direccionable con un identificador ULID unico
- **Columna Estructural**: El orden y la estructura son ciudadanos de primera clase
- **Versionado estilo Git**: Control de versiones completo para tu base de conocimiento
- **Asistencia de IA**: Los nodos fantasma detectan huecos, la sintesis genera documentos

### 1.2 Tu Primera Captura

Vamos a capturar tu primera idea:

```bash
# Captura rapida (crea + stage + commit en un solo paso)
pkmai quick "Mi primera nota sobre PKM-AI"
```

Deberias ver una salida similar a:

```
[CREATED] Block 01ARZ3NDEKTSV4RRFFQ69G5FAV
[fleeting] Mi primera nota sobre PKM-AI

[STAGED] Ready to commit
[COMMIT] a1b2c3d - Quick capture: Mi primera nota sobre PKM-AI
```

### 1.3 Verificar que Funciona

```bash
# Listar todos tus bloques
pkmai list

# O buscarlo
pkmai search "PKM"
```

Salida esperada:

```
01ARZ3NDEKTSV4RRFFQ69G5FAV  [f] Mi primera nota sobre PKM-AI
                              created: 2026-03-20T10:30:00Z
```

### 1.4 El Concepto Central: Tipos de Bloques

PKM-AI utiliza la **metodologia Zettelkasten** con estos tipos de bloques:

| Tipo | Alias | Proposito | Ejemplo |
|------|-------|-----------|---------|
| `fleeting` | `f` | Capturas rapidas, temporales | Notas de reunion, TODOs |
| `literature` | `l` | Notas de fuentes externas | Resumenes de libros, notas de articulos |
| `permanent` | `p` | Conocimiento atomico, perenne | Conceptos, intuiciones |
| `structure` | `s`, `moc` | Contenedores de documentos | Indice, Mapa de Contenido |
| `hub` | `h` | Puntos de entrada por tema | Indices de temas |
| `task` | `t` | Elementos de accion | Entregables, errores |
| `reference` | `r` | Referencias externas | URLs, citas |
| `outline` | `o` | Esquemas jerarquicos | Estructura de documento |
| `ghost` | `g` | Huecos detectados por IA | Explicaciones faltantes |

**El Flujo Zettelkasten:**

```
Fleeting (captura) → Literature (procesar) → Permanent (elaborar)
                                                    ↓
                                               Structure (organizar)
```

---

## 2. Instalacion

### Requisitos Previos

- **Rust 1.75+** (verificar con `rustc --version`)
- **Cargo** (incluido con Rust)
- **Git**

### 2.1 Linux

#### Opcion A: Compilar desde Fuente (Recomendado)

```bash
# 1. Instalar Rust si no esta instalado
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 2. Clonar el repositorio
git clone https://github.com/rubentxu/pkmai
cd pkmai

# 3. Compilar
cargo build --release

# 4. Instalar
sudo cp target/release/pkmai /usr/local/bin/
sudo chmod +x /usr/local/bin/pkmai
```

#### Opcion B: Instalacion a Nivel de Usuario

```bash
git clone https://github.com/rubentxu/pkmai
cd pkmai
cargo build --release
mkdir -p ~/.local/bin
mv target/release/pkmai ~/.local/bin/
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### 2.2 macOS

#### Opcion A: Homebrew

```bash
git clone https://github.com/rubentxu/pkmai
cd pkmai
brew install rust
cargo build --release
cargo install --path .
```

#### Opcion B: Binario Directo

```bash
git clone https://github.com/rubentxu/pkmai
cd pkmai
cargo build --release
mkdir -p ~/bin
mv target/release/pkmai ~/bin/
echo 'export PATH="$HOME/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

### 2.3 Windows

```powershell
# 1. Instalar Rust desde https://rustup.rs

# 2. Clonar y compilar
git clone https://github.com/rubentxu/pkmai
cd pkmai
cargo build --release

# 3. Copiar a un directorio en PATH
mkdir $env:USERPROFILE\bin
copy target\release\pkmai.exe $env:USERPROFILE\bin\

# 4. Agregar a PATH (PowerShell)
$env:Path += ";$env:USERPROFILE\bin"
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$env:USERPROFILE\bin", "User")
```

### 2.4 Verificar Instalacion

```bash
pkmai --version
pkmai --help
```

Salida esperada:

```
pkmai 1.0.0
PKM-AI - Personal Knowledge Management with AI

USAGE:
    pkmai [OPTIONS] <COMMAND>

COMMANDS:
    quick      Captura rapida: crear + stage + commit
    create     Crear un nuevo bloque
    list       Listar bloques
    show       Mostrar detalles del bloque
    link       Crear enlaces entre bloques
    ...
```

### 2.5 Inicializar la Base de Datos

```bash
pkmai db init
```

Esto crea la base de datos en `~/.pkmai/` por defecto.

---

## 3. Primeros Pasos

### 3.1 Flujo de Captura Rapida

La forma mas rapida de capturar ideas:

```bash
# Captura rapida basica (tipo fleeting)
pkmai quick "Reunion con el equipo sobre planificacion Q2"

# Con tipo especifico
pkmai quick "Patrones async en Rust" -t literature

# Con tags
pkmai quick "Intuicion importante" -T "rust,arquitectura"

# Combinado
pkmai quick "Nota de libro sobre patrones de diseno" -t literature -T "libros,patrones"
```

### 3.2 Crear Bloques Directamente

Para mas control, usa `create`:

```bash
# Crear una nota permanente
pkmai create -t permanent \
  --title "Fundamentos del Modelo de Actor" \
  --content "El modelo de actor trata a los actores como las unidades fundamentales de la computacion concurrente. Cada actor tiene un buzon y se comunica mediante paso de mensajes."

# Crear con tags
pkmai create -t permanent \
  --title "Ownership en Rust" \
  --content "Ownership es la caracteristica unica de Rust para gestion de memoria..." \
  -T "rust,memoria,safety"

# Crear una estructura (Mapa de Contenido)
pkmai create -t structure \
  --title "Indice de Programacion Rust" \
  --content "Indice principal para notas de programacion Rust"
```

### 3.3 Listar y Buscar

```bash
# Listar todos los bloques (limite por defecto: 50)
pkmai list

# Filtrar por tipo
pkmai list -t permanent
pkmai list -t fleeting

# Buscar por titulo (fuzzy)
pkmai search "rust own"

# Buscar en contenido (regex)
pkmai grep "ownership"
pkmai grep "TODO|FIXME" -i

# Filtrar por tags
pkmai list -T "rust,concurrency"
```

### 3.4 Ver Detalles del Bloque

```bash
# Mostrar bloque por ULID
pkmai show 01ARZ3NDEKTSV4RRFFQ69G5FAV

# Mostrar con enlaces relacionados
pkmai show 01ARZ3NDEKTSV4RRFFQ69G5FAV --related
```

Salida esperada:

```
[PERMANENT] 01ARZ3NDEKTSV4RRFFQ69G5FAV
──────────────────────────────────────
Title:   Fundamentos del Modelo de Actor
Type:    permanent
Tags:    concurrency,actors
Created: 2026-03-20T10:30:00Z
Updated: 2026-03-20T10:30:00Z

Content:
El modelo de actor trata a los actores como las unidades fundamentales
de la computacion concurrente. Cada actor tiene un buzon y
se comunica mediante paso de mensajes.

Links (3):
  → 01ARZ3NDEKTSV4RRFFQ69G5FA1 (extends)
  → 01ARZ3NDEKTSV4RRFFQ69G5FA2 (supports)
  ← 01ARZ3NDEKTSV4RRFFQ69G5FA3 (extends)
```

### 3.5 Enlazar Bloques

Crear relaciones entre bloques:

```bash
# Enlace basico (tipo related)
pkmai link --from <ULID_1> --to <ULID_2>

# Enlaces semanticos
pkmai link --from <ULID_1> --to <ULID_2> -t extends
pkmai link --from <ULID_1> --to <ULID_2> -t supports
pkmai link --from <ULID_1> --to <ULID_2> -t refines

# Enlaces estructurales
pkmai link --from <ULID_1> --to <ULID_STRUCTURE> -t section_of
pkmai link --from <ULID_1> --to <ULID_2> -t next
```

**Tipos de Enlaces Explicados:**

| Tipo | Significado | Caso de Uso |
|------|-------------|-------------|
| `extends` | Bloque extiende otro | Elaboraciones sobre un concepto |
| `refines` | Version mas especifica | Explicacion detallada |
| `supports` | Evidencia de apoyo | Citas, ejemplos |
| `contradicts` | Vista opuesta | Debates |
| `references` | Citacion externa | Fuentes |
| `section_of` | Bloque pertenece a estructura | Capitulo en documento |
| `next` | Relacion secuencial | Siguiente bloque en la columna |
| `related` | Relacionado (por defecto) | Asociacion general |

### 3.6 Control de Versiones

PKM-AI tiene control de versiones completo estilo Git:

```bash
# Verificar estado
pkmai version status

# Stagear cambios (automatico por defecto)
pkmai version add <ULID>

# Commit con mensaje
pkmai version commit -m "Agregar notas del modelo de actor"

# Ver historial
pkmai version log
pkmai version log --oneline

# Ramas
pkmai version branch                        # Listar
pkmai version branch mi-rama             # Crear
pkmai version checkout mi-rama             # Cambiar
pkmai version checkout -b nueva-rama        # Crear + cambiar
```

---

## 4. Flujo de Trabajo Basico

### 4.1 Ejemplo de Flujo Diario

```bash
# 1. Manana: Captura rapida de pensamientos
pkmai quick "Idea sobre cache distribuido"
pkmai quick "Nota de la reunion diaria"
pkmai quick "Referencia a articulo interesante" -t literature

# 2. Revision: Convertir fleeting a permanent
pkmai promote <FLEETING_ULID> -t permanent

# 3. Crear estructura para un proyecto
pkmai create -t structure \
  --title "Indice del Proyecto X" \
  --content "Indice principal para documentacion del Proyecto X"

# 4. Enlazar bloques relacionados
pkmai link --from <NOTE_ULID> --to <STRUCTURE_ULID> -t section_of

# 5. Hacer commit de tu trabajo
pkmai version commit -m "Captura diaria: notas del proyecto"
```

### 4.2 La Columna Estructural

La **Columna Estructural** es la columna vertebral ordenada de los documentos, basada en el principio Folgezettel de Zettelkasten:

```bash
# Crear bloques ordenados
pkmai create -t permanent --title "Introduccion" -c "..."
pkmai create -t permanent --title "Capitulo 1" -c "..."

# Enlazarlos en secuencia
pkmai link --from <INTRO_ULID> --to <CHAPTER1_ULID> -t next

# Recorrer la columna
pkmai traverse -d 5
```

### 4.3 Auto-Staging

Por defecto, todos los cambios se auto-stagean:

```bash
# Esto crea Y stagea automaticamente
pkmai create -t permanent --title "Nueva nota" --content "..."

# Desactivar auto-staging
pkmai create -t permanent --title "Nueva nota" --content "..." --no-stage
pkmai link --from <A> --to <B> --no-stage

# Staging manual requerido entonces
pkmai version add <ULID>
pkmai version commit -m "mensaje"
```

### 4.4 Promocion Zettelkasten

Promover notas a traves de la jerarquia del conocimiento:

```bash
# Fleeting → Literature
pkmai promote <ULID> -t literature

# Literature → Permanent
pkmai promote <ULID> -t permanent

# Cualquiera → Structure (como indice)
pkmai promote <ULID> -t structure
```

### 4.5 Modo Interactivo

Para creacion guiada con verificacion previa de IA:

```bash
# Modo interactivo detecta duplicados
pkmai create -t permanent --title "Ownership en Rust" --interactive

# Salida:
# 🤖 Verificacion Previa de IA:
# ⚠️  Nota similar encontrada: "Rust Ownership Model" (0.94 similitud)
# 📍 Ubicacion sugerida: "Programacion Rust" (afinidad: 0.72)
# 🏷️  Tags sugeridos: rust, memory, ownership
# 🔗 Enlaces sugeridos: 3 notas
#
# [y]es (usar existente) / [n]o (crear nueva) / [e]dit / [a]bort:
```

---

## 5. Proximos Pasos

### 5.1 Zettelkasten Avanzado

**Crear una Nota Atomica:**

```bash
# 1. Nota de literatura (de una fuente)
pkmai create -t literature \
  --title "Notas sobre 'Programming Rust' Cap.3" \
  --content "El capitulo 3 cubre patrones de concurrencia..."

# 2. Promover a permanent (tu propia sintesis)
pkmai create -t permanent \
  --title "Patrones de Concurrencia en Rust" \
  --content "Basado en Programming Rust, los patrones principales son..."

# 3. Enlazarlos
pkmai link --from <PERMANENT_ULID> --to <LITERATURE_ULID> -t supports
```

**Construir una Estructura (MOC):**

```bash
# Crear estructura indice
pkmai create -t structure \
  --title "Indice de Sistemas Distribuidos" \
  --content "Indice completo de notas de sistemas distribuidos"

# Agregar secciones
pkmai link --from <SECTION1_ULID> --to <INDEX_ULID> -t section_of
pkmai link --from <SECTION2_ULID> --to <INDEX_ULID> -t section_of

# Ver tabla de contenidos
pkmai toc <INDEX_ULID>
```

### 5.2 Caracteristicas de IA

**Nodos Fantasma (huecos detectados por IA):**

```bash
# Listar huecos detectados
pkmai ghost list

# Ver detalles del fantasma
pkmai ghost show <GHOST_ULID>

# Completar un nodo fantasma
pkmai ghost fill <GHOST_ULID> --content "La explicacion va aqui..."

# O promover a permanent
pkmai promote <GHOST_ULID> -t permanent --content "Contenido"
```

**Verificacion de Gravedad (Agrupacion Semantica):**

```bash
# Encontrar bloques relacionados
pkmai gravity-check <ULID>
pkmai gravity-check <ULID> -t 0.8  # Umbral mas alto = mas estricto
```

**Sintesis de Documentos:**

```bash
# Generar TOC
pkmai toc <STRUCTURE_ULID>

# Sintetizar a Markdown
pkmai synthesize <STRUCTURE_ULID> -o markdown

# Sintetizar a HTML
pkmai synthesize <STRUCTURE_ULID> -o html

# Sintetizar a PDF (requiere typst)
pkmai synthesize <STRUCTURE_ULID> -o pdf
```

### 5.3 TUI Interactivo

Lanzar el explorador visual del grafo de conocimiento:

```bash
pkmai architect
```

**Navegacion:**

| Tecla | Accion |
|-------|--------|
| `j` / `↓` | Mover abajo |
| `k` / `↑` | Mover arriba |
| `h` / `←` | Colapsar |
| `l` / `→` | Expandir |
| `Enter` | Ver detalle |
| `Esc` | Volver |
| `:` | Modo comando |

**Comandos en TUI:**

```
:search <query>  Busqueda fuzzy
:filter <type>  Filtrar por tipo
:new             Nueva nota
:quit            Salir
```

### 5.4 Control de Versiones Avanzado

**Fusionar Ramas:**

```bash
# Cambiar a rama
pkmai version checkout main

# Fusionar
pkmai version merge rama-feature

# Ver diferencias
pkmai version diff <ULID>
```

**Sincronizacion Remota:**

```bash
# Agregar remoto
pkmai version remote add origin https://github.com/user/pkm.git

# Push
pkmai version push

# Pull
pkmai version pull
```

### 5.5 Mantenimiento

**Estadisticas de la Base de Datos:**

```bash
pkmai db stats
```

**Validar Integridad:**

```bash
# Verificar problemas
pkmai lint

# Auto-reparar
pkmai lint --fix
```

**Exportar/Importar:**

```bash
# Exportar
pkmai db export --format json > backup.json

# Importar
pkmai db import backup.json
```

---

## Tarjeta de Referencia Rapida

```bash
# ===== CAPTURA =====
pkmai quick "Mi idea"                    # Fleeting + stage + commit
pkmai quick "Nota" -t literature         # Tipo especifico
pkmai quick "Importante" -T "tag1,tag2" # Con tags

# ===== CREAR =====
pkmai create -t permanent -T "Titulo" -c "Contenido"
pkmai create -t structure -T "Indice"

# ===== BUSCAR =====
pkmai list                               # Todos los bloques
pkmai list -t permanent                  # Por tipo
pkmai search "consulta"                  # Busqueda fuzzy
pkmai grep "patron"                      # Buscar en contenido

# ===== VER =====
pkmai show <ULID>                        # Detalles del bloque
pkmai architect                          # TUI interactivo

# ===== ENLAZAR =====
pkmai link --from <A> --to <B> -t extends

# ===== VERSION =====
pkmai version status
pkmai version commit -m "mensaje"
pkmai version log --oneline
pkmai version branch

# ===== ORGANIZAR =====
pkmai promote <ULID> -t permanent
pkmai toc <STRUCTURE_ULID>
pkmai traverse -d 5

# ===== IA =====
pkmai ghost list
pkmai gravity-check <ULID>
pkmai synthesize <STRUCTURE_ULID> -o markdown

# ===== MANTENIMIENTO =====
pkmai lint --fix
pkmai db stats
pkmai db export > backup.json
```

---

## Solucion de Problemas

### Problemas de Base de Datos

```bash
# Reinicializar (ADVERTENCIA: elimina datos)
rm -rf ~/.pkmai/
pkmai db init
```

### Rendimiento

```bash
# Ejecutar con registro detallado
pkmai --verbose list
```

### Ayuda

```bash
pkmai --help
pkmai <comando> --help
```

---

## Proximos Pasos para Usuarios Avanzados

1. **Leer el Manual de Usuario:** `docs/USER_MANUAL_ES.md`
2. **Entender Conceptos:** `docs/CONCEPTS_ES.md`
3. **Referencia CLI:** `docs/cli-cheat-sheet.md`
4. **Arquitectura:** `docs/SPEC_ES.md`

---

**Ultima actualizacion:** 2026-03-20
**Version:** 1.0.0
