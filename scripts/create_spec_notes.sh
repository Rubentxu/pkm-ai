#!/bin/bash
# Script para crear todas las notas del SPEC_ES.md en pkmai
# Una vez creadas, se pueden sintetizar en un documento completo

set -e

echo "=== Creando notas del SPEC_ES.md ==="

# ============================================================================
# 1. Vision y Principios
# ============================================================================

echo "Creando 1. Vision y Principios..."

pkmai create --block-type permanent --title "1.1 Vision Central" --content "PKM-AI es un Sistema Operativo de Conocimiento para equipos que trabajan con enjambres de agentes de IA. Mientras que herramientas como Obsidian o Logseq son zologicos individuales de notas, PKM-AI esta construido para:
- Multiples agentes de IA operando concurrentemente en la misma base de conocimiento
- Sintesis automatica de fragmentos Zettelkasten en documentos tecnicos profesionales
- Ordenamiento determinista emergente de la estructura, no impuesto jerarquicamente
- Alto rendimiento (objetivo: 65,000 bloques con operaciones de grafo en <16ms)"

pkmai create --block-type permanent --title "1.2 Principios Fundamentales" --content "Principios Fundamentales de PKM-AI:
- Modelo de Bloques-Atomo: Cada pieza de conocimiento es un bloque direccionable con ULID
- Columna Vertebral Estructural Primero: Orden y estructura son ciudadanos de primera clase
- Separacion Semantica/Estructural: Aristas semanticas (enlaces) vs aristas estructurales (ordenamiento)
- Nodos Fantasma como Predicados: Los huecos son restricciones que describen contenido ideal
- Rendimiento como Requisito: O(Delta bloques) no O(N) para ganchos de gravedad"

pkmai create --block-type permanent --title "1.3 Relacion entre Proyectos" --content "PKM-AI y Nexus-WASM son proyectos hermanos bajo hodei-pkm:
- Nexus-WASM: Runtime WASM de alto rendimiento para 65,536 entidades @ 60 FPS
- PKM-AI: Sistema operativo de conocimiento para 65,000+ bloques con agentes concurrentes
Ambos comparten arquitectura de actores pero difieren en dominio y tecnologia."

# ============================================================================
# 2. Modelos de Dominio
# ============================================================================

echo "Creando 2. Modelos de Dominio..."

pkmai create --block-type permanent --title "2.1 Bloque" -c "Bloque es la unidad fundamental de informacion en PKM-AI. Cada bloque tiene:
- id: Ulid (ordenable cronologicamente)
- block_type: Tipo del bloque (fleeting, literature, permanent, structure, hub, task, reference, outline, ghost)
- content: Contenido Markdown
- properties: Metadatos flexibles
- embedding_bloom: Filtro de busqueda semantica
- created_at, updated_at: Timestamps

Tipos de Bloque:
- fleeting (f): Notas de captura temporal
- literature (l): Material de referencia de fuentes externas
- permanent (p): Notas atomicas Zettelkasten
- structure (s, moc): Contenedores estructurales
- hub (h): Puntos de entrada de tema central
- task (t): Elementos de accion
- reference (r): Referencias externas
- outline (o): Esquemas jerarquicos
- ghost (g): Marcador de posicion para contenido faltante"

pkmai create --block-type permanent --title "2.2 Arista" -c "Arista representa enlaces entre bloques. Tiene:
- id: Ulid
- link_type: Tipo de enlace
- from, to: Bloques origen y destino
- properties: Metadatos del enlace
- sequence_weight: FractionalIndex para posicionamiento
- updated_at: Timestamp

Tipos de Enlace:
Semanticos: extends, refines, contradicts, questions, supports, references, related, similar_to
Estructurales: section_of, subsection_of, ordered_child, next, next_sibling, first_child, contains, parent
IA: ai_suggested"

pkmai create --block-type permanent --title "2.3 FractionalIndex (CRITICO: NO f32)" -c "CRITICO: NO usar f32 para ordenamiento de secuencias. Usar indexacion fraccionaria lexicografica.

FractionalIndex es una estructura que usa cadenas lexicograficas en lugar de floats para evitar problemas de precision. Proporciona:
- first(): Crea el primer indice
- after(last): Crea un indice despues del ultimo
- between(before, after): Crea un indice entre dos existentes

