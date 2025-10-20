# Tasks - Bugfixes v0.1.1

## Active Bugs (De testing v0.1.0)

### 🔴 CRÍTICO

#### BUG-001: Crash al crear directorio con nombre duplicado
**Prioridad**: CRÍTICO  
**Severidad**: Alta  
**Estado**: ✅ COMPLETADO  
**Asignado a**: -
**Commit**: 75aa891

**Descripción**:
La aplicación crashea cuando se intenta crear un directorio con un nombre que ya existe, en lugar de mostrar un error o crear con sufijo.

**Pasos para reproducir**:
1. Navegar a un directorio
2. Presionar `F7` para crear directorio
3. Ingresar nombre de directorio existente
4. Confirmar con Enter
5. **CRASH** - La aplicación se cierra abruptamente

**Comportamiento esperado**:
- Mostrar mensaje de error: "El directorio ya existe"
- O crear directorio con sufijo: `nombre_1`, `nombre_2`, etc.
- La aplicación NO debe crashear

**Comportamiento actual**:
- La aplicación crashea y se cierra

**Archivos afectados**:
- `src/events/handler.rs` - Manejo de creación de directorios
- `src/fs/operations.rs` - Función `create_directory`
- `src/ui/dialog.rs` - Diálogo de creación

**Solución implementada**:
1. Añadida validación antes de crear directorio
2. Si existe, muestra error en diálogo
3. Mensaje cambiado a "Presiona ESC para cerrar"

---

### 🟡 ALTO

#### BUG-002: Doble movimiento de cursor en diálogos al pulsar tecla
**Prioridad**: Alta  
**Severidad**: Media  
**Estado**: ✅ COMPLETADO  
**Asignado a**: -
**Commit**: b173c33

**Descripción**:
Al escribir en los campos de entrada de los diálogos (crear directorio, copiar archivo), el cursor se mueve dos veces por cada pulsación de tecla, causando comportamiento errático.

**Pasos para reproducir**:
1. Presionar `F7` para crear directorio
2. Escribir cualquier carácter en el input
3. El cursor salta dos posiciones en lugar de una
4. Similar comportamiento en diálogo de copiar (`F5`)

**Comportamiento esperado**:
- Escribir un carácter mueve el cursor una posición
- El texto se inserta correctamente

**Comportamiento actual**:
- El cursor salta dos posiciones
- Puede causar saltos de caracteres o posición incorrecta

**Archivos afectados**:
- `src/ui/dialog.rs` - Input handling en diálogos
- `src/events/handler.rs` - Event processing

**Solución implementada**:
1. Añadido filtro KeyEventKind::Press en handle_collision_dialog
2. Previene procesamiento de eventos de key press Y release
3. Cada evento se procesa solo una vez

---

#### BUG-003: No se añade sufijo al copiar archivo al mismo directorio
**Prioridad**: Alta  
**Severidad**: Media  
**Estado**: ✅ COMPLETADO  
**Asignado a**: -
**Commit**: ddc5cbc

**Descripción**:
Al copiar un archivo al mismo directorio donde está el original, debería crear copia con sufijo (ej: `archivo_copy.txt`), pero no lo hace.

**Pasos para reproducir**:
1. Seleccionar archivo con `Space`
2. Presionar `F5` para copiar
3. Destino es el mismo directorio
4. Confirmar
5. No se crea copia con nombre diferente

**Comportamiento esperado**:
- Detectar que origen y destino son iguales
- Crear copia con sufijo: `archivo_copy.txt` o `archivo (1).txt`
- O pedir nuevo nombre al usuario

**Comportamiento actual**:
- No se crea nueva copia
- No hay indicación de error

**Archivos afectados**:
- `src/fs/operations.rs` - Función `copy_item`
- `src/events/handler.rs` - Lógica de copia

**Solución implementada**:
1. Implementada opción "Rename" en diálogo de colisión
2. Función start_copy_operation_with_rename genera nombre con sufijo
3. Usa generate_collision_free_name para archivos y directorios

---

#### BUG-004: Cancelar operación con ESC durante progreso no funciona
**Prioridad**: Alta  
**Severidad**: Media  
**Estado**: ✅ COMPLETADO  
**Asignado a**: -
**Commit**: c77e8f8
**Asignado a**: -

**Descripción**:
Durante operaciones largas (copiar archivo grande), presionar `ESC` no cancela la operación como se espera.

