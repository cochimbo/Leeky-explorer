# Feature Specification: Explorador de Archivos TUI con Doble Panel

**Feature Branch**: `001-explorador-de-archivos`  
**Created**: 2024-01-10  
**Status**: Draft  
**Input**: User description: "Explorador de archivos TUI en Rust con doble panel para navegar, copiar, mover, eliminar archivos y crear carpetas usando teclado"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Navegación Dual Panel con Teclado (Priority: P1)

Como usuario del explorador, quiero navegar por el sistema de archivos usando dos paneles lado a lado controlados completamente con teclado, para poder comparar y trabajar con archivos en diferentes ubicaciones simultáneamente.

**Why this priority**: Este es el núcleo del explorador. Sin navegación básica dual-panel, no hay producto. Es el MVP mínimo viable que permite al usuario moverse por sus archivos.

**Independent Test**: Puede probarse ejecutando el explorador, usando flechas arriba/abajo para navegar en un panel, Tab para cambiar de panel, Enter para entrar en carpetas y Backspace para subir niveles. Funciona independientemente sin necesidad de copiar/mover archivos.

**Acceptance Scenarios**:

1. **Given** el explorador está abierto mostrando dos paneles, **When** presiono flecha arriba/abajo, **Then** el cursor se mueve entre archivos/carpetas en el panel activo
2. **Given** estoy en el panel izquierdo, **When** presiono Tab, **Then** el foco cambia al panel derecho (indicado visualmente)
3. **Given** el cursor está sobre una carpeta, **When** presiono Enter, **Then** el panel navega dentro de esa carpeta mostrando su contenido
4. **Given** estoy dentro de una carpeta, **When** presiono Backspace, **Then** el panel sube un nivel al directorio padre
5. **Given** estoy en el panel derecho, **When** presiono Tab, **Then** el foco vuelve al panel izquierdo
6. **Given** el explorador muestra archivos, **When** la lista es más larga que la pantalla, **Then** puedo hacer scroll con flechas y el scroll bar se muestra correctamente

---

### User Story 2 - Copiar y Mover Archivos entre Paneles (Priority: P2)

Como usuario, quiero copiar archivos/carpetas desde el panel origen al panel destino usando F5 (copiar) y F6 (mover), con barra de progreso para operaciones largas, para gestionar mis archivos de forma eficiente entre ubicaciones.

**Why this priority**: Después de poder navegar, copiar/mover es la operación más fundamental de un file manager. Permite manipular archivos entre los dos paneles.

**Independent Test**: Requiere la navegación (P1) pero puede probarse independientemente seleccionando un archivo, presionando F5/F6, y verificando que se copia/mueve al directorio del otro panel con feedback visual.

**Acceptance Scenarios**:

1. **Given** tengo un archivo seleccionado en panel izquierdo y panel derecho muestra carpeta destino, **When** presiono F5, **Then** aparece diálogo de confirmación "Copiar [archivo] a [destino]?"
2. **Given** confirmo la operación de copia, **When** la copia comienza, **Then** se muestra barra de progreso con porcentaje y MB copiados/totales
3. **Given** una operación de copia exitosa, **When** termina, **Then** el archivo aparece en el panel destino y se muestra mensaje "Copia completada"
4. **Given** tengo un archivo seleccionado, **When** presiono F6 y confirmo, **Then** el archivo se mueve (desaparece de origen, aparece en destino)
5. **Given** selecciono una carpeta, **When** presiono F5/F6, **Then** la carpeta completa se copia/mueve recursivamente con progreso por archivo
6. **Given** hay error durante copia (permisos, espacio), **When** ocurre el error, **Then** se muestra mensaje de error claro y se permite reintentar o cancelar

---

### User Story 3 - Eliminar Archivos y Crear Carpetas (Priority: P3)

Como usuario, quiero eliminar archivos/carpetas con F8 (con confirmación) y crear nuevas carpetas con F7, para mantener organizado mi sistema de archivos directamente desde el explorador.

**Why this priority**: Complementa las operaciones básicas de gestión. Menos crítico que copiar/mover pero necesario para un file manager completo.

