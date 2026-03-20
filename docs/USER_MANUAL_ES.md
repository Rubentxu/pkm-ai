# PKM-AI — Manual de Usuario

> Gestion Personal de Conocimiento con Zettelkasten + Columna Estructural

## 1. Instalacion

### Requisitos Previos

- **Rust 1.75+** (verificar con `rustc --version`)
- **Cargo** (incluido con Rust)
- **Git** (para clonar el repositorio)
- **SurrealDB** (incrustado, no requiere instalacion separada)

---

### 1.1 macOS

#### Opcion A: Homebrew (Recomendado)

```bash
# Clonar repositorio
git clone https://github.com/rubentxu/pkmai
cd pkmai

# Instalar con cargo
cargo install --path .

# O compilar e instalar manualmente
cargo build --release --locked
sudo mv target/release/pkmai /usr/local/bin/
sudo chmod +x /usr/local/bin/pkmai
```

#### Opcion B: Binario Directo

```bash
# Crear directorio local bin
mkdir -p ~/bin

# Compilar el binario
git clone https://github.com/rubentxu/pkmai
cd pkmai
cargo build --release --locked

# Mover a ~/bin
mv target/release/pkmai ~/bin/

# Agregar a PATH (anadir a ~/.zshrc o ~/.bash_profile)
echo 'export PATH="$HOME/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

#### Verificar en macOS

```bash
pkmai --version
```

#### Solucion de Problemas macOS

```bash
# Si "pkmai: command not found" despues de instalacion
source ~/.zshrc

# Si macOS bloqueo por seguridad
# Preferencias del Sistema → Seguridad y Privacidad → Permitir pkmai
# O ejecutar: xattr -d com.apple.quarantine ~/bin/pkmai
```

---

### 1.2 Linux

#### Opcion A: Compilar desde Fuente

```bash
# Instalar Rust si no esta instalado
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Clonar y compilar
git clone https://github.com/rubentxu/pkmai
cd pkmai
cargo build --release --locked

# Instalar globalmente
sudo cp target/release/pkmai /usr/local/bin/
sudo chmod +x /usr/local/bin/pkmai
```

#### Opcion B: Instalacion a Nivel de Usuario

```bash
# Clonar repositorio
git clone https://github.com/rubentxu/pkmai
cd pkmai

# Compilar version de lanzamiento
cargo build --release --locked

# Instalar en ~/.local/bin
mkdir -p ~/.local/bin
mv target/release/pkmai ~/.local/bin/

# Agregar a PATH (anadir a ~/.bashrc o ~/.zshrc)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

#### Verificar en Linux

```bash
pkmai --version
```

#### Solucion de Problemas Linux

```bash
# Si hay error de permisos en /usr/local/bin
ls -la /usr/local/bin/pkmai

# Verificar dependencias de librerias
ldd $(which pkmai)

# Instalar librerias faltantes si es necesario
sudo apt install libssl-dev # Debian/Ubuntu
sudo dnf install openssl-devel # Fedora
```

---

### 1.3 Windows

#### Opcion A: Winget (Recomendado)

```powershell
# Instalar Rust via winget (si no esta instalado)
winget install Rust.Rust

# Clonar repositorio
git clone https://github.com/rubentxu/pkmai
cd pkmai

# Compilar
cargo build --release --locked

# Copiar a un directorio en PATH
mkdir $env:USERPROFILE\bin
copy target\release\pkmai.exe $env:USERPROFILE\bin\

# Agregar a PATH (PowerShell)
$env:Path += ";$env:USERPROFILE\bin"
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$env:USERPROFILE\bin", "User")
```

#### Opcion B: Instalacion Manual

```powershell
# 1. Instalar Rust desde https://rustup.rs

# 2. Clonar repositorio
git clone https://github.com/rubentxu/pkmai
cd pkmai

# 3. Compilar
cargo build --release --locked

# 4. Copiar binario a una ubicacion conveniente
#    ej., C:\Users\<TuUsuario>\bin\

# 5. Agregar a PATH:
#    Configuracion de Windows → Sistema → Informacion → Configuracion avanzada del sistema
#    → Variables de entorno → PATH → Editar → Agregar C:\Users\<TuUsuario>\bin
```

#### Verificar en Windows

```powershell
pkmai --version
```

#### Solucion de Problemas Windows

