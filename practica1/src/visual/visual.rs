use crate::data::filters::{TripFilter, filter_to_file, get_filter_stats, get_popular_destinations};
use crate::data::trip_struct::Trip;
use crate::data::data_lector::stream_process_csv;
use crate::data::disk_hash::build_hash_table_from_csv;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;
use eframe::egui;
use egui::{Color32, RichText, Ui};
use std::fs;
use std::io::Write;
use std::time::Instant;

// Constantes del sistema
const APP_VERSION: &str = "1.0.0";
const MAX_MEMORY_USAGE: &str = "10MB";
const CURRENT_DATE: &str = "2025-06-26 04:35:17";
const CURRENT_USER: &str = "juanloaiza21";

// Implementar Clone para TripFilter (faltaba)
impl Clone for TripFilter {
    fn clone(&self) -> Self {
        match self {
            TripFilter::Price { min, max } => TripFilter::Price {
                min: min.clone(),
                max: max.clone(),
            },
            TripFilter::Index(idx) => TripFilter::Index(idx.clone()),
            TripFilter::Destination(dest) => TripFilter::Destination(dest.clone()),
            TripFilter::And(filters) => TripFilter::And(filters.clone()),
            TripFilter::Or(filters) => TripFilter::Or(filters.clone()),
        }
    }
}

// Estructura para almacenar resultados de operaciones en segundo plano
struct BackgroundTaskResult {
    message: String,
    success: bool,
    data: Option<Vec<Trip>>,
    stats: Option<HashMap<String, f64>>,
    popular_destinations: Option<Vec<(String, usize)>>,
}

// Estado de la aplicación
pub struct TripsVisualApp {
    // Información de usuario y sistema
    username: String,
    current_datetime: String,
    
    // Rutas de archivos
    csv_path: Option<PathBuf>,
    output_dir: Option<PathBuf>,
    hash_dir: Option<PathBuf>,
    
    // Estado de filtros
    filter_price_min: String,
    filter_price_max: String,
    filter_index: String,
    filter_destination: String,
    
    // Estado de la tarea en segundo plano
    bg_task_running: bool,
    bg_task_result: Option<Arc<BackgroundTaskResult>>,
    
    // Datos temporales (limitados para no exceder memoria)
    preview_data: Vec<Trip>,
    
    // Pestaña actual
    current_tab: Tab,
    
    // Configuración
    max_preview_rows: usize,
    show_welcome: bool,
    theme_dark: bool,
    
    // Configuración de consola
    show_console_progress: bool,
}

// Pestañas de la interfaz
enum Tab {
    Home,
    DataViewer,
    Filters,
    Statistics,
    Settings,
    About,
}

// Función para imprimir en consola con fecha y hora
fn log_to_console(message: &str) {
    println!("[{}] {}", CURRENT_DATE, message);
}

// Función para crear el directorio tmp si no existe
fn ensure_tmp_dir_exists() -> PathBuf {
    let tmp_dir = PathBuf::from("./tmp");
    if !tmp_dir.exists() {
        log_to_console(&format!("Creando directorio para tablas hash: ./tmp"));
        fs::create_dir_all(&tmp_dir).expect("No se pudo crear el directorio tmp");
        log_to_console("Directorio ./tmp creado exitosamente");
    } else {
        log_to_console("Directorio ./tmp ya existe, usando como ubicación para tablas hash");
    }
    tmp_dir
}

impl Default for TripsVisualApp {
    fn default() -> Self {
        // Crear directorio tmp para tablas hash
        let hash_dir = ensure_tmp_dir_exists();
        
        log_to_console("Inicializando aplicación con los siguientes parámetros:");
        log_to_console(&format!("- Usuario: {}", CURRENT_USER));
        log_to_console(&format!("- Fecha: {}", CURRENT_DATE));
        log_to_console(&format!("- Directorio hash: {}", hash_dir.display()));
        
        Self {
            username: CURRENT_USER.to_string(),
            current_datetime: CURRENT_DATE.to_string(),
            csv_path: None,
            output_dir: None,
            hash_dir: Some(hash_dir),
            filter_price_min: String::new(),
            filter_price_max: String::new(),
            filter_index: String::new(),
            filter_destination: String::new(),
            bg_task_running: false,
            bg_task_result: None,
            preview_data: Vec::new(),
            current_tab: Tab::Home,
            max_preview_rows: 100,
            show_welcome: true,
            theme_dark: true,
            show_console_progress: true,
        }
    }
}

impl TripsVisualApp {
    // Crear una nueva instancia de la aplicación
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        log_to_console("Iniciando aplicación Analizador de Viajes");
        
        // Configurar tema oscuro por defecto
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.visuals.override_text_color = Some(Color32::from_rgb(240, 240, 240));
        cc.egui_ctx.set_style(style);
        
        log_to_console("Tema visual configurado: Oscuro");
        