**Independent Test**: Puede probarse independientemente presionando F7 para crear carpeta (aparece diálogo de nombre), F8 para eliminar (requiere confirmación), verificando que las operaciones se reflejan en ambos paneles.

**Acceptance Scenarios**:

1. **Given** estoy en cualquier panel, **When** presiono F7, **Then** aparece diálogo "Nombre de nueva carpeta:" con campo de texto
2. **Given** ingreso nombre válido y confirmo, **When** presiono Enter, **Then** la carpeta se crea en el directorio actual del panel activo
3. **Given** tengo un archivo seleccionado, **When** presiono F8, **Then** aparece diálogo "¿Eliminar [archivo]? (S/N)"
4. **Given** confirmo eliminación con 'S', **When** se ejecuta, **Then** el archivo desaparece de la lista y se muestra "Eliminado: [archivo]"
5. **Given** tengo una carpeta seleccionada, **When** presiono F8 y confirmo, **Then** aparece segundo diálogo "¿Eliminar carpeta y todo su contenido? (S/N)"
6. **Given** intento eliminar carpeta no vacía, **When** confirmo doble confirmación, **Then** se elimina recursivamente con contador de archivos eliminados
7. **Given** hay error de permisos, **When** intento crear/eliminar, **Then** se muestra error claro sin crashear la aplicación

---

### User Story 4 - Búsqueda y Filtrado Rápido (Priority: P4)

Como usuario, quiero buscar/filtrar archivos en el panel actual presionando '/' y escribiendo un patrón (glob o texto simple), para encontrar archivos rápidamente en directorios grandes sin navegar manualmente.

**Why this priority**: Feature avanzado que mejora usabilidad pero no es esencial para operación básica. Puede añadirse después del MVP.

**Independent Test**: Puede probarse independientemente presionando '/', escribiendo patrón (ej. "*.txt"), y verificando que solo se muestran archivos coincidentes. Presionar Esc limpia el filtro.

**Acceptance Scenarios**:

1. **Given** estoy en un panel con muchos archivos, **When** presiono 'F3', **Then** aparece campo de búsqueda en la parte inferior: "Buscar: _"
2. **Given** el campo de búsqueda está activo, **When** escribo texto (ej. "doc"), **Then** la lista se filtra en tiempo real mostrando solo archivos que contienen "doc"
3. **Given** tengo filtro activo, **When** presiono Esc, **Then** el filtro se limpia y se muestran todos los archivos nuevamente
4. **Given** escribo patrón glob (ej. "*.rs"), **When** presiono Enter, **Then** se muestran solo archivos Rust
5. **Given** el filtro no coincide con nada, **When** escribo patrón sin resultados, **Then** se muestra "Sin resultados para: [patrón]"
6. **Given** tengo filtro activo, **When** navego a otra carpeta, **Then** el filtro se mantiene aplicado en la nueva ubicación

---

### User Story 5 - Selección Múltiple para Operaciones en Lote (Priority: P5)

Como usuario, quiero seleccionar múltiples archivos/carpetas en un panel usando la barra espaciadora para marcarlos uno a uno, o Ctrl+A para seleccionar todos, para poder copiar/mover/eliminar varios elementos simultáneamente en una sola operación.

**Why this priority**: Feature de productividad avanzado que complementa las operaciones básicas. No es crítico para el MVP pero mejora significativamente la experiencia para operaciones en lote.

**Independent Test**: Puede probarse independientemente navegando por archivos, presionando Espacio para marcar/desmarcar items (se muestran con indicador visual como asterisco o color diferente), luego presionando F5/F6/F8 y verificando que la operación se aplica a todos los seleccionados con feedback de progreso global.

**Acceptance Scenarios**:

1. **Given** estoy navegando en un panel, **When** presiono Espacio sobre un archivo/carpeta, **Then** el item se marca visualmente (ej. con "*" o color resaltado) y el cursor avanza al siguiente item
2. **Given** tengo un item ya marcado, **When** presiono Espacio sobre él nuevamente, **Then** se desmarca y vuelve a su apariencia normal
3. **Given** estoy en un panel con varios archivos, **When** presiono Ctrl+A, **Then** todos los items visibles se marcan simultáneamente
4. **Given** tengo todos los items marcados con Ctrl+A, **When** presiono Ctrl+A nuevamente, **Then** todos los items se desmarcan (toggle)
5. **Given** tengo 3 archivos marcados, **When** presiono F5 (copiar), **Then** aparece diálogo "Copiar 3 items a [destino]?"
6. **Given** confirmo copia de múltiples items, **When** la operación comienza, **Then** se muestra barra de progreso global "Copiando 2/3: archivo2.txt (45%)" con progreso por archivo y progreso total
7. **Given** tengo múltiples items marcados, **When** presiono F8 (eliminar), **Then** aparece confirmación "¿Eliminar 3 items seleccionados? (S/N)"
8. **Given** completo operación sobre items marcados, **When** la operación termina exitosamente, **Then** las marcas se limpian automáticamente y se muestra "3 items copiados/movidos/eliminados"
9. **Given** tengo items marcados, **When** presiono Esc, **Then** todas las marcas se limpian sin realizar ninguna operación
10. **Given** tengo items marcados y navego a otra carpeta, **When** cambio de directorio con Enter o Backspace, **Then** las marcas se limpian automáticamente (las selecciones no persisten entre directorios)
11. **Given** tengo items marcados en el panel izquierdo, **When** cambio al panel derecho con Tab, **Then** las marcas del panel izquierdo se mantienen visibles pero inactivas hasta que vuelva a ese panel
12. **Given** hay error durante operación en lote (ej. falta permiso en archivo 2 de 5), **When** ocurre el error, **Then** se muestra diálogo "(C)ontinuar con siguientes / (R)eintentar / (A)bortar operación?" sin perder progreso de items ya procesados

**Visual Feedback Requirements**:
- Items marcados DEBEN mostrarse con indicador claro (ej. "* archivo.txt" o background color diferente)
- Contador de items marcados DEBE mostrarse en header del panel (ej. "3 items seleccionados")
- Durante operaciones en lote, progreso DEBE mostrar: item actual (X/total), nombre archivo actual, progreso archivo actual (%), progreso global estimado

---

### User Story 6 - Previsualización de Archivos de Texto (Priority: P6)

Como usuario, quiero previsualizar el contenido de archivos de texto (txt, md, log, json, xml, código fuente, etc.) presionando F4 en un diálogo modal, para ver rápidamente el contenido sin tener que abrir un editor externo.

**Why this priority**: Feature de conveniencia que mejora la experiencia de usuario al explorar archivos. No es crítico para operaciones básicas pero muy útil para revisar contenido rápidamente.

**Independent Test**: Puede probarse independientemente seleccionando un archivo de texto (ej. README.md), presionando F4, verificando que aparece un diálogo modal centrado mostrando el contenido del archivo con scroll, syntax highlighting opcional, y que Esc cierra el modal.

**Acceptance Scenarios**:

1. **Given** tengo un archivo de texto seleccionado (ej. .txt, .md, .rs, .json), **When** presiono F4, **Then** aparece diálogo modal centrado mostrando el contenido del archivo
2. **Given** el diálogo de preview está abierto, **When** el archivo tiene más líneas que el alto del modal, **Then** puedo hacer scroll con flechas arriba/abajo y Page Up/Page Down
3. **Given** estoy viendo un archivo de texto, **When** presiono Esc o Q, **Then** el diálogo se cierra y vuelvo a la vista de paneles
4. **Given** selecciono un archivo de código fuente (ej. .rs, .py, .js), **When** presiono F4, **Then** el contenido se muestra con números de línea en el margen izquierdo
5. **Given** el archivo es muy grande (>1MB), **When** presiono F4, **Then** se muestra mensaje "Cargando..." mientras se lee el archivo
6. **Given** el archivo no es de texto (binario), **When** presiono F4, **Then** se muestra mensaje "No se puede previsualizar: archivo binario"
7. **Given** el diálogo de preview está abierto, **When** el archivo tiene encoding UTF-8 válido, **Then** se muestran correctamente caracteres especiales (ñ, tildes, emoji)
8. **Given** estoy previsualizando un archivo, **When** presiono Home/End, **Then** el scroll salta al inicio/final del archivo
9. **Given** el archivo de texto es muy largo (>10,000 líneas), **When** abro preview, **Then** se muestra indicador de posición "Línea 150/10,523 (1%)" en la parte inferior del modal
10. **Given** tengo un archivo seleccionado, **When** no tengo permisos de lectura, **Then** F4 muestra mensaje "Error: Permiso denegado para leer archivo"