```powershell
# Si 'pkmai' no se reconoce
# Cerrar y reabrir la terminal

# Verificar PATH
echo $env:Path

# Verificar que el binario existe
Test-Path "$env:USERPROFILE\bin\pkmai.exe"
```

---

### 1.4 Multiplataforma: Cargo Install (Todos los SO)

```bash
# Instalacion directa desde repositorio git
cargo install --git https://github.com/rubentxu/pkmai --locked

# O desde codigo fuente local
cargo install --path . --locked
```

---

### Verificar la Instalacion

```bash
pkmai --version
pkmai --help
```

---

### Configuracion

| Parametro | Valor Predeterminado | Variable de Entorno |
|-----------|---------------------|---------------------|
| Ruta de la base de datos | `~/.pkmai/` | `PKMAI_DB_PATH` |
| Auto-staging | `true` | `--stage` / `--no-stage` |
| Nivel de registro | `info` (silencioso) | `PKMAI_LOG_LEVEL` |
| Modo verbose | `false` | `--verbose` |

### Inicializacion de la Base de Datos

```bash
pkmai db init
```

---

### Shell Completions

Para autocompletado en bash/zsh/fish:

```bash
# Bash
source docs/shell-completion/bash/pkmai

# Zsh
source docs/shell-completion/zsh/_pkmai

# Fish
source docs/shell-completion/fish/pkmai.fish
```

---

### Desinstalacion Rapida

```bash
# Eliminar binario
sudo rm /usr/local/bin/pkmai          # Linux (global)
rm ~/.local/bin/pkmai                  # Linux (usuario)
rm ~/bin/pkmai                         # macOS

# Windows: Eliminar de la carpeta de instalacion y PATH
```

---

## 2. Inicio Rapido

### Captura Rapida con `quick`

El comando `quick` es la forma mas rapida de capturar una idea:

```bash
# Captura rapida (fleeting por defecto)
pkmai quick "Mi idea sobre Rust ownership"

# Con tipo especifico
pkmai quick "Nota de libro" -t literature

# Con tags
pkmai quick "Concepto importante" -T "rust,memoria"

# Con auto-staging y commit automatico
# (comportamiento por defecto, usa --no-stage para desactivar)
```

El comando `quick` automaticamente:
1. Crea el bloque
2. Lo agrega al staging
3. Hace commit con mensaje "Quick capture: [primeros 50 caracteres]"

### Crear Tu Primer Bloque

```bash
# Crear una nota permanente
pkmai create -t permanent \
  --title "Fundamentos del Modelo de Actor" \
  --content "El modelo de actor trata a los actores como las unidades fundamentales de la computacion concurrente."

# Crear una nota transitoria (captura rapida)
pkmai create -t fleeting \
  --title "Notas de Reunion" \
  --content "Se discutio el cronograma del proyecto y los entregables."

# Crear una nota de literatura (de fuentes)
pkmai create -t literature \
  --title "Patrones de Concurrencia en Rust" \
  --content "Resumen del capitulo sobre concurrencia de 'Programming Rust'."
```

### Auto-Staging

Por defecto, todos los bloques y enlaces se auto-stagean:

```bash
# Comportamiento por defecto: auto-staging activado
pkmai create -t permanent -T "Nueva nota"

# Desactivar auto-staging para un comando
pkmai create -t permanent -T "Nueva nota" --no-stage

# Desactivar globalmente (requiere usar --stage para stappear)
pkmai create -t permanent -T "Nueva nota" --no-stage
pkmai link <src> <dst> --no-stage
```

### Listar y Buscar Bloques

```bash
# Listar todos los bloques
pkmai list

# Filtrar por tipo
pkmai list -t fleeting
pkmai list -t permanent

# Busqueda fuzzy (encuentra aunque haya errores tipicos)
pkmai search "rust own"           # Encuentra "Rust Ownership Model"
pkmai list --search "prog"        # Encuentra "Rust Programming"

# Limitar resultados
pkmai list -l 20

# Filtrar por tags
pkmai list --tags "rust,memoria"
```

### AI Pre-Flight

Al crear bloques con `--verbose`, se muestra informacion de IA:

```bash
# Con sugerencias de IA
pkmai create -t permanent --title "Rust Ownership" --verbose

# Salida:
# 🤖 AI Pre-Flight:
# ⚠️  Notas similares encontradas (posibles duplicados):
#    - "Rust Ownership Model" (0.94)
# 📍 Ubicacion sugerida: "Rust Programming" (afinidad: 0.72)
# 🏷️  Tags sugeridos: rust, memory, ownership
# 🔗 Enlaces sugeridos: 3 notas
```

### Modo Interactivo

Para una experiencia mas guiada:

```bash
# Modo interactivo con confirmacion de duplicados
pkmai create -t permanent --title "Rust Ownership" --interactive

# Si encuentra duplicado >0.95 similitud, pregunta:
# ⚠️ Nota similar encontrada: "Rust Ownership Model" (0.97)
# [y]es (usar existente) / [n]o (crear nueva) / [e]dit / [a]bort:
```

### Auto-Deteccion de Tipo

Los comandos detectan automaticamente el tipo de bloque:

```bash
# Auto-detecta tipo 'task' por keywords
pkmai create "TODO: implementar auth"

# Auto-detecta tipo 'reference' por keywords
pkmai create "Cita del libro Programming Rust..."

# Auto-detecta tipo 'structure' por keywords
pkmai create "Indice: Rust Programming"
```

---

## 3. Referencia de Comandos CLI

### Comandos de Captura Rapida

| Comando | Descripcion | Ejemplo |
|---------|-------------|---------|
| `quick` | Captura + stage + commit | `pkmai quick "Mi idea"` |
| `quick -t literature` | Con tipo especifico | `pkmai quick "Nota" -t literature` |
| `quick -T "tag1,tag2"` | Con tags | `pkmai quick "Idea" -T "rust,rust"` |

### Comandos de Bloques

| Comando | Descripcion | Ejemplo |
|---------|-------------|---------|
| `create` | Crear un nuevo bloque | `pkmai create -t permanent -T "Titulo" -c "Contenido"` |
| `promote` | Cambiar tipo de bloque | `pkmai promote <id> -t permanent` |
| `list` | Listar bloques | `pkmai list -t fleeting -l 20` |
| `search` | Busqueda fuzzy | `pkmai search "rust own"` |
| `show` | Mostrar detalles del bloque | `pkmai show 01ARZ3NDEKTSV4RRFFQ69G5FAV` |
| `grep` | Buscar contenido | `pkmai grep "patron"` |
| `lint` | Validar integridad estructural | `pkmai lint` |

### Flags Globales

| Flag | Descripcion |
|------|-------------|
| `--stage` | Forzar staging (default) |
| `--no-stage` | No hacer staging automatico |
| `--verbose` | Mostrar informacion de IA |
| `--interactive` / `-i` | Modo interactivo |
| `--db-path <path>` | Ruta de base de datos |

### Promocion de Bloques

Transiciones validas de Zettelkasten:

```bash
# Promocionar a permanent (default)
pkmai promote 01ABC... -t permanent

# Promocionar fleeting a literature
pkmai promote 01ABC... -t literature

# Promocionar ghost a permanent (completar hueco)
pkmai promote 01ABC... -t permanent --stage

# Transiciones validas:
# fleeting → literature, permanent, ghost
# literature → permanent, ghost
# ghost → permanent
# structure/hub/task/reference/outline → permanent
```

### Comandos de Enlaces

| Comando | Descripcion | Ejemplo |
|---------|-------------|---------|
| `link` | Crear enlace entre bloques | `pkmai link <block-id> <structure-id> --type section_of --weight 1.0` |

### Comandos de Version

| Comando | Descripcion | Ejemplo |
|---------|-------------|---------|
| `version status` | Ver estado actual | `pkmai status` |
| `version commit` | Confirmar cambios | `pkmai version commit -m "mensaje"` |
| `version log` | Ver historial | `pkmai version log` |
| `version log --oneline` | Historial compacto | `pkmai log --oneline` |
| `version branch` | Gestionar ramas | `pkmai version branch` |

### Comandos de Base de Datos

| Comando | Descripcion | Ejemplo |
|---------|-------------|---------|
| `db init` | Inicializar base de datos | `pkmai db init` |
| `db stats` | Estadisticas | `pkmai db stats` |
| `db export` | Exportar a JSON | `pkmai db export > backup.json` |
| `db import` | Importar de JSON | `pkmai db import backup.json` |

### Comandos de Estructura

