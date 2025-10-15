# Implementation Plan: Explorador de Archivos TUI con Doble Panel

**Branch**: `001-explorador-de-archivos` | **Date**: 2024-10-14 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-explorador-de-archivos/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Construir un explorador de archivos TUI (Terminal User Interface) en Rust con doble panel lateral para navegación simultánea de directorios, soportando operaciones fundamentales de archivos (copiar, mover, eliminar, crear carpetas), selección múltiple para operaciones en lote, previsualización de archivos de texto e imágenes en modales, y descompresión de archivos comprimidos (ZIP, TAR, 7Z, RAR) con soporte de contraseñas. Todo controlado completamente por teclado. La aplicación usará Ratatui para renderizado TUI, crossterm para eventos de teclado, y tokio para operaciones asíncronas de archivos sin bloquear la UI.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)  
**Primary Dependencies**: 
- `ratatui` 0.25+ - TUI framework para renderizado de widgets y layout
- `crossterm` 0.27+ - Terminal backend, eventos de teclado, control de cursor
- `tokio` 1.35+ - Runtime asíncrono para operaciones de I/O sin bloquear UI
- `walkdir` 2.5+ - Recursión de directorios para copiar/eliminar carpetas
- `humansize` 2.1+ - Formateo human-readable de tamaños de archivo
- `serde` + `serde_json` 1.0+ - Persistencia de estado (directorios actuales)
- `anyhow` 1.0+ - Manejo de errores ergonómico
- `glob` 0.3+ - Pattern matching para búsqueda/filtrado
- `image` 0.24+ - Decodificación de PNG/JPG/GIF/BMP para preview
- `artem` 0.3+ o `viuer` 0.7+ - Conversión de imagen a ASCII/Unicode art
- `zip` 0.6+ - Lectura de archivos ZIP (con soporte de contraseñas)
- `tar` 0.4+ - Lectura de archivos TAR
- `flate2` 1.0+ - Descompresión GZIP/BZ2
- `xz2` 0.1+ - Descompresión XZ/LZMA
- `sevenz-rust` 0.5+ - Lectura de archivos 7Z
- `unrar` 0.5+ - Lectura de archivos RAR (wrapper de libunrar)
- `encoding_rs` 0.8+ - Detección de encoding para archivos de texto no-UTF8

**Storage**: Filesystem local + archivo de configuración JSON (`~/.config/leeky-explorer/state.json`) para persistir último estado  
**Testing**: `cargo test` con unit tests + integration tests usando eventos mock de teclado  
**Target Platform**: Linux/macOS/Windows - cualquier terminal con soporte ANSI (mínimo 80 columnas, 256 colores)  
**Project Type**: Single binary CLI application (standalone executable)  
**Performance Goals**: 
- Respuesta a input de teclado <100ms (feedback inmediato)
- Navegación entre directorios <500ms 
- Actualización de barra de progreso cada 250ms en operaciones largas
- Soporte para directorios con 10,000+ archivos sin lag

**Constraints**: 
- Solo teclado (cero dependencia de mouse)
- UI debe ser responsive en terminales desde 80x24 hasta pantalla completa
- Operaciones de archivos grandes (GB) no deben bloquear navegación
- Recuperación graceful de errores de IO (permisos, disco lleno) sin crash

**Scale/Scope**: 
- Aplicación single-user local
- ~4,000-6,000 LOC estimadas (extendido de original por features de preview y descompresión)
- 8 User Stories implementables incrementalmente (P1-P8)
- 2 paneles + header + footer + diálogos modales + preview modales (7-10 componentes UI)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

✅ **Specification-First**: spec.md actualizado con 8 User Stories priorizadas (P1-P8) antes de implementación  
✅ **Incremental User Stories**: P1 (navegación) es MVP independiente, P2-P8 se pueden implementar incrementalmente  
✅ **Template-Driven**: Usando spec-template.md y plan-template.md del framework SpecKit  
✅ **Compliance Gates**: Este plan documenta estructura técnica antes de comenzar Phase 0  
✅ **Test Clauses**: Cada User Story tiene acceptance scenarios verificables  
✅ **Simplicity First**: Arquitectura directa con módulos claramente separados, sin abstracciones innecesarias  
✅ **No More Than 3**: Proyecto único (explorador TUI), no múltiples servicios  