**Modal Design Requirements**:
- Modal DEBE ocupar ~80% del ancho y ~80% del alto del terminal
- Modal DEBE mostrar título con nombre del archivo y tamaño en la parte superior
- Modal DEBE mostrar borde claro y fondo que lo diferencie de los paneles
- Modal DEBE mostrar hint en la parte inferior: "↑↓: Scroll | Home/End: Inicio/Fin | Esc: Cerrar"
- Contenido DEBE usar fuente monoespaciada preservando indentación original

---

### User Story 7 - Previsualización de Imágenes (Priority: P7)

Como usuario, quiero previsualizar imágenes (PNG, JPG, GIF, BMP) presionando F4 en un diálogo modal mostrando una representación ASCII art o bloques Unicode, para identificar imágenes sin salir del terminal.

**Why this priority**: Feature avanzado de visualización. Menos crítico que preview de texto pero útil para usuarios que trabajan con archivos multimedia. Requiere procesamiento de imagen a ASCII/Unicode.

**Independent Test**: Puede probarse independientemente seleccionando un archivo de imagen (ej. logo.png), presionando F4, verificando que aparece un diálogo modal mostrando representación visual de la imagen usando caracteres ASCII o bloques Unicode, con información de dimensiones y formato.

**Acceptance Scenarios**:

1. **Given** tengo un archivo de imagen seleccionado (.png, .jpg, .jpeg, .gif, .bmp), **When** presiono F4, **Then** aparece diálogo modal centrado mostrando representación ASCII/Unicode de la imagen
2. **Given** el diálogo de imagen está abierto, **When** la imagen es demasiado grande para el modal, **Then** se escala automáticamente manteniendo aspect ratio
3. **Given** estoy viendo preview de imagen, **When** presiono Esc o Q, **Then** el diálogo se cierra y vuelvo a la vista de paneles
4. **Given** la imagen tiene dimensiones muy grandes (ej. 4K), **When** presiono F4, **Then** se muestra mensaje "Cargando imagen..." durante el procesamiento
5. **Given** el archivo de imagen está corrupto, **When** presiono F4, **Then** se muestra mensaje "Error: No se puede decodificar la imagen"
6. **Given** el diálogo de imagen está abierto, **When** el modal se muestra, **Then** se incluye metadata en el título: "imagen.png (1920x1080, 2.5 MB, PNG)"
7. **Given** tengo terminal con soporte de colores (256 colors o truecolor), **When** previsualizo imagen, **Then** la representación usa colores para mejor fidelidad visual
8. **Given** tengo terminal limitado a 16 colores, **When** previsualizo imagen, **Then** se usa representación ASCII en escala de grises
9. **Given** la imagen es muy pequeña (<100x100 px), **When** previsualizo, **Then** se muestra a tamaño real sin escalar
10. **Given** el archivo de imagen es muy grande (>10MB), **When** presiono F4, **Then** se muestra confirmación "Imagen grande (12 MB). ¿Previsualizar? (S/N)" antes de cargar
11. **Given** estoy viendo imagen y terminal soporta bloques Unicode, **When** el preview se renderiza, **Then** se usan caracteres de bloque medio (▀▄█) para mejor resolución vertical