| Comando | Descripcion | Ejemplo |
|---------|-------------|---------|
| `traverse` | Recorrer la columna estructural | `pkmai traverse -d 3` |
| `toc` | Generar Tabla de Contenidos | `pkmai toc <structure-id>` |
| `synthesize` | Sintetizar documento | `pkmai synthesize <structure-id> --template technical-whitepaper --output pdf` |
| `gravity-check` | Verificar agrupacion semantica | `pkmai gravity-check` |

### Comandos de Nodos Fantasma

| Comando | Descripcion | Ejemplo |
|---------|-------------|---------|
| `ghost` | Gestionar nodos fantasma | `pkmai ghost list` |
| `ghost fill` | Completar un nodo fantasma | `pkmai ghost fill <ghost-id> --content "Nuevo contenido"` |
| `ghost dismiss` | Descartar un nodo fantasma | `pkmai ghost dismiss <ghost-id>` |

### Comandos Interactivos

| Comando | Descripcion |
|---------|-------------|
| `architect` | Lanzar TUI interactivo para exploracion del grafo de conocimiento |

---

## 4. Tipos de Bloques

| Tipo | Proposito | Ejemplo | Keywords de Auto-Deteccion |
|------|-----------|---------|---------------------------|
| `fleeting` | Captura rapida, notas temporales | Notas de reunion, tareas pendientes | - |
| `literature` | Notas de fuentes externas | Resumenes de libros, notas de articulos | idea, note, observation |
| `permanent` | Notas de conocimiento atomico | Conclusiones clave, conceptos | - |
| `structure` | Raiz del documento, organizacion | MOC (Mapa de Contenido) | index, moc, overview, summary |
| `hub` | Vision general del tema, indice | Indice de materias | - |
| `task` | Elementos de accion | Tareas, entregables | TODO, fix, implement, complete |
| `reference` | Enlaces externos, citas | URLs, citas | quote, book, chapter, author |
| `outline` | Estructura jerarquica | Esquema del documento | - |
| `ghost` | Brechas detectadas por IA | Explicaciones faltantes | - |

### Flujo Zettelkasten

```
Fleeting → Literature → Permanent
              ↓
           Ghost (detectado por IA)
              ↓
           Permanent (completado)
```

### Jerarquia de Tipos de Bloques

```
structure
├── hub
│   ├── permanent
│   │   ├── literature
│   │   └── reference
│   └── outline
│       └── permanent
├── task
└── ghost
```

---

## 5. Tipos de Enlaces

| Tipo | Significado | Caso de Uso |
|------|-------------|-------------|
| `section_of` | El bloque es una seccion de la estructura | Capitulo en documento |
| `supports` | Evidencia de apoyo | Citas, ejemplos |
| `extends` | Extension de otro bloque | Elaboraciones |
| `refines` | Version mas especifica | Explicacion detallada |
| `contradicts` | Vista opuesta | Debates, alternativas |
| `references` | Citacion externa | Enlaces a fuentes |
| `next` | Relacion secuencial | Siguiente bloque en la columna |
| `gravity` | Atraccion semantica | Relacionado pero no secuencial |

### Sintaxis de Enlaces

```bash
# Enlace basico (auto-staged por defecto)
pkmai link <source-id> <target-id> --type supports

# Con peso (para ordenamiento de la columna estructural)
pkmai link <block-id> <structure-id> --type section_of --weight 1.5

# Sin auto-staging
pkmai link <source-id> <target-id> --type supports --no-stage
```

---

## 6. Columna Estructural

La **Columna Estructural** es la columna vertebral del conocimiento ordenado, basada en el principio Folgezettel de Zettelkasten.

### Sistema de Peso de Secuencia

Los pesos utilizan ordenamiento basado en flotantes para inserciones flexibles:

```
1.0  → Bloque A (inicio)
1.5  → Bloque A1 (entre 1.0 y 2.0)
2.0  → Bloque B
2.1  → Bloque B1 (sub-seccion de B)
2.2  → Bloque B2
3.0  → Bloque C
```

### Recorrer la Columna

```bash
# Recorrido basico
pkmai traverse

# Con limite de profundidad
pkmai traverse -d 3

# Desde un bloque especifico
pkmai traverse --from 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

### Ganchos de Gravedad

La agrupacion semantica conecta bloques relacionados a traves de la columna:

```bash
# Verificar agrupacion semantica
pkmai gravity-check