Esto garantiza que nunca hay degradacion de precision sin importar cuantes inserciones se hagan."

pkmai create --block-type permanent --title "2.4 SmartSection" -c "SmartSection representa una seccion inteligente con analisis semantico:
- block: Bloque asociado
- intent: Proposito de la seccion
- boundary_constraints: Restricciones de limites
- semantic_centroid: Vector centroide semantico
- medoid_id: ID del medoide (bloque mas representativo)
- vacancy_status: Estado de ocupacion (Full, NearlyFull, Partial, Sparse, Empty)
- coherence_score: Puntuacion de coherencia

Vacancy indica el porcentaje de ocupacion de la seccion:
- Full: >90%
- NearlyFull: 70-90%
- Partial: 30-70%
- Sparse: 10-30%
- Empty: <10%"

pkmai create --block-type permanent --title "2.5 GhostNode" -c "GhostNode representa un marcador de posicion para contenido faltante:
- id: Ulid del fantasma
- expected_keywords: Palabras clave esperadas en el contenido
- confidence: Confianza de la prediccion (0.0-1.0)
- parent_id: Seccion padre
- suggested_position: Posicion sugerida (FractionalIndex)
- status: Estado (Pending, Filled, Dismissed)

Los nodos fantasma son predicados que describen contenido ideal, ayudando a mantener la integridad estructural."

# ============================================================================
# 3. Columna Vertebral Estructural
# ============================================================================

echo "Creando 3. Columna Vertebral Estructural..."

pkmai create --block-type permanent --title "3.1 Definicion" -c "La Columna Vertebral Estructural es el eje ordenado de un documento, implementado como bloques enlazados via aristas NEXT. Es el orden primario de un documento."

pkmai create --block-type permanent --title "3.2 Reglas de Recorrido" -c "Reglas para recorrer la columna vertebral:
1. Iniciar desde el bloque raiz
2. Seguir aristas NEXT en orden
3. Respetar limite de profundidad (por defecto: 100)
4. Detectar y manejar ciclos"

pkmai create --block-type permanent --title "3.3 Algoritmo de Recorrido" -c "El algoritmo traverse_spine recorre la columna vertebral:
- Recibe: db, root (ULID), max_depth
- Usa HashSet para trackear nodos visitados
- Funcion recursiva traverse_recursive con limite de profundidad
- Consulta SurrealDB: SELECT out.*, sequence_weight FROM edge WHERE in = node AND link_type = next ORDER BY sequence_weight ASC
- Maneja estructuras anidadas: si un bloque es Structure, sigue recurriendo"

# ============================================================================
# 4. Secciones Inteligentes
# ============================================================================

echo "Creando 4. Secciones Inteligentes..."

pkmai create --block-type permanent --title "4.1 Centroide Semantico" -c "El centroide semantico se calcula usando media ponderada por importancia (enlaces entrantes + 1). Para cada dimension del embedding, se suma el valor ponderado y se divide por el total de pesos."

pkmai create --block-type permanent --title "4.2 Medoide" -c "El medoide es el bloque mas cercano al centroide semantico. Es el bloque mas representativo de una seccion, calculado como aquel con menor distancia al centroide."

# ============================================================================
# 5. Nodos Fantasma
# ============================================================================

echo "Creando 5. Nodos Fantasma..."

pkmai create --block-type permanent --title "5.1 Algoritmo de Deteccion" -c "Algoritmo de deteccion de nodos fantasma:
1. Obtener todos los bloques en una Estructura
2. Calcular centroides semanticos para cada seccion
3. Para cada par consecutivo, calcular la distancia
4. Si la distancia > umbral, insertar GhostNode"

# ============================================================================
# 6. API Estilo Git
# ============================================================================

echo "Creando 6. API Estilo Git..."

pkmai create --block-type permanent --title "6.1 Commit" -c "Commit representa una instantanea del estado:
- id: CommitId
- structure_snapshot: Estructura completa
- parents: Commits padres (para merges)
- author: AgentId que creo el commit
- message: Mensaje descriptivo
- created_at: Timestamp
- blocks_added, blocks_removed, blocks_modified: Listas de cambios"

pkmai create --block-type permanent --title "6.2 View (Ref)" -c "View (Ref) representa una referencia a un punto en el grafo:
- Branch: Rama con nombre, target (ULID), is_head
- Tag: Etiqueta con nombre, target, mensaje"