**Modal Design Requirements**:
- Modal DEBE ocupar ~90% del ancho y ~90% del alto del terminal para maximizar área de visualización
- Modal DEBE mostrar título con: nombre, dimensiones originales, tamaño archivo, formato
- Modal DEBE centrar la imagen renderizada dentro del modal
- Modal DEBE mostrar hint en la parte inferior: "Esc: Cerrar"
- Conversión de imagen DEBE priorizar bloques Unicode si terminal soporta, fallback a ASCII art
- Colores DEBEN adaptarse automáticamente a capacidad del terminal (truecolor > 256 > 16 > monocromo)

---

### User Story 8 - Descompresión de Archivos (Priority: P8)

Como usuario, quiero descomprimir archivos comprimidos (ZIP, TAR.GZ, TAR.BZ2, 7Z, RAR) presionando F9 sobre el archivo, con soporte para archivos protegidos con contraseña, para extraer contenido sin salir del explorador.

**Why this priority**: Feature de conveniencia avanzado para gestión de archivos comprimidos. No es crítico para el MVP pero muy útil para workflows que involucran descargas y backups comprimidos.

**Independent Test**: Puede probarse independientemente seleccionando un archivo comprimido (ej. backup.zip), presionando F9, verificando que aparece diálogo modal mostrando lista de archivos dentro del archivo, opción de seleccionar destino de extracción, y barra de progreso durante descompresión. Para archivos con contraseña, debe aparecer campo de entrada de password antes de extraer.

**Acceptance Scenarios**:

1. **Given** tengo un archivo ZIP seleccionado, **When** presiono F9, **Then** aparece diálogo modal "Extraer archivo.zip" mostrando lista de archivos contenidos con preview de estructura (carpetas y archivos)
2. **Given** el diálogo de extracción está abierto, **When** el modal se muestra, **Then** veo información: cantidad de archivos, tamaño total descomprimido, ratio de compresión
3. **Given** estoy viendo el diálogo de extracción, **When** presiono Enter o E (Extraer), **Then** aparece diálogo "Destino de extracción:" pre-rellenado con ruta del panel opuesto
4. **Given** confirmo extracción a directorio destino, **When** la extracción comienza, **Then** se muestra barra de progreso "Extrayendo 5/23: documento.pdf (22%)" con progreso por archivo y global
5. **Given** el archivo comprimido está protegido con contraseña (ZIP/7Z/RAR), **When** presiono F9, **Then** aparece campo de entrada "Contraseña:" con caracteres ocultos (*****)
6. **Given** ingreso contraseña correcta, **When** presiono Enter, **Then** se procede con la extracción normalmente
7. **Given** ingreso contraseña incorrecta, **When** intento extraer, **Then** se muestra error "Contraseña incorrecta" y opción de "(R)eintentar / (C)ancelar"
8. **Given** el archivo comprimido contiene múltiples carpetas anidadas, **When** extraigo, **Then** se preserva la estructura de directorios original
9. **Given** hay archivo con mismo nombre en destino durante extracción, **When** ocurre colisión, **Then** aparece diálogo "(S)obreescribir este / Sobreescribir (T)odos / (R)enombrar / (O)mitir / (C)ancelar"
10. **Given** estoy extrayendo archivo grande (varios GB), **When** presiono Esc durante extracción, **Then** aparece confirmación "¿Cancelar extracción? Archivos parciales se eliminarán (S/N)"
11. **Given** el archivo comprimido está corrupto, **When** presiono F9, **Then** se muestra error "No se puede leer: archivo corrupto o formato no soportado"
12. **Given** selecciono archivo TAR.GZ o TAR.BZ2, **When** presiono F9, **Then** se detecta automáticamente el formato y se descomprime en un solo paso
13. **Given** el archivo comprimido contiene enlaces simbólicos (TAR), **When** extraigo, **Then** se preservan los enlaces simbólicos en sistemas que lo soporten
14. **Given** no hay espacio suficiente en destino, **When** intento extraer, **Then** se muestra error "Espacio insuficiente: se necesitan 2.5 GB, disponibles 1.2 GB"
15. **Given** el archivo RAR está dividido en múltiples partes (file.part1.rar, file.part2.rar), **When** selecciono la primera parte y presiono F9, **Then** se detectan automáticamente las otras partes y se extraen todas