**Status**: ✅ PASS - Cumple todos los principios del constitution.md v1.0.0

**Note**: Se han añadido 4 User Stories adicionales (P5-P8) después del diseño inicial del MVP. Scope expandido incluye: selección múltiple, preview de texto/imágenes, y descompresión de archivos. Total estimado: ~4,000-6,000 LOC.

## Project Structure

### Documentation (this feature)

```
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```
src/
├── main.rs              # Entry point: setup terminal, event loop, cleanup
├── app.rs               # AppState: manage dual panels, active panel, operations
├── models/
│   ├── mod.rs
│   ├── panel.rs         # Panel struct: path, entries, cursor, scroll state
│   ├── file_entry.rs    # FileEntry: name, type, size, permissions, mtime
│   ├── operation.rs     # Operation enum: Copy/Move/Delete with progress
│   └── selection.rs     # SelectionState: marked items per panel
├── ui/
│   ├── mod.rs
│   ├── layout.rs        # Build Ratatui layout: 2 columns + header + footer
│   ├── panel_widget.rs  # Render file list with selection highlight
│   ├── dialog.rs        # Modal dialogs: confirm, input, progress bar
│   ├── preview_modal.rs # Preview modals: text viewer, image viewer
│   └── theme.rs         # Colors and styles (directories, files, symlinks)
├── fs/
│   ├── mod.rs
│   ├── navigator.rs     # Navigate dirs: read_dir, enter, go_up, filter
│   ├── operations.rs    # Async copy/move/delete with progress callbacks
│   └── metadata.rs      # Extract metadata: size, permissions, format dates
├── preview/
│   ├── mod.rs
│   ├── text_viewer.rs   # Load and render text files with scroll
│   ├── image_viewer.rs  # Convert images to ASCII/Unicode art
│   └── encoding.rs      # Detect and handle non-UTF8 encodings
├── archive/
│   ├── mod.rs
│   ├── extractor.rs     # Extract ZIP/TAR/7Z/RAR with progress
│   ├── formats.rs       # Format detection by magic bytes
│   └── password.rs      # Password prompt and validation
├── events/
│   ├── mod.rs
│   ├── handler.rs       # Map keyboard events to app actions
│   └── keybindings.rs   # Key constants: F5=Copy, F6=Move, F4=Preview, F9=Extract
└── config/
    ├── mod.rs
    ├── state.rs         # Load/save state from JSON
    └── paths.rs         # Config file paths (~/.config/leeky-explorer/)

tests/
├── unit/
│   ├── panel_tests.rs
│   ├── operations_tests.rs
│   ├── navigation_tests.rs
│   ├── selection_tests.rs
│   ├── archive_tests.rs
│   └── preview_tests.rs
└── integration/
    ├── app_tests.rs     # Simulate full keyboard workflows
    └── fixtures/        # Test directories with mock files + test archives