pkmai create --block-type permanent --title "6.3 WorkingSet (Indice)" -c "WorkingSet representa el indice de preparacion:
- id: WorkingSetId
- author: AgentId
- staged_blocks: Bloques en staging (BTreeMap<Ulid, BlockDelta>)
- staged_edges: Aristas en staging
- removed_blocks, removed_edges: Elementos eliminados
- operations: Historial de operaciones
- created_at, updated_at: Timestamps"

# ============================================================================
# 7. Comandos CLI
# ============================================================================

echo "Creando 7. Comandos CLI..."

pkmai create --block-type permanent --title "7.1 Comandos Principales" -c "Comandos CLI principales (34 total, 100% implementados):
- nexus create: Crear nuevo bloque
- nexus list: Listar bloques con filtrado
- nexus show: Mostrar detalles del bloque
- nexus link: Crear enlaces entre bloques
- nexus grep: Buscar contenido de bloques
- nexus traverse: Recorrer columna vertebral
- nexus gravity-check: Verificar conectividad
- nexus toc: Generar tabla de contenidos
- nexus synthesize: Sintetizar documento
- nexus ghost: Gestionar nodos fantasma
- nexus architect: Lanzar TUI interactiva
- nexus lint: Validar integridad estructural
- nexus db: Gestion de base de datos
- nexus api: Iniciar servidor API REST"

pkmai create --block-type permanent --title "7.2 Comandos de Control de Versiones" -c "Comandos de control de versiones (equivalentes Git):
- version status (git status)
- version log (git log)
- version diff (git diff)
- version add (git add)
- version commit (git commit)
- version branch (git branch)
- version checkout (git checkout)
- version merge (git merge)
- version tag (git tag)
- version push (git push)
- version pull (git pull)"

# ============================================================================
# 8. Herramientas MCP
# ============================================================================

echo "Creando 8. Herramientas MCP..."

pkmai create --block-type permanent --title "8.1 Herramientas de Bloque (4)" -c "Herramientas MCP para bloques:
- create_block(block_type, title, content?, tags?): Crear bloque
- get_block(id): Obtener bloque por ULID
- search_blocks(query?, block_type?, tags?, limit?): Buscar bloques
- update_block(id, title?, content?, tags?): Actualizar bloque"

pkmai create --block-type permanent --title "8.2 Herramientas de Enlace (3)" -c "Herramientas MCP para enlaces:
- create_link(from_id, to_id, link_type, weight?, context?): Crear enlace
- get_links(block_id, direction?): Obtener enlaces de un bloque
- suggest_links(block_id, limit?): Sugerencias de IA para enlaces"

pkmai create --block-type permanent --title "8.3 Herramientas de Columna Vertebral (3)" -c "Herramientas MCP para columna vertebral:
- traverse_spine(root_id?, depth?, link_type?): Recorrer la columna
- gravity_check(block_id, threshold?): Verificar conectividad
- reorder_block(block_id, new_position, parent_id?): Reordenar bloque"

pkmai create --block-type permanent --title "8.4 Herramientas de Estructura (3)" -c "Herramientas MCP para estructuras:
- get_section_map(root_id): Obtener jerarquia de secciones
- detect_gaps(section_id): Detectar faltantes en una seccion
- list_ghosts(root_id?): Listar marcadores fantasma"

pkmai create --block-type permanent --title "8.5 Herramientas de Sintesis (2)" -c "Herramientas MCP para sintesis:
- synthesize(structure_id, format?, template?): Generar documento
- get_toc(structure_id): Obtener tabla de contenidos"

# ============================================================================
# 9. API REST
# ============================================================================

echo "Creando 9. API REST..."

pkmai create --block-type permanent --title "9.1 Endpoints" -c "API REST (Base URL: /api/v1, ~70% implementada):

HEALTH:
- GET /health: Verificacion de salud

BLOCKS:
- GET /blocks: Listar bloques
- GET /blocks/:id: Obtener bloque
- POST /blocks: Crear bloque
- PUT /blocks/:id: Actualizar bloque
- DELETE /blocks/:id: Eliminar bloque
- GET /blocks/:id/history: Historial del bloque

STRUCTURES:
- GET /structures: Listar estructuras
- GET /structures/:id: Obtener estructura
- POST /structures: Crear estructura
- PUT /structures/:id: Actualizar estructura
- DELETE /structures/:id: Eliminar estructura
- GET /structures/:id/spine: Obtener columna vertebral