**Modal Design Requirements**:
- Modal inicial DEBE mostrar lista de archivos contenidos con scroll si es larga
- Modal DEBE mostrar estadísticas: cantidad de archivos, tamaño comprimido vs descomprimido, ratio
- Campo de contraseña DEBE ocultar caracteres con asteriscos (*) o puntos (•)
- Durante extracción, progreso DEBE mostrar: archivo actual (X/total), nombre, progreso individual (%), progreso global estimado
- Modal DEBE mostrar hints: "Enter: Extraer | L: Listar contenido | Esc: Cerrar"
- Estructura de carpetas en preview DEBE usar indentación o caracteres de árbol (├─ └─)

**Formato Support Requirements**:
- Sistema DEBE soportar: ZIP, TAR, TAR.GZ, TAR.BZ2, TAR.XZ, 7Z, RAR (lectura)
- Sistema DEBE detectar formato automáticamente por magic bytes, no solo por extensión
- Sistema DEBE manejar archivos ZIP con contraseña (encrypted entries)
- Sistema DEBE manejar archivos 7Z con contraseña
- Sistema DEBE manejar archivos RAR con contraseña (v4 y v5)

---

### Edge Cases

- **¿Qué pasa cuando se intenta copiar un archivo a sí mismo?**: Mostrar error "Origen y destino son iguales"
- **¿Qué pasa cuando hay un archivo con el mismo nombre en destino?**: Mostrar diálogo "(S)obreescribir / (R)enombrar / (C)ancelar"
- **¿Cómo maneja el sistema permisos insuficientes?**: Mostrar error específico "Permiso denegado: [detalle]" sin crashear
- **¿Qué pasa si se llena el disco durante una copia?**: Detectar error de espacio, mostrar mensaje, limpiar archivos parciales
- **¿Cómo se manejan enlaces simbólicos?**: Mostrar con indicador visual (ej. "->"), al copiar preguntar si copiar enlace o contenido
- **¿Qué pasa con archivos ocultos (dotfiles)?**: Mostrarse por defecto con color diferenciado
- **¿Cómo se manejan rutas muy largas?**: Truncar con "..." en medio, mostrar ruta completa en barra de estado
- **¿Qué pasa al navegar a directorio sin permisos de lectura?**: Mostrar error y mantener en directorio anterior
- **¿Cómo se cancelan operaciones largas?**: Presionar Esc durante progreso cancela operación con confirmación
- **¿Qué pasa si marco items y luego aplico un filtro?**: Las marcas se mantienen solo en items que coinciden con el filtro; items filtrados pierden marca
- **¿Qué pasa al copiar múltiples items y uno falla?**: Mostrar opción de continuar/abortar, registrar items fallidos para reporte final
- **¿Puedo marcar items en ambos paneles simultáneamente?**: No, las marcas son independientes por panel pero solo el panel activo puede marcar/desmarcar
- **¿Qué pasa si presiono F4 en una carpeta?**: Mostrar mensaje "No se puede previsualizar: es un directorio"
- **¿Cómo maneja archivos de texto con encoding no-UTF8?**: Intentar detectar encoding (Latin-1, CP1252), mostrar mensaje si falla: "Encoding no soportado"
- **¿Qué pasa si la imagen tiene transparencia (PNG alpha)?**: Renderizar con fondo del terminal, transparencia se convierte a color de fondo
- **¿Puedo previsualizar mientras hay operación de copia en progreso?**: Sí, preview no bloquea operaciones de fondo
- **¿Qué pasa con archivos de imagen animados (GIF animado)?**: Mostrar solo primer frame con nota "(GIF animado - frame 1)"
- **¿Qué pasa si presiono F9 en un archivo que no es comprimido?**: Mostrar mensaje "No es un archivo comprimido o formato no reconocido"
- **¿Puedo extraer solo algunos archivos del comprimido en vez de todos?**: No en esta versión, se extrae todo el contenido (extracción selectiva out of scope)
- **¿Qué pasa con archivos comprimidos que contienen rutas absolutas?**: Convertir a rutas relativas por seguridad, mostrar warning
- **¿Cómo maneja archivos comprimidos con nombres de archivo duplicados internamente?**: Mantener solo el último encontrado, registrar warning
- **¿Qué pasa si el archivo comprimido contiene bombas ZIP (archivos diseñados para expandirse enormemente)?**: Limitar extracción a 10GB descomprimido, mostrar error si excede

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Sistema DEBE mostrar dos paneles verticales lado a lado ocupando cada uno ~50% del ancho terminal
- **FR-002**: Sistema DEBE soportar navegación completa con teclado sin requerir mouse
- **FR-003**: Sistema DEBE mostrar para cada archivo: nombre, tamaño (formato humano: KB/MB/GB), fecha modificación, permisos
- **FR-004**: Sistema DEBE distinguir visualmente entre archivos regulares, carpetas, enlaces simbólicos, y ejecutables
- **FR-005**: Sistema DEBE mostrar el directorio actual (pwd) en la parte superior de cada panel
- **FR-006**: Sistema DEBE implementar barra de estado global inferior mostrando: teclas disponibles (F5:Copiar F6:Mover F7:Mkdir F8:Del)
- **FR-007**: Sistema DEBE manejar resize del terminal redibujando correctamente los paneles
- **FR-008**: Sistema DEBE implementar operaciones de archivos de forma asíncrona (no bloquear UI)
- **FR-009**: Sistema DEBE mostrar feedback visual inmediato (<100ms) para toda interacción de usuario
- **FR-010**: Sistema DEBE persistir estado (directorios actuales de ambos paneles) al salir y restaurar al iniciar
- **FR-011**: Sistema DEBE manejar errores de IO sin crashear, mostrando mensajes claros al usuario
- **FR-012**: Sistema DEBE soportar rutas absolutas y relativas en ambos paneles independientemente
- **FR-013**: Sistema DEBE implementar confirmaciones para operaciones destructivas (eliminar, sobreescribir)
- **FR-014**: Sistema DEBE mostrar progreso para operaciones que tomen >1 segundo
- **FR-015**: Sistema DEBE soportar selección múltiple con Espacio (toggle individual) y Ctrl+A (toggle todos)
- **FR-016**: Sistema DEBE previsualizar archivos de texto con F4 mostrando contenido en modal con scroll
- **FR-017**: Sistema DEBE previsualizar imágenes con F4 mostrando representación ASCII/Unicode art en modal
- **FR-018**: Sistema DEBE descomprimir archivos ZIP/TAR/7Z/RAR con F9, incluyendo soporte para contraseñas
- **FR-019**: Sistema DEBE mostrar items marcados con indicador visual claro (ej. asterisco o background color)
- **FR-020**: Sistema DEBE aplicar operaciones (copiar/mover/eliminar) a todos los items marcados cuando hay selección múltiple
- **FR-021**: Sistema DEBE detectar encoding de archivos de texto (UTF-8, Latin-1, CP1252) automáticamente
- **FR-022**: Sistema DEBE adaptar preview de imágenes a capacidad del terminal (truecolor > 256 colors > 16 colors)
- **FR-023**: Sistema DEBE detectar formato de archivos comprimidos por magic bytes, no solo por extensión