        Self::default()
    }
    
    // Actualizar la fecha y hora
    fn update_datetime(&mut self) {
        self.current_datetime = CURRENT_DATE.to_string();
    }
    
    // Seleccionar archivo CSV
    fn select_csv_file(&mut self, ui: &mut Ui) {
        if ui.button("📂 Seleccionar Archivo CSV").clicked() {
            log_to_console("Abriendo diálogo para seleccionar archivo CSV...");
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("CSV", &["csv"])
                .set_title("Seleccionar archivo CSV")
                .pick_file() 
            {
                log_to_console(&format!("Archivo CSV seleccionado: {}", path.display()));
                self.csv_path = Some(path);
                self.preview_data.clear(); // Limpiar datos previos
            } else {
                log_to_console("Selección de archivo CSV cancelada");
            }
        }
        
        if let Some(path) = &self.csv_path {
            ui.horizontal(|ui| {
                ui.label("Archivo seleccionado:");
                ui.monospace(path.to_string_lossy().to_string());
            });
            
            // Botón para cargar vista previa
            if ui.button("👁️ Cargar Vista Previa").clicked() && !self.bg_task_running {
                self.load_preview_data();
            }
        }
    }
    
    // Cargar datos de vista previa (limitados)
    fn load_preview_data(&mut self) {
        if let Some(csv_path) = &self.csv_path {
            let csv_path_clone = csv_path.clone();
            let max_rows = self.max_preview_rows;
            self.bg_task_running = true;
            
            log_to_console(&format!("Cargando vista previa de datos (máximo {} filas)...", max_rows));
            
            let result = Arc::new(Mutex::new(None));
            let result_clone = Arc::clone(&result);
            let show_progress = self.show_console_progress;
            
            // Ejecutar en segundo plano para no bloquear la UI
            thread::spawn(move || {
                let start_time = Instant::now();
                let mut preview_data = Vec::with_capacity(max_rows);
                let mut count = 0;
                let mut last_progress = 0;
                
                if show_progress {
                    print!("Progreso: [");
                    std::io::stdout().flush().unwrap();
                }
                
                let _ = stream_process_csv(&csv_path_clone, |trip| {
                    if count < max_rows {
                        // Mostrar progreso en consola
                        if show_progress && count % (max_rows / 10) == 0 && count > 0 {
                            let progress = count * 10 / max_rows;
                            if progress > last_progress {
                                for _ in 0..(progress - last_progress) {
                                    print!("#");
                                    std::io::stdout().flush().unwrap();
                                }
                                last_progress = progress;
                            }
                        }
                        
                        preview_data.push(trip.clone());
                        count += 1;
                        Ok(())
                    } else {
                        // Terminar temprano cuando alcancemos el máximo
                        Err("Límite de vista previa alcanzado".into())
                    }
                }).or_else(|e| {
                    if e.to_string() == "Límite de vista previa alcanzado" {
                        Ok(())
                    } else {
                        Err(e)
                    }
                });
                
                // Completar barra de progreso
                if show_progress {
                    for _ in last_progress..10 {
                        print!("#");
                    }
                    println!("] Completado");
                }
                
                let elapsed = start_time.elapsed();
                log_to_console(&format!("Vista previa cargada: {} registros en {:.2} segundos", 
                                     preview_data.len(), elapsed.as_secs_f64()));
                
                // Guardar resultado
                let task_result = BackgroundTaskResult {
                    message: format!("Vista previa cargada: {} registros", preview_data.len()),
                    success: true,
                    data: Some(preview_data),
                    stats: None,
                    popular_destinations: None,
                };
                
                *result_clone.lock().unwrap() = Some(Arc::new(task_result));
            });
            
            // Configurar callback para cuando termine
            let app_result = Arc::new(Mutex::new(None));
            let app_result_clone = Arc::clone(&app_result);
            
            std::thread::spawn(move || {
                // Esperar hasta que el resultado esté disponible
                loop {
                    if let Some(res) = &*result.lock().unwrap() {
                        *app_result_clone.lock().unwrap() = Some(Arc::clone(res));
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            });
            
            self.bg_task_result = Some(Arc::new(BackgroundTaskResult {
                message: "Cargando vista previa...".to_string(),
                success: false,
                data: None,
                stats: None,
                popular_destinations: None,
            }));
        }
    }
    
    // Ejecutar filtro de precio
    fn run_price_filter(&mut self) {
        if let (Some(csv_path), Some(output_dir)) = (&self.csv_path, &self.output_dir) {
            let min_price = self.filter_price_min.parse::<f64>().unwrap_or(0.0);
            let max_price = self.filter_price_max.parse::<f64>().unwrap_or(f64::MAX);
            
            let csv_path_clone = csv_path.clone();
            let output_path = output_dir.join(format!("precio_{}_a_{}.csv", min_price, max_price));
            self.bg_task_running = true;
            
            log_to_console(&format!("Aplicando filtro de precio: ${} - ${}", min_price, max_price));
            log_to_console(&format!("Archivo de salida: {}", output_path.display()));
            
            let filter = TripFilter::Price {
                min: Some(min_price),
                max: Some(max_price)
            };
            
            // Ejecutar en segundo plano
            let result = Arc::new(Mutex::new(None));
            let result_clone = Arc::clone(&result);
            let show_progress = self.show_console_progress;
            
            thread::spawn(move || {
                let start_time = Instant::now();
                
                // Realizar el filtrado
                match filter_to_file(&csv_path_clone, &output_path, filter, None) {
                    Ok(count) => {
                        let elapsed = start_time.elapsed();
                        log_to_console(&format!("Filtro de precio completado: {} registros en {:.2} segundos", 
                                            count, elapsed.as_secs_f64()));
                        
                        let task_result = BackgroundTaskResult {
                            message: format!("Filtrado completado: {} registros encontrados.\nGuardado en: {}", 
                                          count, output_path.to_string_lossy()),
                            success: true,
                            data: None,
                            stats: None,
                            popular_destinations: None,
                        };
                        *result_clone.lock().unwrap() = Some(Arc::new(task_result));
                    },
                    Err(e) => {
                        log_to_console(&format!("Error al aplicar filtro de precio: {}", e));
                        
                        let task_result = BackgroundTaskResult {
                            message: format!("Error al filtrar: {}", e),
                            success: false,
                            data: None,
                            stats: None,
                            popular_destinations: None,
                        };
                        *result_clone.lock().unwrap() = Some(Arc::new(task_result));
                    }
                }
            });
            
            // Configurar callback
            let app_result = Arc::new(Mutex::new(None));
            let app_result_clone = Arc::clone(&app_result);
            
            std::thread::spawn(move || {
                loop {
                    if let Some(res) = &*result.lock().unwrap() {
                        *app_result_clone.lock().unwrap() = Some(Arc::clone(res));
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            });
            
            self.bg_task_result = Some(Arc::new(BackgroundTaskResult {
                message: format!("Filtrando viajes por precio (${} - ${})...", min_price, max_price),
                success: false,
                data: None,
                stats: None,
                popular_destinations: None,
            }));
        }
    }
    
    // Ejecutar filtro por índice
    fn run_index_filter(&mut self) {
        if let (Some(csv_path), Some(output_dir)) = (&self.csv_path, &self.output_dir) {
            let index = self.filter_index.clone();
            
            if index.is_empty() {
                return;
            }
            
            let csv_path_clone = csv_path.clone();
            let output_path = output_dir.join(format!("index_{}.csv", index));
            self.bg_task_running = true;
            
            log_to_console(&format!("Buscando viaje con índice: {}", index));
            log_to_console(&format!("Archivo de salida: {}", output_path.display()));
            
            let filter = TripFilter::Index(index.clone());
            let index_clone = index.clone(); // Clonar para usar en el mensaje final
            let show_progress = self.show_console_progress;
            
            // Ejecutar en segundo plano
            let result = Arc::new(Mutex::new(None));
            let result_clone = Arc::clone(&result);
            
            thread::spawn(move || {
                let start_time = Instant::now();
                
                // Realizar el filtrado
                match filter_to_file(&csv_path_clone, &output_path, filter, Some(1)) {
                    Ok(count) => {
                        let elapsed = start_time.elapsed();
                        
                        if count > 0 {
                            log_to_console(&format!("Viaje con índice {} encontrado en {:.2} segundos", 
                                                index, elapsed.as_secs_f64()));
                            
                            let message = format!("Viaje con índice {} encontrado.\nGuardado en: {}", 
                                                index, output_path.to_string_lossy());
                            
                            let task_result = BackgroundTaskResult {
                                message,
                                success: true,
                                data: None,
                                stats: None,
                                popular_destinations: None,
                            };
                            *result_clone.lock().unwrap() = Some(Arc::new(task_result));
                        } else {
                            log_to_console(&format!("No se encontró ningún viaje con índice {} (búsqueda tomó {:.2} segundos)", 
                                                index, elapsed.as_secs_f64()));
                            
                            let message = format!("No se encontró ningún viaje con índice {}", index);
                            
                            let task_result = BackgroundTaskResult {
                                message,
                                success: false,
                                data: None,
                                stats: None,
                                popular_destinations: None,
                            };
                            *result_clone.lock().unwrap() = Some(Arc::new(task_result));
                        }
                    },
                    Err(e) => {
                        log_to_console(&format!("Error al buscar índice {}: {}", index, e));
                        
                        let task_result = BackgroundTaskResult {
                            message: format!("Error al buscar: {}", e),
                            success: false,
                            data: None,
                            stats: None,
                            popular_destinations: None,
                        };
                        *result_clone.lock().unwrap() = Some(Arc::new(task_result));
                    }
                }
            });
            
            // Configurar callback
            let app_result = Arc::new(Mutex::new(None));
            let app_result_clone = Arc::clone(&app_result);
            
            std::thread::spawn(move || {
                loop {
                    if let Some(res) = &*result.lock().unwrap() {
                        *app_result_clone.lock().unwrap() = Some(Arc::clone(res));
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            });
            
            self.bg_task_result = Some(Arc::new(BackgroundTaskResult {
                message: format!("Buscando viaje con índice {}...", index_clone),
                success: false,
                data: None,
                stats: None,
                popular_destinations: None,
            }));
        }
    }
    
    // Ejecutar filtro por destino
    fn run_destination_filter(&mut self) {
        if let (Some(csv_path), Some(output_dir)) = (&self.csv_path, &self.output_dir) {
            let destination = self.filter_destination.clone();
            
            if destination.is_empty() {
                return;
            }
            
            let csv_path_clone = csv_path.clone();
            let stats_csv_path = csv_path.clone();
            let output_path = output_dir.join(format!("destino_{}.csv", destination));
            self.bg_task_running = true;
            
            log_to_console(&format!("Filtrando viajes con destino: {}", destination));
            log_to_console(&format!("Archivo de salida: {}", output_path.display()));
            
            let filter = TripFilter::Destination(destination.clone());
            let destination_clone = destination.clone(); // Clonar para usar en el mensaje final
            let show_progress = self.show_console_progress;
            
            // Ejecutar en segundo plano
            let result = Arc::new(Mutex::new(None));
            let result_clone = Arc::clone(&result);
            
            thread::spawn(move || {
                let start_time = Instant::now();
                
                // Realizar el filtrado
                match filter_to_file(&csv_path_clone, &output_path, filter.clone(), None) {
                    Ok(count) => {
                        let elapsed = start_time.elapsed();
                        
                        // Obtener estadísticas
                        log_to_console("Calculando estadísticas del filtro...");
                        let stats_result = get_filter_stats(&stats_csv_path, filter);
                        
                        let stats = match stats_result {
                            Ok(stats) => {
                                log_to_console("Estadísticas calculadas correctamente");
                                Some(stats)
                            },
                            Err(e) => {
                                log_to_console(&format!("Error al calcular estadísticas: {}", e));
                                None
                            }
                        };
                        
                        if count > 0 {
                            log_to_console(&format!("Se encontraron {} viajes con destino {} en {:.2} segundos", 
                                                count, destination, elapsed.as_secs_f64()));
                            
                            let message = format!("Se encontraron {} viajes con destino {}.\nGuardado en: {}", 
                                                count, destination, output_path.to_string_lossy());
                            
                            let task_result = BackgroundTaskResult {
                                message,
                                success: true,
                                data: None,
                                stats,
                                popular_destinations: None,
                            };
                            *result_clone.lock().unwrap() = Some(Arc::new(task_result));
                        } else {
                            log_to_console(&format!("No se encontraron viajes con destino {} (búsqueda tomó {:.2} segundos)", 
                                                destination, elapsed.as_secs_f64()));
                            
                            let message = format!("No se encontraron viajes con destino {}", destination);
                            
                            let task_result = BackgroundTaskResult {
                                message,
                                success: false,
                                data: None,
                                stats,
                                popular_destinations: None,
                            };
                            *result_clone.lock().unwrap() = Some(Arc::new(task_result));
                        }
                    },
                    Err(e) => {
                        log_to_console(&format!("Error al filtrar por destino {}: {}", destination, e));
                        
                        let task_result = BackgroundTaskResult {
                            message: format!("Error al filtrar: {}", e),
                            success: false,
                            data: None,
                            stats: None,
                            popular_destinations: None,
                        };
                        *result_clone.lock().unwrap() = Some(Arc::new(task_result));
                    }
                }
            });
            
            // Configurar callback
            let app_result = Arc::new(Mutex::new(None));
            let app_result_clone = Arc::clone(&app_result);
            
            std::thread::spawn(move || {
                loop {
                    if let Some(res) = &*result.lock().unwrap() {
                        *app_result_clone.lock().unwrap() = Some(Arc::clone(res));
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            });
            
            self.bg_task_result = Some(Arc::new(BackgroundTaskResult {
                message: format!("Filtrando viajes con destino {}...", destination_clone),
                success: false,
                data: None,
                stats: None,
                popular_destinations: None,
            }));
        }
    }
    
    // Seleccionar directorio de salida
    fn select_output_dir(&mut self, ui: &mut Ui) {
        if ui.button("📁 Seleccionar Directorio de Salida").clicked() {
            log_to_console("Abriendo diálogo para seleccionar directorio de salida...");
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Seleccionar directorio para resultados")
                .pick_folder() 
            {
                log_to_console(&format!("Directorio de salida seleccionado: {}", path.display()));
                self.output_dir = Some(path);
            } else {
                log_to_console("Selección de directorio de salida cancelada");
            }
        }
        
        if let Some(path) = &self.output_dir {
            ui.horizontal(|ui| {
                ui.label("Directorio de salida:");
                ui.monospace(path.to_string_lossy().to_string());
            });
        }
    }
    
    // Construir tabla hash
    fn build_hash_table(&mut self) {
        if let (Some(csv_path), Some(hash_dir)) = (&self.csv_path, &self.hash_dir) {
            let csv_path_clone = csv_path.clone();
            let hash_dir_clone = hash_dir.clone();
            self.bg_task_running = true;
            
            log_to_console(&format!("Construyendo tabla hash en: {}", hash_dir_clone.display()));
            log_to_console("Este proceso puede tomar tiempo dependiendo del tamaño del archivo...");
            
            let result = Arc::new(Mutex::new(None));
            let result_clone = Arc::clone(&result);
            let show_progress = self.show_console_progress;
            
            thread::spawn(move || {
                let start_time = Instant::now();
                
                if show_progress {
                    println!("Construyendo tabla hash... (puede tomar varios minutos)");
                }
                
                match build_hash_table_from_csv(&csv_path_clone, &hash_dir_clone) {
                    Ok(count) => {
                        let elapsed = start_time.elapsed();
                        log_to_console(&format!("Tabla hash construida con éxito: {} registros en {:.2} segundos", 
                                            count, elapsed.as_secs_f64()));
                        
                        let task_result = BackgroundTaskResult {
                            message: format!("Tabla hash construida con éxito. Se procesaron {} registros.", count),
                            success: true,
                            data: None,
                            stats: None,
                            popular_destinations: None,
                        };
                        *result_clone.lock().unwrap() = Some(Arc::new(task_result));
                    },
                    Err(e) => {
                        log_to_console(&format!("Error al construir tabla hash: {}", e));
                        
                        let task_result = BackgroundTaskResult {
                            message: format!("Error al construir tabla hash: {}", e),
                            success: false,
                            data: None,
                            stats: None,
                            popular_destinations: None,
                        };
                        *result_clone.lock().unwrap() = Some(Arc::new(task_result));
                    }
                }
            });
            
            // Configurar callback
            let app_result = Arc::new(Mutex::new(None));
            let app_result_clone = Arc::clone(&app_result);
            
            std::thread::spawn(move || {
                loop {
                    if let Some(res) = &*result.lock().unwrap() {
                        *app_result_clone.lock().unwrap() = Some(Arc::clone(res));
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            });
            
            self.bg_task_result = Some(Arc::new(BackgroundTaskResult {
                message: "Construyendo tabla hash...".to_string(),
                success: false,
                data: None,
                stats: None,
                popular_destinations: None,
            }));
        }
    }
    
    // Mostrar pantalla de bienvenida
    fn show_welcome_screen(&mut self, ctx: &egui::Context) {
        // Actualizar la fecha y hora antes de mostrar
        self.update_datetime();
        
        egui::Window::new("Bienvenido al Analizador de Viajes")
            .fixed_size([400.0, 300.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("🚕 Analizador de Viajes");
                    ui.add_space(10.0);
                    ui.label("Esta aplicación te permite procesar y analizar grandes conjuntos de datos CSV con un uso mínimo de memoria.");
                    ui.add_space(5.0);
                    ui.label("Desarrollada como parte de la Práctica 1 de Sistemas Operativos.");
                    
                    ui.add_space(20.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("Usuario:");
                        ui.strong(&self.username);
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Fecha y hora:");
                        ui.strong(&self.current_datetime);
                    });
                    
                    ui.add_space(20.0);
                    
                    if ui.button("Comenzar").clicked() {
                        log_to_console("Usuario inició la aplicación desde la pantalla de bienvenida");
                        self.show_welcome = false;
                    }
                });
            });
    }
    
    // Mostrar pestaña principal
    fn show_home_tab(&mut self, ui: &mut Ui) {
        // Actualizar la fecha y hora
        self.update_datetime();
        
        ui.vertical_centered(|ui| {
            ui.heading("🚕 Analizador de Viajes");
            ui.label(format!("Versión {} - {}", APP_VERSION, self.current_datetime));
            ui.label(format!("Usuario: {}", self.username));
        });
        
        ui.add_space(20.0);
        
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Archivo CSV");
                ui.label("Selecciona el archivo CSV que deseas procesar:");
                self.select_csv_file(ui);
            });
            
            ui.separator();
            
            ui.vertical(|ui| {
                ui.heading("Directorio de Salida");
                ui.label("Selecciona dónde guardar los resultados:");
                self.select_output_dir(ui);
            });
        });
        
        ui.add_space(20.0);
        
        ui.collapsing("Acciones Rápidas", |ui| {
            ui.horizontal(|ui| {
                if ui.button("📊 Ver Datos").clicked() {
                    log_to_console("Cambiando a pestaña: Ver Datos");
                    self.current_tab = Tab::DataViewer;
                }
                if ui.button("🔍 Filtros").clicked() {
                    log_to_console("Cambiando a pestaña: Filtros");
                    self.current_tab = Tab::Filters;
                }
                if ui.button("📈 Estadísticas").clicked() {
                    log_to_console("Cambiando a pestaña: Estadísticas");
                    self.current_tab = Tab::Statistics;
                }
            });
        });
        
        ui.add_space(10.0);
        
        if let Some(hash_dir) = &self.hash_dir {
            ui.horizontal(|ui| {
                ui.label("Directorio de tabla hash:");
                ui.monospace(hash_dir.to_string_lossy().to_string());
            });
            
            if let Some(_) = &self.csv_path {
                if ui.button("🔨 Construir Tabla Hash").clicked() && !self.bg_task_running {
                    self.build_hash_table();
                }
            }
        }
        
        // Mostrar mensajes de tareas en segundo plano
        self.show_background_task_status(ui);
        
        // Pie de página
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("Uso de memoria limitado a ");
                ui.strong(MAX_MEMORY_USAGE);
                ui.label(" - ");
                if ui.link("Acerca de").clicked() {
                    log_to_console("Cambiando a pestaña: Acerca de");
                    self.current_tab = Tab::About;
                }
            });
        });
    }
    
    // Mostrar pestaña de visualización de datos
    fn show_data_viewer_tab(&mut self, ui: &mut Ui) {
        ui.heading("📊 Visualizador de Datos");
        
        if self.preview_data.is_empty() {
            ui.label("No hay datos para mostrar. Carga una vista previa desde la pestaña principal.");
            if ui.button("⬅️ Volver").clicked() {
                log_to_console("Volviendo a pestaña: Inicio");
                self.current_tab = Tab::Home;
            }
            return;
        }
        
        ui.label(format!("Mostrando {} registros (limitado para controlar uso de memoria)", 
                       self.preview_data.len()));
        
        // Tabla con scroll
        egui::ScrollArea::both().max_height(400.0).show(ui, |ui| {
            egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(egui_extras::Column::auto().at_least(80.0))  // Índice
                .column(egui_extras::Column::auto().at_least(100.0)) // Origen
                .column(egui_extras::Column::auto().at_least(100.0)) // Destino
                .column(egui_extras::Column::auto().at_least(80.0))  // Distancia
                .column(egui_extras::Column::auto().at_least(80.0))  // Precio
                .header(20.0, |mut header| {
                    header.col(|ui| { ui.strong("Índice"); });
                    header.col(|ui| { ui.strong("Origen"); });
                    header.col(|ui| { ui.strong("Destino"); });
                    header.col(|ui| { ui.strong("Distancia"); });
                    header.col(|ui| { ui.strong("Total ($)"); });
                })
                .body(|mut body| {
                    for trip in &self.preview_data {
                        body.row(20.0, |mut row| {
                            row.col(|ui| { ui.label(&trip.index); });
                            row.col(|ui| { ui.label(&trip.pu_location_id); });
                            row.col(|ui| { ui.label(&trip.do_location_id); });
                            row.col(|ui| { ui.label(&trip.trip_distance); });
                            row.col(|ui| { ui.label(&trip.total_amount); });
                        });
                    }
                });
        });
        
        ui.add_space(10.0);
        if ui.button("⬅️ Volver").clicked() {
            log_to_console("Volviendo a pestaña: Inicio");
            self.current_tab = Tab::Home;
        }
    }
    
    // Mostrar pestaña de filtros
    fn show_filters_tab(&mut self, ui: &mut Ui) {
        ui.heading("🔍 Filtros");
        
        if self.csv_path.is_none() || self.output_dir.is_none() {
            ui.label(RichText::new("⚠️ Debes seleccionar un archivo CSV y un directorio de salida primero").color(Color32::YELLOW));
            if ui.button("⬅️ Volver").clicked() {
                log_to_console("Volviendo a pestaña: Inicio");
                self.current_tab = Tab::Home;
            }
            return;
        }
        
        ui.add_space(10.0);
        
        // Filtro por precio
        ui.collapsing("💰 Filtro por Precio", |ui| {
            ui.horizontal(|ui| {
                ui.label("Precio mínimo: $");
                ui.text_edit_singleline(&mut self.filter_price_min);
                ui.label("Precio máximo: $");
                ui.text_edit_singleline(&mut self.filter_price_max);
            });
            
            if ui.button("Aplicar Filtro de Precio").clicked() && !self.bg_task_running {
                self.run_price_filter();
            }
        });
        
        // Filtro por índice
        ui.collapsing("🔢 Filtro por Índice", |ui| {
            ui.horizontal(|ui| {
                ui.label("Índice: ");
                ui.text_edit_singleline(&mut self.filter_index);
            });
            
            if ui.button("Buscar por Índice").clicked() && !self.bg_task_running {
                self.run_index_filter();
            }
        });
        
        // Filtro por destino
        ui.collapsing("📍 Filtro por Destino", |ui| {
            ui.horizontal(|ui| {
                ui.label("ID de Destino: ");
                ui.text_edit_singleline(&mut self.filter_destination);
            });
            
            if ui.button("Filtrar por Destino").clicked() && !self.bg_task_running {
                self.run_destination_filter();
            }
        });
        
        ui.add_space(20.0);
        if ui.button("⬅️ Volver").clicked() {
            log_to_console("Volviendo a pestaña: Inicio");
            self.current_tab = Tab::Home;
        }
        
        // Mostrar mensajes de tareas en segundo plano
        self.show_background_task_status(ui);
    }
    
    // Mostrar pestaña de estadísticas
    fn show_statistics_tab(&mut self, ui: &mut Ui) {
        ui.heading("📈 Estadísticas");
        
        if self.csv_path.is_none() {
            ui.label(RichText::new("⚠️ Debes seleccionar un archivo CSV primero").color(Color32::YELLOW));
            if ui.button("⬅️ Volver").clicked() {
                log_to_console("Volviendo a pestaña: Inicio");
                self.current_tab = Tab::Home;
            }
            return;
        }
        
        ui.add_space(10.0);
        
        // Estadísticas generales de la vista previa
        if !self.preview_data.is_empty() {
            ui.collapsing("📊 Estadísticas de Vista Previa", |ui| {
                let mut total_distance = 0.0;
                let mut total_amount = 0.0;
                
                for trip in &self.preview_data {
                    total_distance += trip.trip_distance.parse::<f64>().unwrap_or(0.0);
                    total_amount += trip.total_amount.parse::<f64>().unwrap_or(0.0);
                }
                
                let avg_distance = total_distance / self.preview_data.len() as f64;
                let avg_amount = total_amount / self.preview_data.len() as f64;
                
                ui.label(format!("Registros: {}", self.preview_data.len()));
                ui.label(format!("Distancia promedio: {:.2} millas", avg_distance));
                ui.label(format!("Monto promedio: ${:.2}", avg_amount));
            });
        }
        
        // Destinos más populares
        ui.collapsing("🏙️ Destinos Populares", |ui| {
            if ui.button("Calcular Destinos Más Populares").clicked() && !self.bg_task_running {
                self.calculate_popular_destinations();
            }
        });
        
        // Estadísticas de filtros aplicados
        if let Some(result) = &self.bg_task_result {
            if let Some(stats) = &result.stats {
                ui.collapsing("📝 Estadísticas de Filtro", |ui| {
                    if let Some(count) = stats.get("count") {
                        ui.label(format!("Total de viajes: {}", count));
                    }
                    
                    if let Some(avg_distance) = stats.get("avg_distance") {
                        ui.label(format!("Distancia promedio: {:.2} millas", avg_distance));
                    }
                    
                    if let Some(avg_amount) = stats.get("avg_amount") {
                        ui.label(format!("Precio promedio: ${:.2}", avg_amount));
                    }
                    
                    if let Some(avg_passengers) = stats.get("avg_passengers") {
                        ui.label(format!("Pasajeros promedio: {:.2}", avg_passengers));
                    }
                });
            }
        }
        
        ui.add_space(20.0);
        if ui.button("⬅️ Volver").clicked() {
            log_to_console("Volviendo a pestaña: Inicio");
            self.current_tab = Tab::Home;
        }
        
        // Mostrar mensajes de tareas en segundo plano
        self.show_background_task_status(ui);
    }
    
    // Calcular destinos populares
    fn calculate_popular_destinations(&mut self) {
        if let Some(csv_path) = &self.csv_path {
            let csv_path_clone = csv_path.clone();
            self.bg_task_running = true;
            
            log_to_console("Calculando destinos más populares...");
            
            let result = Arc::new(Mutex::new(None));
            let result_clone = Arc::clone(&result);
            let show_progress = self.show_console_progress;
            
            thread::spawn(move || {
                let start_time = Instant::now();
                
                // Calcular destinos populares
                match get_popular_destinations(&csv_path_clone, 10) {
                    Ok(popular_dests) => {
                        let elapsed = start_time.elapsed();
                        log_to_console(&format!("Destinos populares calculados en {:.2} segundos", elapsed.as_secs_f64()));
                        
                        let mut message = String::from("Destinos más populares:\n");
                        
                        if show_progress {
                            println!("Top 10 destinos más populares:");
                            for (i, (dest, count)) in popular_dests.iter().enumerate() {
                                println!("{}. Destino ID: {} - {} viajes", i+1, dest, count);
                            }
                        }
                        
                        for (i, (dest, count)) in popular_dests.iter().enumerate() {
                            message.push_str(&format!("{}. Destino ID: {} - {} viajes\n", i+1, dest, count));
                        }
                        
                        let task_result = BackgroundTaskResult {
                            message,
                            success: true,
                            data: None,
                            stats: None,
                            popular_destinations: Some(popular_dests),
                        };
                        *result_clone.lock().unwrap() = Some(Arc::new(task_result));
                    },
                    Err(e) => {
                        log_to_console(&format!("Error al calcular destinos populares: {}", e));
                        
                        let task_result = BackgroundTaskResult {
                            message: format!("Error al calcular destinos populares: {}", e),
                            success: false,
                            data: None,
                            stats: None,
                            popular_destinations: None,
                        };
                        *result_clone.lock().unwrap() = Some(Arc::new(task_result));
                    }
                }
            });
            
            // Configurar callback
            let app_result = Arc::new(Mutex::new(None));
            let app_result_clone = Arc::clone(&app_result);
            
            std::thread::spawn(move || {
                loop {
                    if let Some(res) = &*result.lock().unwrap() {
                        *app_result_clone.lock().unwrap() = Some(Arc::clone(res));
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            });
            
            self.bg_task_result = Some(Arc::new(BackgroundTaskResult {
                message: "Calculando destinos más populares...".to_string(),
                success: false,
                data: None,
                stats: None,
                popular_destinations: None,
            }));
        }
    }
    
    // Mostrar pestaña de configuración
    fn show_settings_tab(&mut self, ui: &mut Ui) {
        ui.heading("⚙️ Configuración");
        
        ui.add_space(10.0);
        
        // Tema de la interfaz
        ui.horizontal(|ui| {
            ui.label("Tema:");
            if ui.selectable_label(self.theme_dark, "🌙 Oscuro").clicked() {
                log_to_console("Cambiando tema a: Oscuro");
                self.theme_dark = true;
                let mut visuals = ui.ctx().style().visuals.clone();
                visuals.dark_mode = true;
                ui.ctx().set_visuals(visuals);
            }
            if ui.selectable_label(!self.theme_dark, "☀️ Claro").clicked() {
                log_to_console("Cambiando tema a: Claro");
                self.theme_dark = false;
                let mut visuals = ui.ctx().style().visuals.clone();
                visuals.dark_mode = false;
                ui.ctx().set_visuals(visuals);
            }
        });
        
        // Configuración de vista previa
        ui.horizontal(|ui| {
            ui.label("Máximo de filas en vista previa:");
            let old_value = self.max_preview_rows;
            ui.add(egui::Slider::new(&mut self.max_preview_rows, 10..=500)
                .text("filas"));
            
            if old_value != self.max_preview_rows {
                log_to_console(&format!("Máximo de filas en vista previa cambiado a: {}", self.max_preview_rows));
            }
        });
        
        // Configuración de consola
        ui.checkbox(&mut self.show_console_progress, "Mostrar progreso detallado en consola");
        if ui.button("Limpiar mensajes de consola").clicked() {
            // En sistemas basados en UNIX/Linux esto funciona
            print!("\x1B[2J\x1B[1;1H");
            log_to_console("Consola limpiada");
        }
        
        // Directorio de tabla hash
        ui.collapsing("🔄 Configuración de Tabla Hash", |ui| {
            if let Some(hash_dir) = &self.hash_dir {
                ui.horizontal(|ui| {
                    ui.label("Directorio de tabla hash actual:");
                    ui.monospace(hash_dir.to_string_lossy().to_string());
                });
            } else {
                ui.label("No hay directorio de tabla hash configurado.");
            }
            
            if ui.button("Cambiar Directorio para Tabla Hash").clicked() {
                log_to_console("Abriendo diálogo para cambiar directorio de tabla hash...");
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Seleccionar directorio para tabla hash")
                    .pick_folder() 
                {
                    log_to_console(&format!("Nuevo directorio de tabla hash: {}", path.display()));
                    self.hash_dir = Some(path);
                } else {
                    log_to_console("Selección de directorio de tabla hash cancelada");
                }
            }
            
            if ui.button("Restaurar Directorio Predeterminado").clicked() {
                log_to_console("Restaurando directorio de tabla hash a ./tmp");
                self.hash_dir = Some(ensure_tmp_dir_exists());
            }
        });
        
        ui.add_space(10.0);
        
        // Información de memoria
        ui.collapsing("📝 Información de Memoria", |ui| {
            ui.label("Esta aplicación está optimizada para mantener el uso de memoria por debajo de 10MB.");
            ui.label("Estrategias de optimización:");
            ui.label("• Procesamiento por streaming (sin cargar el CSV completo)");
            ui.label("• Vista previa limitada de datos");
            ui.label("• Operaciones en segundo plano");
            ui.label("• Tabla hash basada en disco");
        });
        
        ui.add_space(20.0);
        if ui.button("⬅️ Volver").clicked() {
            log_to_console("Volviendo a pestaña: Inicio");
            self.current_tab = Tab::Home;
        }
    }
    
    // Mostrar pestaña de acerca de
    fn show_about_tab(&mut self, ui: &mut Ui) {
        // Actualizar la fecha y hora
        self.update_datetime();
        
        ui.vertical_centered(|ui| {
            ui.heading("ℹ️ Acerca de");
            
            ui.add_space(20.0);
            
            ui.label(RichText::new("Analizador de Viajes").size(24.0));
            ui.label(format!("Versión {}", APP_VERSION));
            
            ui.add_space(10.0);
            
            ui.label("Desarrollado por Juan Loaiza (@juanloaiza21)");
            ui.label(format!("Fecha: {}", self.current_datetime));
            
            ui.add_space(20.0);
            
            ui.label("Esta aplicación permite procesar y analizar grandes conjuntos de datos CSV manteniendo un uso de memoria por debajo de 10MB.");
            
            ui.add_space(10.0);
            
            ui.label("Características principales:");
            ui.label("• Procesamiento de archivos CSV grandes (3GB+)");
            ui.label("• Filtrado por precio, índice y destino");
            ui.label("• Estadísticas de viajes");
            ui.label("• Tabla hash basada en disco para búsquedas rápidas");
            ui.label("• Interfaz gráfica amigable");
            
            ui.add_space(20.0);
            ui.label("Universidad del Valle - Sistemas Operativos");
            ui.label("Práctica 1 - 2025");
        });
        
        ui.add_space(20.0);
        if ui.button("⬅️ Volver").clicked() {
            log_to_console("Volviendo a pestaña: Inicio");
            self.current_tab = Tab::Home;
        }
    }
    
    // Mostrar estado de tareas en segundo plano
    fn show_background_task_status(&mut self, ui: &mut Ui) {
        if self.bg_task_running {
            // Verificar si hay resultados
            let result_ready = if let Some(result) = &self.bg_task_result {
                result.data.is_some() || result.message.contains("completado") || result.message.contains("encontrado") || result.message.contains("Error")
            } else {
                false
            };
            
            if result_ready {
                self.bg_task_running = false;
            }
            
            // Mostrar mensaje de progreso o resultado
            if let Some(result) = &self.bg_task_result {
                ui.add_space(10.0);
                ui.separator();
                
                if self.bg_task_running {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(&result.message);
                    });
                } else {
                    let text_color = if result.success {
                        Color32::GREEN
                    } else {
                        Color32::RED
                    };
                    
                    ui.label(RichText::new(&result.message).color(text_color));
                    
                    // Si hay datos de vista previa, actualizarlos
                    if let Some(data) = &result.data {
                        self.preview_data = data.clone();
                    }
                }
            }
        }
    }
}