# La salida muestra:
# - Bloques con alta atraccion gravitacional
# - Bloques huerfanos (sin conexiones)
# - Enlaces de gravedad sugeridos
```

---

## 7. Sintesis de Documentos

Transforma tus fragmentos de conocimiento en documentos completos.

### Generar Tabla de Contenidos

```bash
# Generar TOC para una estructura
pkmai toc 01ARZ3NDEKTSV4RRFFQ69G5FAV

# Con limite de profundidad
pkmai toc 01ARZ3NDEKTSV4RRFFQ69G5FAV --depth 3
```

### Sintetizar Documento

```bash
# Sintetizar a Markdown
pkmai synthesize 01ARZ3NDEKTSV4RRFFQ69G5FAV --output markdown

# Sintetizar a PDF (cuando la caracteristica typst esta habilitada)
pkmai synthesize 01ARZ3NDEKTSV4RRFFQ69G5FAV --template technical-whitepaper --output pdf

# Con plantilla personalizada
pkmai synthesize <structure-id> --template mi_plantilla --output pdf
```

### Pipeline de Sintesis

```
50 Zettels → Nota de Estructura → TOC → Documento
```

---

## 8. Nodos Fantasma

Los nodos fantasma son brechas en tu base de conocimiento detectadas por IA.

### Listar Nodos Fantasma

```bash
pkmai ghost list
```

### Propiedades de los Nodos Fantasma

| Propiedad | Descripcion |
|-----------|-------------|
| `confidence` | Puntuacion de confianza de IA (0-1) |
| `expected_keywords` | Palabras clave que la IA predice que deberian estar |
| `parent_block` | Bloque que detecto la brecha |

### Flujo de Trabajo para Completar o Descartar

```bash
# Completar un nodo fantasma con contenido
pkmai ghost fill <ghost-id> --content "Nuevo contenido explicativo"

# O usar promote (completar el hueco)
pkmai promote <ghost-id> -t permanent --content "Contenido"

# Descartar si la brecha no es relevante
pkmai ghost dismiss <ghost-id>
```

---

## 9. Configuracion

### Variables de Entorno

```bash
# Ubicacion de la base de datos
export PKMAI_DB_PATH=~/.pkmai/

# Nivel de registro (trace, debug, info, warn, error)
export PKMAI_LOG_LEVEL=info

# Habilitar caracteristicas de IA (requiere la caracteristica ai-integration)
export PKMAI_AI_ENABLED=true
```

### Flags Globales

```bash
# Modo verbose (muestra logs de debug)
pkmai --verbose list

# Ruta de base de datos
pkmai --db-path /tmp/pkmai.db list

# Auto-staging (default: true)
pkmai create -t permanent -T "Nota"     # Auto-staged
pkmai create -t permanent -T "Nota" --no-stage  # No staged
```

### Feature Flags

Compilar con caracteristicas especificas:

```bash
# Predeterminado (almacenamiento RocksDB)
cargo build --release

# Almacenamiento en memoria (pruebas)
cargo build --release --features memory

# Con integracion de IA
cargo build --release --features ai-integration

# Con renderizado PDF mediante Typst
cargo build --release --features typst
```

---

## 10. TUI Interactivo

Lanzar la interfaz de terminal interactiva para la exploracion del grafo de conocimiento:

```bash
pkmai architect
```

### Navegacion Vim-Style

| Tecla | Accion |
|-------|--------|
| `j` / `↓` | Mover abajo |
| `k` / `↑` | Mover arriba |
| `h` / `←` | Colapsar |
| `l` / `→` | Expandir |
| `g` | Ir al inicio |
| `G` | Ir al final |
| `PageUp` / `PageDown` | Paginar |
| `Enter` | Ver detalle |
| `Esc` | Volver |

### Enlaces y Navegacion

| Tecla | Accion |
|-------|--------|
| `b` | Navegar al primer backlink (en modo detalle) |
| `o` | Navegar al primer outgoing link (en modo detalle) |

### Comandos

| Comando | Accion |
|---------|--------|
| `:` | Entrar en modo comando |
| `:search <query>` | Busqueda fuzzy |
| `:filter <type>` | Filtrar por tipo |
| `:filter` | Limpiar filtro |
| `:new` | Nueva nota |
| `:all` | Mostrar todos |
| `:quit` | Salir |
| `q` | Salir |

### Atajos

| Tecla | Accion |
|-------|--------|
| `?` | Mostrar ayuda |
| `Tab` | Ciclar filtros |
| `r` | Recargar datos |

### Caracteristicas del TUI

- Navegacion visual de bloques con emojis por tipo
- Ver backlinks y outgoing links en modo detalle
- Comandos rapidos para buscar y filtrar
- Panel de ayuda integrado
- Colores para mejor legibilidad

---

## 11. Solucion de Problemas

### Problemas de Base de Datos

```bash
# Reinicializar base de datos (ADVERTENCIA: elimina datos existentes)
rm -rf ~/.pkmai/
pkmai db init
```

### Rendimiento

```bash
# Ejecutar con registro para identificar comandos lentos
pkmai --verbose list
```

### Validacion

```bash
# Verificar integridad estructural
pkmai lint