### Key Entities

- **Panel**: Representa una vista de directorio con: ruta actual, lista de entries, índice de selección, estado de scroll
- **FileEntry**: Representa un item del filesystem con: nombre, tipo (File/Dir/Symlink), tamaño, permisos, fecha modificación
- **Operation**: Representa operación de archivo en progreso: tipo (Copy/Move/Delete), origen, destino, progreso (bytes/total)
- **AppState**: Estado global con: panel izquierdo, panel derecho, panel activo, operación actual, filtro activo, historial de navegación
- **SelectionState**: Gestiona items marcados por panel: HashSet de paths marcados, contador de seleccionados
- **PreviewState**: Estado de preview modal: tipo (Text/Image), contenido cargado, posición de scroll, dimensiones
- **ArchiveEntry**: Representa archivo dentro de comprimido: nombre, tamaño comprimido/descomprimido, es_carpeta
- **ArchiveOperation**: Operación de extracción: formato detectado, total archivos, progreso actual, contraseña opcional

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Usuario puede navegar entre paneles y directorios en <500ms por operación (respuesta inmediata)
- **SC-002**: Usuario puede copiar archivo de 100MB con barra de progreso actualizada cada 250ms
- **SC-003**: 100% de operaciones de archivo con confirmación previa para acciones destructivas (cero eliminaciones accidentales)
- **SC-004**: Sistema se recupera de 100% de errores de IO sin crashear (errores de permisos, disco lleno, etc.)
- **SC-005**: Usuario puede completar flujo completo (navegar -> copiar -> crear carpeta -> mover) usando solo teclado sin consultar documentación (teclas visibles en UI)