COMMITS:
- GET /commits: Listar commits
- GET /commits/:id: Obtener commit
- POST /commits: Crear commit
- GET /commits/:id/diff: Ver diff

VIEWS:
- GET /views: Listar vistas
- GET /views/:name: Obtener vista
- POST /views: Crear vista
- PUT /views/:name: Actualizar vista
- DELETE /views/:name: Eliminar vista

WORKINGSET:
- GET /working-set: Obtener conjunto de trabajo
- POST /working-set/stage: Agregar a preparacion
- POST /working-set/unstage: Quitar de preparacion
- POST /working-set/commit: Crear commit
- DELETE /working-set: Descartar

SYNC:
- POST /sync/push: Empujar a remoto
- POST /sync/pull: Traer de remoto
- POST /sync/fetch: Obtener metadatos
- GET /sync/status: Estado de sincronizacion"

# ============================================================================
# 10. Arquitectura
# ============================================================================

echo "Creando 10. Arquitectura..."

pkmai create --block-type permanent --title "10.1 Alto Nivel" -c "Arquitectura de alto nivel:
- CLI / TUI / MCP / API: Interfaces de usuario y programaticas
- PKM-AI CORE LIBRARY: Logica de negocio (Block CRUD, Edge Manager, FractionalIndex, Spine Traversal, Lint Engine, Ghost System)
- SURREALDB DAEMON: Base de datos (Unix socket: /tmp/pkmai-surreal.sock, tablas: block, edge, commit, view, working_set)"

pkmai create --block-type permanent --title "10.2 Patron Modo Daemon" -c "SurrealDB se ejecuta como proceso servidor, no embebido, para soportar acceso concurrente multi-proceso. El cliente intenta conectar al socket Unix existente; si no existe, lanza un nuevo daemon."

# ============================================================================
# 11. Estado de Implementacion
# ============================================================================

echo "Creando 11. Estado de Implementacion..."

pkmai create --block-type permanent --title "11.1 Cobertura de Pruebas" -c "Estado de pruebas:
- Total: 229 pruebas
- Aprobadas: 229
- Fallidas: 0"

pkmai create --block-type permanent --title "11.2 Estado de Modulos" -c "Estado de modulos (todos verdes):
- FractionalIndex: 3 pruebas
- Block Model: 5 pruebas
- GhostNode: 5+ pruebas
- SmartSection + Bloom: 10 pruebas
- GravityHooks: 10 pruebas
- Commit: 8 pruebas
- View: 10 pruebas
- WorkingSet: 12 pruebas
- LinkSuggester: 3 pruebas
- Synthesis: 5 pruebas
- MCP: 16 pruebas
- Traverse: 4 pruebas
- CLI Create: 11 pruebas
- CLI Link: 18 pruebas
- Embeddings: 5 pruebas
- Delta: 6 pruebas"

pkmai create --block-type permanent --title "11.3 Problemas Conocidos" -c "Problemas conocidos:
- 11 warnings del compilador (menor): Limpieza antes de v1.0
- Validacion de API REST incompleta (medio): Agregar manejo de errores
- Centroide ponderado no implementado (medio): Usar media simple"

# ============================================================================
# Apendice A: Decisiones de Diseno Criticas
# ============================================================================

echo "Creando Apendice A..."

pkmai create --block-type permanent --title "A.1 FractionalIndex sobre f32" -c "Decision A.1: Usar f32 para ordenamiento de secuencias causa degradacion de precision. FractionalIndex con cadenas lexicograficas nunca se degrada, sin importar cuantes inserciones se hagan entre dos elementos existentes."

pkmai create --block-type permanent --title "A.2 Direccion de section_of" -c "Decision A.2: La direccion correcta de section_of es Zettel -> Estructura (contenido apunta al contenedor). En SQL: RELATE block:01HABC1->edge:section_of->block:01HSTRUCT"

pkmai create --block-type permanent --title "A.3 traverse_spine Debe Ser Async" -c "Decision A.3: traverse_spine debe ser async porque necesita consultar la base de datos de forma asincrona para obtener los bloques hijos en cada nivel de la jerarquia."

echo "=== Todas las notas del SPEC_ES.md creadas exitosamente ==="