Cargo.toml               # Dependencies and project metadata
README.md                # Quickstart: install, run, key bindings
```

**Structure Decision**: Single Rust binary project (Option 1). La estructura modular separa concerns claramente: `models/` para data structures (incluyendo SelectionState), `ui/` para renderizado Ratatui (incluyendo preview modals), `fs/` para operaciones de archivos async, `preview/` para visualización de texto/imágenes, `archive/` para extracción de comprimidos, `events/` para input handling. Esta organización permite testear cada módulo independientemente y mantiene `main.rs` y `app.rs` como orquestadores ligeros.

## Complexity Tracking

*No violations to track - all constitution gates passed. This project follows simplicity-first principles with a single binary, clear module boundaries, and incremental delivery model.*

---

## Architecture & Implementation Details

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         Terminal                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Ratatui Frame                                        │  │
│  │  ┌──────────────────┬──────────────────┐             │  │
│  │  │  Panel Left      │  Panel Right     │  Header     │  │
│  │  │  (FileEntry[])   │  (FileEntry[])   │  (PWDs)     │  │
│  │  │  - cursor: usize │  - cursor: usize │             │  │
│  │  │  - scroll: usize │  - scroll: usize │             │  │
│  │  └──────────────────┴──────────────────┘             │  │
│  │  ┌─────────────────────────────────────┐             │  │
│  │  │  Footer (Key Bindings)              │             │  │
│  │  └─────────────────────────────────────┘             │  │
│  │  ┌─────────────────────────────────────┐             │  │
│  │  │  Dialog (Modal - optional)          │             │  │
│  │  │  - Confirm / Input / Progress       │             │  │
│  │  └─────────────────────────────────────┘             │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
           ▲                                  │
           │ Render                           │ Events
           │                                  ▼
┌──────────────────────────────────────────────────────────────┐
│                      AppState                                 │
│  - left_panel: Panel                                          │
│  - right_panel: Panel                                         │
│  - active_panel: PanelSide                                    │
│  - current_operation: Option<Operation>                       │
│  - dialog_state: Option<DialogState>                          │
│  - filter: Option<String>                                     │
│  - selection_state: SelectionState (marked items per panel)   │
│  - preview_state: Option<PreviewState> (text/image modal)     │
│  - archive_state: Option<ArchiveState> (extraction progress)  │
└──────────────────────────────────────────────────────────────┘
           │                                  ▲
           │ Async Commands                   │ Results
           ▼                                  │
┌──────────────────────────────────────────────────────────────┐
│                   File Operations (tokio)                     │
│  - fs::copy_with_progress(src, dst, tx)                      │
│  - fs::move_item(src, dst)                                    │
│  - fs::delete_recursive(path, tx)                             │
│  - fs::create_dir(path)                                       │
│  - fs::list_dir(path) -> Vec<FileEntry>                      │
│  - preview::load_text_file(path) -> Result<String>           │
│  - preview::image_to_ascii(path) -> Result<String>           │
│  - archive::extract_with_progress(path, dest, password, tx)  │
│  - archive::list_contents(path) -> Vec<ArchiveEntry>         │
└──────────────────────────────────────────────────────────────┘
```

### Event Loop Flow

```rust
// Simplified event loop in main.rs
loop {
    terminal.draw(|f| ui::render(f, &app_state))?;
    
    if crossterm::event::poll(Duration::from_millis(100))? {
        match crossterm::event::read()? {
            Event::Key(key) => {
                match app_state.handle_key(key) {
                    Action::Quit => break,
                    Action::Navigate(direction) => { /* update cursor */ },
                    Action::EnterDir => { /* load new dir */ },
                    Action::StartCopy => { 
                        // Spawn async operation
                        let (tx, rx) = channel();
                        tokio::spawn(async move {
                            fs::copy_with_progress(src, dst, tx).await
                        });
                        app_state.current_operation = Some(rx);
                    },
                    // ... other actions
                }
            },
            Event::Resize(w, h) => { /* update layout */ },
            _ => {}
        }
    }
    
    // Check progress updates from async operations
    if let Some(rx) = &app_state.current_operation {
        if let Ok(progress) = rx.try_recv() {
            app_state.update_progress(progress);
        }
    }
}
```

### Key Data Structures

```rust
// models/panel.rs
pub struct Panel {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub cursor: usize,
    pub scroll_offset: usize,
    pub filter: Option<String>,
}

// models/file_entry.rs
pub struct FileEntry {
    pub name: String,
    pub entry_type: EntryType, // File, Dir, Symlink
    pub size: u64,
    pub modified: SystemTime,
    pub permissions: Permissions,
}

// models/operation.rs
pub enum Operation {
    Copy { src: PathBuf, dst: PathBuf, progress: Progress },
    Move { src: PathBuf, dst: PathBuf },
    Delete { path: PathBuf, progress: Progress },
}

pub struct Progress {
    pub bytes_done: u64,
    pub bytes_total: u64,
    pub files_done: usize,
    pub files_total: usize,
}
```

### Ratatui Layout Strategy

```rust
// ui/layout.rs
use ratatui::layout::{Constraint, Direction, Layout};

pub fn create_layout(area: Rect) -> (Rect, Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Header (PWDs)
            Constraint::Min(10),        // Panels
            Constraint::Length(2),      // Footer (keys)
        ])
        .split(area);
    
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Left panel
            Constraint::Percentage(50), // Right panel
        ])
        .split(chunks[1]);
    
    (chunks[0], panels[0], panels[1], chunks[2])
}
```