## Assumptions *(optional)*

- Se asume que el terminal soporta códigos ANSI para colores y control de cursor (mínimo 256 colores)
- Se asume terminal con ancho mínimo de 80 columnas para mostrar ambos paneles decentemente
- Se asume filesystem POSIX-like (Linux/macOS) o Windows con soporte de rutas estándar
- Se asume usuario tiene permisos de lectura en directorio HOME para punto de inicio
- Se asume que operaciones de archivo pueden tardar (archivos grandes) y necesitan ser async
- No se asume conocimiento previo de shortcuts - todos deben estar visibles en UI

## Out of Scope *(optional)*

**Explícitamente NO incluido en esta feature**:

- **Edición de archivos integrada**: No habrá editor de texto dentro del explorador (usar editor externo)
- **Preview de PDFs o documentos Office**: Solo texto plano e imágenes están soportados
- **Preview de videos**: No reproducción ni thumbnails de archivos de video
- **Operaciones de red**: No FTP, SSH, o montaje de recursos remotos
- **Creación de archivos comprimidos**: Solo descompresión está soportada, no crear ZIP/TAR nuevos
- **Extracción selectiva de archivos**: No seleccionar archivos individuales dentro del comprimido, se extrae todo
- **Modificación de archivos comprimidos**: No añadir/eliminar archivos dentro de un ZIP existente
- **Calculadora de checksums**: No MD5, SHA256, etc.
- **Integración con Git**: No comandos git, status de archivos versionados
- **Marcadores/Bookmarks**: No hay sistema de guardado de ubicaciones favoritas
- **Historial de comandos**: No undo/redo de operaciones de archivo
- **Papelera de reciclaje**: Delete es permanente (con confirmación)
- **Temas/Personalización**: Colores y layout son fijos (por ahora)
- **Plugins o extensibilidad**: No hay sistema de plugins
- **Comparación de archivos**: No diff entre archivos de ambos paneles
- **Syntax highlighting avanzado**: No resaltado de sintaxis completo, solo números de línea
- **Búsqueda dentro de preview**: No búsqueda de texto dentro del contenido previsualizado
- **Preview de contenido de archivos comprimidos**: No ver archivos dentro del ZIP sin extraer

**Razón del scope**: Mantener MVP enfocado en navegación y operaciones básicas de archivos. Features avanzados pueden añadirse en futuras iteraciones basadas en feedback de usuarios.

---

## Notes for Planning Phase

**Sugerencias para `plan.md`**:

- Estructura de proyecto Rust: `src/main.rs`, `src/app.rs`, `src/ui/`, `src/fs/`, `src/models/`
- Dependencias clave: `ratatui` (0.25+), `crossterm` (0.27+), `tokio` (1.35+), `walkdir` (2.5+)
- Event loop con `crossterm::event::read()` para keyboard input
- Layout de Ratatui con `Layout::split()` para dividir terminal en 2 columnas + header + footer
- Async file operations con `tokio::fs` para no bloquear UI
- Progress tracking con channels (mpsc) entre thread de IO y UI thread
- Persistencia con `serde_json` guardando estado en `~/.config/leeky-explorer/state.json`
- Testing con `cargo test` + integration tests simulando teclado con eventos mock