# Auto-reparar problemas menores
pkmai lint --fix
```

### Debug de Auto-Staging

```bash
# Ver que esta staged
pkmai status

# Hacer staging manual si fue desactivado
# (debes usar los comandos individuales con --stage)
```

---

## 12. Tarjeta de Referencia Rapida

```bash
# ===== CAPTURA RAPIDA =====
pkmai quick "Mi idea"              # Fleeting + stage + commit
pkmai quick "Nota" -t literature   # Tipo especifico

# ===== CREAR =====
pkmai create -t permanent -T "Titulo" -c "Contenido"
pkmai create -t permanent -T "Titulo" --verbose  # Con AI pre-flight
pkmai create -t permanent -T "Titulo" --interactive  # Modo interactivo

# Auto-deteccion de tipo
pkmai create "TODO: hacer algo"    # → Task
pkmai create "Cita de libro..."    # → Reference

# ===== AUTO-STAGING =====
pkmai create -t permanent -T "Nota"          # Staged (default)
pkmai create -t permanent -T "Nota" --no-stage  # No staged
pkmai link <src> <dst> --type supports       # Staged (default)
pkmai link <src> <dst> --type supports --no-stage

# ===== BUSCAR =====
pkmai list                          # Todos
pkmai list -t permanent             # Por tipo
pkmai list --tags "rust,memoria"   # Por tags
pkmai list --search "rust"         # Busqueda fuzzy
pkmai search "rust own"            # Busqueda fuzzy (dedicado)
pkmai grep "patron"               # Buscar en contenido

# ===== NAVEGAR =====
pkmai show <id>                    # Ver bloque
pkmai architect                    # TUI interactivo

# ===== ORGANIZAR =====
pkmai promote <id> -t permanent    # Cambiar tipo
pkmai link <src> <dst> --type supports
pkmai traverse                    # Recorrer columna
pkmai toc <structure-id>           # Generar TOC

# ===== VERSION =====
pkmai status                       # Estado actual
pkmai version commit -m "mensaje"  # Commit
pkmai log --oneline               # Historial compacto
pkmai version branch              # Ramas

# ===== AI =====
pkmai ghost list                  # Ver huecos
pkmai ghost fill <id> --content "..."  # Completar
pkmai ghost dismiss <id>          # Descartar
pkmai gravity-check               # Ver clustering

# ===== BASE DE DATOS =====
pkmai db stats                     # Estadisticas
pkmai db export > backup.json      # Exportar
pkmai db import backup.json        # Importar
```

---

## 13. Changelog de Features

### v0.x - Roadmap Completado

**Fase 1: Core Workflow**
- `pkmai quick` - Captura rapida con auto-stage y commit
- `pkmai promote` - Transicion de tipos Zettelkasten
- Flags `--stage`/`--no-stage` globales

**Fase 2: AI Pre-Flight**
- Deteccion de duplicados por similitud >0.9
- Sugerencia de ubicacion (afinidad semantica)
- Sugerencia de tags basados en notas similares
- Sugerencia de enlacesAutomaticos

**Fase 3: Busqueda y UI**
- Busqueda fuzzy por titulo
- Modo interactivo con opciones `[y/n/e/a]`
- Auto-deteccion de tipo de bloque

**Fase 4: UX Polish**
- `pkmai status` - Estado git-like
- `pkmai log --oneline` - Historial compacto
- Emojis en output de list
- Shell completions (bash/zsh/fish)
- TUI con navegacion vim y comandos `:`