### Async File Operations with Progress

```rust
// fs/operations.rs
use tokio::sync::mpsc;

pub async fn copy_with_progress(
    src: &Path,
    dst: &Path,
    tx: mpsc::Sender<Progress>
) -> Result<()> {
    let total_size = get_total_size(src).await?;
    let mut bytes_copied = 0u64;
    
    if src.is_file() {
        let mut reader = tokio::fs::File::open(src).await?;
        let mut writer = tokio::fs::File::create(dst).await?;
        
        let mut buffer = vec![0u8; 8192];
        loop {
            let n = reader.read(&mut buffer).await?;
            if n == 0 { break; }
            writer.write_all(&buffer[..n]).await?;
            
            bytes_copied += n as u64;
            tx.send(Progress { 
                bytes_done: bytes_copied, 
                bytes_total: total_size,
                files_done: 0,
                files_total: 1,
            }).await?;
        }
    } else {
        // Recursive copy for directories using walkdir
        copy_dir_recursive(src, dst, tx, total_size).await?;
    }
    
    Ok(())
}
```

### State Persistence

```rust
// config/state.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PersistedState {
    pub left_path: PathBuf,
    pub right_path: PathBuf,
    pub active_panel: PanelSide,
}

impl PersistedState {
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        fs::create_dir_all(path.parent().unwrap())?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }
    
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("No config dir"))?;
        Ok(config_dir.join("leeky-explorer").join("state.json"))
    }
}
```

### Error Handling Strategy

```rust
// All public functions return Result<T, anyhow::Error>
// UI displays errors in modal dialogs without crashing

pub fn handle_key(&mut self, key: KeyEvent) -> Result<Action> {
    match key.code {
        KeyCode::F5 => {
            self.start_copy().context("Failed to start copy")?;
        },
        KeyCode::F8 => {
            self.start_delete().context("Failed to start delete")?;
        },
        // ... other keys
    }
    Ok(Action::Continue)
}

// In event loop:
if let Err(e) = app.handle_key(key) {
    app.show_error_dialog(format!("Error: {:?}", e));
}
```

### Testing Strategy

**Unit Tests** (`tests/unit/`):
- `panel_tests.rs`: Test navigation, cursor movement, filtering
- `operations_tests.rs`: Test copy/move/delete logic with temp dirs
- `file_entry_tests.rs`: Test metadata extraction, formatting

**Integration Tests** (`tests/integration/`):
- `app_tests.rs`: Simulate full keyboard workflows using mock events
- Create fixture directories with known structure
- Assert final state after sequence of key presses

```rust
#[tokio::test]
async fn test_copy_workflow() {
    let (fixture_dir, app) = setup_test_app().await;
    
    // Simulate: Arrow down, F5 (copy), Enter (confirm)
    app.handle_key(KeyCode::Down.into()).unwrap();
    app.handle_key(KeyCode::F(5).into()).unwrap();
    app.handle_key(KeyCode::Enter.into()).unwrap();
    
    // Wait for operation
    wait_for_operation(&app).await;
    
    // Assert file was copied
    assert!(fixture_dir.join("panel_right/copied_file.txt").exists());
}
```

---

## Phase 0: Research & Validation *(to be filled)*

*This section will be populated during Phase 0 research with:*
- Ratatui best practices for dual-pane layouts
- crossterm event handling patterns
- tokio async file operations performance benchmarks
- State persistence approaches (JSON vs TOML vs RON)
- Terminal compatibility testing results

## Phase 1: Design & Contracts *(to be filled)*

*This section will be populated during Phase 1 with:*
- `data-model.md`: Complete schemas for Panel, FileEntry, Operation
- `contracts/`: API contracts for fs operations modules
- `quickstart.md`: Build instructions, installation, usage guide
- UI mockups for each dialog type (confirm, input, progress)

## Phase 2: Task Breakdown

*Generated by `/speckit.tasks` command - see `tasks.md` when created*