impl eframe::App for TripsVisualApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Mostrar pantalla de bienvenida si es necesario
        if self.show_welcome {
            self.show_welcome_screen(ctx);
            return;
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            // Barra de navegación superior
            ui.horizontal(|ui| {
                if ui.selectable_label(matches!(self.current_tab, Tab::Home), "🏠 Inicio").clicked() {
                    log_to_console("Cambiando a pestaña: Inicio");
                    self.current_tab = Tab::Home;
                }
                if ui.selectable_label(matches!(self.current_tab, Tab::DataViewer), "📊 Ver Datos").clicked() {
                    log_to_console("Cambiando a pestaña: Ver Datos");
                    self.current_tab = Tab::DataViewer;
                }
                if ui.selectable_label(matches!(self.current_tab, Tab::Filters), "🔍 Filtros").clicked() {
                    log_to_console("Cambiando a pestaña: Filtros");
                    self.current_tab = Tab::Filters;
                }
                if ui.selectable_label(matches!(self.current_tab, Tab::Statistics), "📈 Estadísticas").clicked() {
                    log_to_console("Cambiando a pestaña: Estadísticas");
                    self.current_tab = Tab::Statistics;
                }
                if ui.selectable_label(matches!(self.current_tab, Tab::Settings), "⚙️ Config").clicked() {
                    log_to_console("Cambiando a pestaña: Configuración");
                    self.current_tab = Tab::Settings;
                }
                if ui.selectable_label(matches!(self.current_tab, Tab::About), "ℹ️ Acerca de").clicked() {
                    log_to_console("Cambiando a pestaña: Acerca de");
                    self.current_tab = Tab::About;
                }
            });
            
            ui.separator();
            
            // Contenido según la pestaña seleccionada
            match self.current_tab {
                Tab::Home => self.show_home_tab(ui),
                Tab::DataViewer => self.show_data_viewer_tab(ui),
                Tab::Filters => self.show_filters_tab(ui),
                Tab::Statistics => self.show_statistics_tab(ui),
                Tab::Settings => self.show_settings_tab(ui),
                Tab::About => self.show_about_tab(ui),
            }
        });
    }
}

// Función para iniciar la aplicación
pub fn run_app() -> Result<(), eframe::Error> {
    // Asegurar que el directorio tmp existe
    ensure_tmp_dir_exists();
    
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║               ANALIZADOR DE VIAJES - INICIO                   ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║ Usuario: {:<56} ║", CURRENT_USER);
    println!("║ Fecha:   {:<56} ║", CURRENT_DATE);
    println!("║ Versión: {:<56} ║", APP_VERSION);
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║ Iniciando aplicación - Registro de operaciones                ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])  // Ventana más grande (1024x768)
            .with_min_inner_size([800.0, 600.0]),  // Tamaño mínimo mayor
        ..Default::default()
    };
    
    eframe::run_native(
        "Analizador de Viajes - juanloaiza21",
        options,
        Box::new(|cc| Ok(Box::new(TripsVisualApp::new(cc))))
    )
}
