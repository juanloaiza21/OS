# Analizador de Viajes CSV

![Versión](https://img.shields.io/badge/Versión-1.0.0-brightgreen)
![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)
![Memoria](https://img.shields.io/badge/Uso%20de%20Memoria-<10MB-blue)

**Fecha:** 2025-06-26 05:19:55  
**Desarrollador:** juanloaiza21

## Descripción

Analizador de Viajes es una aplicación de alto rendimiento y bajo consumo de memoria desarrollada para procesar, filtrar y analizar grandes conjuntos de datos CSV de viajes (de hasta 3GB+) manteniendo un uso máximo de memoria de 10MB. Utiliza técnicas avanzadas de procesamiento por streaming y almacenamiento en disco para ofrecer un rendimiento excepcional incluso en equipos con recursos limitados.

## Características Principales

- **Procesamiento optimizado para memoria**: Lectura por streaming sin cargar archivos completos
- **Interfaz gráfica intuitiva**: Navegación por pestañas y visualización de datos en tiempo real
- **Filtrado avanzado**: Por precio, índice y destino con salida a archivo CSV
- **Estadísticas detalladas**: Análisis de datos con métricas relevantes
- **Tablas hash basadas en disco**: Búsqueda rápida sin consumo de memoria RAM
- **Registro de operaciones**: Seguimiento detallado del proceso en consola

## Requisitos del Sistema

- Rust 1.70 o superior
- Sistemas operativos compatibles: Linux, macOS, Windows
- 2GB de RAM (mínimo)
- 100MB de espacio en disco (+ espacio para archivos de datos)

## Instalación

### Opción 1: Compilar desde el código fuente

1. Clonar el repositorio:
   ```bash
   git clone https://github.com/juanloaiza21/analizador-viajes.git
   cd analizador-viajes
   ```

2. Compilar en modo de lanzamiento (optimizado):
   ```bash
   cargo build --release
   ```

3. Ejecutar la aplicación:
   ```bash
   cargo run --release
   ```

### Opción 2: CMAKE

1. Agregar cmake

## Estructura de Archivos

```
analizador-viajes/
├── src/
│   ├── data/              # Módulos de procesamiento de datos
│   │   ├── trip_struct.rs # Estructura para almacenar viajes
│   │   ├── data_lector.rs # Lectura optimizada de CSV
│   │   ├── disk_hash.rs   # Tabla hash basada en disco
│   │   ├── filters.rs     # Filtros de datos
│   │   └── mod.rs         # Módulo principal de datos
│   ├── visual/            # Interfaz gráfica
│   │   ├── visual.rs      # Implementación UI
│   │   └── mod.rs         # Exportación de funciones
│   └── main.rs            # Punto de entrada
├── tmp/                   # Directorio para tablas hash (generado automáticamente)
├── Cargo.toml             # Configuración y dependencias
└── README.md              # Este archivo
```

## Guía de Uso

### 1. Inicio

Al iniciar la aplicación, verás una pantalla de bienvenida con información del usuario y fecha actual. Haz clic en "Comenzar" para acceder a la aplicación.

### 2. Interfaz Principal

La interfaz está organizada en pestañas para una navegación intuitiva:

- **Inicio**: Configuración principal y acciones rápidas
- **Ver Datos**: Visualización de registros cargados
- **Filtros**: Aplicación de filtros por precio, índice y destino
- **Estadísticas**: Análisis de datos y destinos populares
- **Config**: Ajustes de la aplicación
- **Acerca de**: Información del software

### 3. Carga de Datos

1. En la pestaña **Inicio**, haz clic en "Seleccionar Archivo CSV"
2. Selecciona un archivo CSV con datos de viajes
3. Selecciona un directorio para guardar los resultados filtrados
4. Haz clic en "Cargar Vista Previa" para ver una muestra de los datos

### 4. Filtrado de Datos

#### Filtro por Precio
1. Ve a la pestaña **Filtros**
2. Expande la sección "Filtro por Precio"
3. Introduce los valores de precio mínimo y máximo
4. Haz clic en "Aplicar Filtro de Precio"
5. El resultado se guardará en el directorio de salida

#### Filtro por Índice
1. Expande la sección "Filtro por Índice"
2. Introduce el índice exacto a buscar
3. Haz clic en "Buscar por Índice"
4. Si se encuentra, se guardará en el directorio de salida

#### Filtro por Destino
1. Expande la sección "Filtro por Destino"
2. Introduce el ID de destino
3. Haz clic en "Filtrar por Destino"
4. Los resultados se guardarán en el directorio de salida

### 5. Análisis Estadístico

1. Ve a la pestaña **Estadísticas**
2. Revisa las estadísticas de la vista previa
3. Haz clic en "Calcular Destinos Más Populares" para ver un ranking
4. Si has aplicado filtros, también podrás ver estadísticas específicas sobre los resultados

### 6. Tabla Hash

Para búsquedas más rápidas en archivos grandes:

1. En la pestaña **Inicio**, verás el directorio predeterminado para tablas hash (`./tmp`)
2. Haz clic en "Construir Tabla Hash" después de seleccionar un archivo CSV
3. La aplicación creará índices para búsquedas rápidas
4. El progreso se mostrará en la consola

### 7. Configuración

En la pestaña **Config** puedes:

1. Cambiar entre tema oscuro y claro
2. Ajustar el número máximo de filas en la vista previa
3. Activar/desactivar los mensajes detallados en consola
4. Cambiar la ubicación del directorio de tabla hash
5. Limpiar mensajes de consola

## Arquitectura y Funcionamiento

### Optimización de Memoria

La aplicación utiliza varias técnicas para mantener un uso de memoria por debajo de 10MB:

1. **Procesamiento por streaming**: Lee el CSV línea por línea sin cargarlo completo
2. **Vista previa limitada**: Solo carga en memoria un número configurable de registros
3. **Tabla hash basada en disco**: Almacena índices en disco en vez de memoria RAM
4. **Operaciones en segundo plano**: Procesamiento asíncrono para grandes conjuntos de datos

### Estructuras de Datos

- `Trip`: Almacena información de un viaje individual
- `TripFilter`: Define criterios de filtrado (precio, índice, destino)
- `DiskHashTable`: Implementa tabla hash con almacenamiento en disco

### Flujo de Operaciones

1. Se selecciona un archivo CSV para procesar
2. Los datos se procesan por streaming para filtrado o estadísticas
3. Los resultados se almacenan en archivos CSV en el directorio de salida
4. Para búsquedas rápidas, se construye una tabla hash en disco
5. La interfaz muestra resultados y estadísticas de forma amigable

## Registro en Consola

La aplicación proporciona un registro detallado de operaciones en la consola:

- Inicio de operaciones con marca de tiempo
- Progreso en forma de barra durante operaciones largas
- Tiempo de ejecución para cada operación
- Resultados y estadísticas
- Errores y advertencias

## Solución de Problemas

### Error al abrir archivo CSV
- Verifica que el archivo exista y tenga permisos de lectura
- Asegúrate de que tenga formato CSV válido
- Comprueba que las columnas tengan los nombres esperados

### Rendimiento lento
- Reduce el número máximo de filas en la vista previa
- Asegúrate de que el directorio de tabla hash esté en un disco rápido
- Para archivos muy grandes, considera filtrar primero por criterios específicos

### Error al construir tabla hash
- Verifica que tengas permisos de escritura en el directorio `./tmp`
- Asegúrate de tener suficiente espacio en disco
- Si el directorio no existe, la aplicación intentará crearlo

### La interfaz no responde
- Las operaciones pesadas se ejecutan en segundo plano, espera a que terminen
- Comprueba la consola para ver el progreso
- Si persiste, reinicia la aplicación


---