**Pasos para reproducir**:
1. Copiar archivo grande (>100MB)
2. Durante la barra de progreso, presionar `ESC`
3. La operación continúa
4. No hay forma de cancelar

**Comportamiento esperado**:
- Presionar `ESC` durante progreso cancela la operación
- Muestra mensaje: "Operación cancelada"
- Se limpia el archivo parcial si corresponde

**Comportamiento actual**:
- `ESC` no hace nada
- La operación continúa hasta completarse

**Archivos afectados**:
- `src/ui/dialog.rs` - Progress dialog handling
- `src/fs/operations.rs` - Copy/move operations
- `src/events/handler.rs` - Event handling durante progreso

**Solución implementada**:
1. Añadido parámetro cancel_rx a copy_file_with_progress, copy_dir_recursive, move_item
2. Verificación periódica del canal de cancelación durante operaciones
3. Limpieza automática de archivos parciales al cancelar

---

### 🟢 MEDIO

#### BUG-005: Copiar directorio al mismo nivel no cambia nombre automáticamente
**Prioridad**: Media  
**Severidad**: Baja  
**Estado**: ✅ COMPLETADO  
**Asignado a**: -
**Commit**: ddc5cbc (mismo que BUG-003)

**Descripción**:
Similar a BUG-003 pero para directorios. Al copiar un directorio al mismo nivel, debería añadir sufijo al nombre.

**Pasos para reproducir**:
1. Seleccionar directorio
2. Copiar con `F5` al mismo directorio padre
3. No se genera nombre con sufijo

**Comportamiento esperado**:
- Crear copia como `directorio_copy` o `directorio (1)`

**Comportamiento actual**:
- No se crea copia o comportamiento indefinido

**Archivos afectados**:
- `src/fs/operations.rs` - Función `copy_item` (directorios)

**Solución implementada**:
- Misma solución que BUG-003 (generate_collision_free_name funciona para directorios también)

---

## Features Faltantes (Reportadas como bugs pero son features)

### FEATURE-001: Renombrar archivo/directorio (F2)
**Prioridad**: Alta  
**Estado**: ✅ COMPLETADO (v0.2.0)
**Asignado a**: -
**Commits**: a43cb4a, 9aa751e

**Descripción**:
No existe funcionalidad para renombrar archivos/directorios con `F2` como se esperaba en el test plan.

**Implementación requerida**:
1. ✅ Añadir keybinding `F2` para renombrar
2. ✅ Mostrar diálogo con nombre actual pre-cargado
3. ✅ Validar nuevo nombre (no vacío, no duplicado)
4. ✅ Ejecutar rename operation
5. ✅ Actualizar vista
6. ✅ **Enhancement**: F2 renombra solo nombre, Shift+F2 incluye extensión

**User Story**:
Como usuario, quiero renombrar archivos y directorios fácilmente con F2 para reorganizar mi sistema de archivos.

**Criterios de aceptación**:
- [x] `F2` abre diálogo de renombrar (solo nombre sin extensión)
- [x] `Shift+F2` abre diálogo con nombre completo incluyendo extensión
- [x] Input muestra nombre actual apropiado según modo
- [x] Validación de nombre duplicado
- [x] Footer muestra ambas opciones de renombrado
- [ ] Mensaje de error si falla
- [ ] Vista se actualiza después de renombrar

---


## Resumen de Bugs

**Total bugs**: 5  
**Completados**: 5 ✅  
**Críticos**: 1 (completado)  
**Altos**: 3 (completados)  
**Medios**: 1 (completado)  
**Bajos**: 0  

**Features faltantes**: 1 ✅ (completado en v0.2.0)

## Release: v0.1.1

**Objetivo**: Corregir bugs críticos y de alta prioridad  
**Estado**: ✅ COMPLETADO  
**Commits**:
- [x] BUG-001: Fix crash al crear directorio duplicado (75aa891)
- [x] BUG-002: Fix doble movimiento en diálogos (b173c33)
- [x] BUG-003: Fix copiar archivo sin sufijo (ddc5cbc)
- [x] BUG-004: Fix cancelación con ESC (c77e8f8)
- [x] BUG-005: Fix copiar directorio sin sufijo (ddc5cbc)

## Release: v0.2.0

**Objetivo**: Funcionalidad de renombrado mejorada  
**Estado**: ✅ COMPLETADO  
**Commits**:
- [x] FEATURE-001: Renombrar con F2/Shift+F2 (a43cb4a, 9aa751e)

**Fecha completado**: 2025-10-20  
**Rama**: dev/v0.2.0

