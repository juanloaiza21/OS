use crate::data::filters::{self, TripFilter};
use crate::data::trip_struct::Trip;
use eframe::{self, egui};
use egui_extras::{Column, TableBuilder};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

// Rutas corregidas
const CSV_PATH: &str = "src/data/data.csv";
const TMP_DIR: &str = "tmp";
const MAX_DISPLAYED_ROWS: usize = 1000; // Para limitar la cantidad de filas mostradas a la vez

// Estructura para compartir datos entre hilos
#[derive(Default)]
struct FilterState {
    filtered_results: Vec<Trip>,
    results_count: usize,
    is_filtering: bool,
    filter_error: Option<String>,
    stats: Option<std::collections::HashMap<String, f64>>,
    popular_destinations: Option<Vec<(String, usize)>>,
    export_status: Option<String>,
    // Nuevos campos para seguimiento de tareas completadas
    statistics_loaded: bool,
    destinations_loaded: bool,
    should_switch_tab: Option<Tab>,
    // Nuevo: Campos para paginación
    current_page: usize,
    total_pages: usize,
    // Campo para almacenar el archivo temporal activo
    temp_file: Option<String>,
}

struct FilterApp {
    // Estado de los filtros
    min_price: String,
    max_price: String,
    index_filter: String,
    destination_filter: String,
    use_and: bool,

    // Estado compartido entre hilos
    state: Arc<Mutex<FilterState>>,

    // Estado para la visualización
    selected_tab: Tab,

    // Estado para la exportación
    export_filename: String,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Tab {
    Data,
    Stats,
    PopularDestinations,
}

impl Default for Tab {
    fn default() -> Self {
        Tab::Data
    }
}

impl Default for FilterApp {
    fn default() -> Self {
        // Crear el directorio tmp si no existe
        println!("Inicializando aplicación de visualización de datos...");

        if !Path::new(TMP_DIR).exists() {
            println!("Creando directorio temporal: {}", TMP_DIR);
            fs::create_dir_all(TMP_DIR).expect("No se pudo crear el directorio tmp");
        }

        // Verificar si el archivo CSV existe
        println!("Verificando archivo de datos: {}", CSV_PATH);
        if !Path::new(CSV_PATH).exists() {
            println!(
                "ADVERTENCIA: No se encontró el archivo de datos en {}",
                CSV_PATH
            );
        } else {
            println!("Archivo de datos encontrado correctamente");
        }

        let app = Self {
            min_price: String::new(),
            max_price: String::new(),
            index_filter: String::new(),
            destination_filter: String::new(),
            use_and: true,
            state: Arc::new(Mutex::new(FilterState::default())),
            selected_tab: Tab::default(),
            export_filename: "filtered_data.csv".to_string(),
        };

        // Realizar una carga inicial de datos
        println!("Iniciando carga inicial de datos...");
        let state_clone = Arc::clone(&app.state);

        thread::spawn(move || {
            // Crear un filtro que incluya todos los datos (sin restricciones)
            let filter = TripFilter::Price {
                min: None,
                max: None,
            };
            let tmp_file = format!("{}/initial_data.csv", TMP_DIR);

            println!("Aplicando filtro inicial para cargar datos...");
            match filters::filter_to_file(CSV_PATH, &tmp_file, filter, None) {
                // Sin límite de resultados
                Ok(count) => {
                    println!("Filtro aplicado. Total de registros encontrados: {}", count);

                    // Cargar los datos filtrados
                    if let Ok(file) = std::fs::File::open(&tmp_file) {
                        println!(
                            "Cargando datos en la interfaz (primeros {} registros)...",
                            MAX_DISPLAYED_ROWS
                        );
                        let reader = std::io::BufReader::new(file);
                        let mut csv_reader = csv::ReaderBuilder::new()
                            .has_headers(true)
                            .from_reader(reader);

                        let mut trips = Vec::with_capacity(MAX_DISPLAYED_ROWS);
                        for (i, result) in csv_reader.records().take(MAX_DISPLAYED_ROWS).enumerate()
                        {
                            if i % 100 == 0 {
                                println!("Procesados {} registros...", i);
                            }

                            if let Ok(record) = result {
                                if record.len() >= 19 {
                                    trips.push(Trip {
                                        vendor_id: record[0].to_string(),
                                        tpep_pickup_datetime: record[1].to_string(),
                                        tpep_dropoff_datetime: record[2].to_string(),
                                        passenger_count: record[3].to_string(),
                                        trip_distance: record[4].to_string(),
                                        ratecode_id: record[5].to_string(),
                                        store_and_fwd_flag: record[6].to_string(),
                                        pu_location_id: record[7].to_string(),
                                        do_location_id: record[8].to_string(),
                                        payment_type: record[9].to_string(),
                                        fare_amount: record[10].to_string(),
                                        extra: record[11].to_string(),
                                        mta_tax: record[12].to_string(),
                                        tip_amount: record[13].to_string(),
                                        tolls_amount: record[14].to_string(),
                                        improvement_surcharge: record[15].to_string(),
                                        total_amount: record[16].to_string(),
                                        congestion_surcharge: record[17].to_string(),
                                        index: record[18].to_string(),
                                    });
                                }
                            }
                        }

                        println!("Cargados {} registros en la interfaz", trips.len());

                        // Actualizar los resultados
                        let mut state = state_clone.lock().unwrap();
                        state.filtered_results = trips;
                        state.results_count = count;
                        state.is_filtering = false;
                        // Inicializar paginación
                        state.current_page = 0;
                        state.total_pages = (count + MAX_DISPLAYED_ROWS - 1) / MAX_DISPLAYED_ROWS;
                        state.temp_file = Some(tmp_file);

                        println!(
                            "Paginación configurada: {} páginas totales",
                            state.total_pages
                        );
                    } else {
                        println!("ERROR: No se pudo abrir el archivo temporal de resultados");
                        let mut state = state_clone.lock().unwrap();
                        state.filter_error =
                            Some("No se pudo abrir el archivo de resultados".to_string());
                        state.is_filtering = false;
                    }
                }
                Err(e) => {
                    println!("ERROR al aplicar el filtro inicial: {}", e);
                    let mut state = state_clone.lock().unwrap();
                    state.filter_error = Some(format!("Error al filtrar: {}", e));
                    state.is_filtering = false;
                }
            }
        });

        app
    }
}

impl FilterApp {
    // Método para cargar todos los datos de una vez
    fn load_all(&self) {
        // Verificar si ya está filtrando
        {
            let mut state = self.state.lock().unwrap();
            if state.is_filtering {
                println!(
                    "\n[CARGA TOTAL] Ya hay un proceso en curso, ignorando solicitud de carga completa"
                );
                return;
            }
            state.is_filtering = true;
            state.filter_error = None;
        }

        println!("\n[CARGA TOTAL] ===== INICIANDO CARGA COMPLETA =====");
        println!("[CARGA TOTAL] Etapa 1/3: Cargando datos filtrados...");

        // Almacenamos los datos de filtro que necesitaremos recrear en cada etapa
        let min_price = self.min_price.parse::<f64>().ok();
        let max_price = self.max_price.parse::<f64>().ok();
        let index_filter = self.index_filter.clone();
        let destination_filter = self.destination_filter.clone();
        let use_and = self.use_and;

        let state_clone = Arc::clone(&self.state);

        // Creamos un hilo principal para gestionar la carga secuencial
        thread::spawn(move || {
            // Crear filtro para la etapa 1
            let filter = create_filter(
                min_price,
                max_price,
                &index_filter,
                &destination_filter,
                use_and,
            );

            // ETAPA 1: Carga de datos filtrados
            let tmp_file = format!("{}/load_all_data.csv", TMP_DIR);
            println!("[CARGA TOTAL] Aplicando filtros a los datos...");

            match filters::filter_to_file(CSV_PATH, &tmp_file, filter, None) {
                // Sin límite para guardar todos los datos
                Ok(count) => {
                    println!(
                        "[CARGA TOTAL] ✓ Filtro aplicado exitosamente. Total de registros: {}",
                        count
                    );

                    if let Ok(file) = std::fs::File::open(&tmp_file) {
                        println!(
                            "[CARGA TOTAL] Cargando datos en memoria (primeros {} registros)...",
                            MAX_DISPLAYED_ROWS
                        );
                        let reader = std::io::BufReader::new(file);
                        let mut csv_reader = csv::ReaderBuilder::new()
                            .has_headers(true)
                            .from_reader(reader);

                        let mut trips = Vec::with_capacity(MAX_DISPLAYED_ROWS);
                        let mut processed = 0;
                        for result in csv_reader.records().take(MAX_DISPLAYED_ROWS) {
                            processed += 1;
                            if processed % 100 == 0 {
                                println!("[CARGA TOTAL] Procesados {} registros...", processed);
                            }

                            if let Ok(record) = result {
                                if record.len() >= 19 {
                                    trips.push(Trip {
                                        vendor_id: record[0].to_string(),
                                        tpep_pickup_datetime: record[1].to_string(),
                                        tpep_dropoff_datetime: record[2].to_string(),
                                        passenger_count: record[3].to_string(),
                                        trip_distance: record[4].to_string(),
                                        ratecode_id: record[5].to_string(),
                                        store_and_fwd_flag: record[6].to_string(),
                                        pu_location_id: record[7].to_string(),
                                        do_location_id: record[8].to_string(),
                                        payment_type: record[9].to_string(),
                                        fare_amount: record[10].to_string(),
                                        extra: record[11].to_string(),
                                        mta_tax: record[12].to_string(),
                                        tip_amount: record[13].to_string(),
                                        tolls_amount: record[14].to_string(),
                                        improvement_surcharge: record[15].to_string(),
                                        total_amount: record[16].to_string(),
                                        congestion_surcharge: record[17].to_string(),
                                        index: record[18].to_string(),
                                    });
                                }
                            }
                        }

                        println!(
                            "[CARGA TOTAL] ✓ Etapa 1/3 completada: {} registros cargados en memoria",
                            trips.len()
                        );

                        // Actualizar los resultados
                        {
                            let mut state = state_clone.lock().unwrap();
                            state.filtered_results = trips;
                            state.results_count = count;
                            state.current_page = 0;
                            state.total_pages =
                                (count + MAX_DISPLAYED_ROWS - 1) / MAX_DISPLAYED_ROWS;
                            state.temp_file = Some(tmp_file.clone());
                            println!(
                                "[CARGA TOTAL] Paginación configurada: {} páginas totales",
                                state.total_pages
                            );
                        }

                        // ETAPA 2: Cálculo de estadísticas
                        println!("\n[CARGA TOTAL] Etapa 2/3: Calculando estadísticas...");

                        // Crear un nuevo filtro para estadísticas
                        let stats_filter = create_filter(
                            min_price,
                            max_price,
                            &index_filter,
                            &destination_filter,
                            use_and,
                        );

                        match filters::get_filter_stats(CSV_PATH, stats_filter) {
                            Ok(stats) => {
                                println!("[CARGA TOTAL] ✓ Estadísticas calculadas correctamente:");
                                println!(
                                    "[CARGA TOTAL]   - Total registros: {}",
                                    stats.get("count").unwrap_or(&0.0)
                                );
                                if let Some(count) = stats.get("count") {
                                    if *count > 0.0 {
                                        println!(
                                            "[CARGA TOTAL]   - Distancia promedio: {:.2}",
                                            stats.get("avg_distance").unwrap_or(&0.0)
                                        );
                                        println!(
                                            "[CARGA TOTAL]   - Precio promedio: ${:.2}",
                                            stats.get("avg_amount").unwrap_or(&0.0)
                                        );
                                        println!(
                                            "[CARGA TOTAL]   - Pasajeros promedio: {:.1}",
                                            stats.get("avg_passengers").unwrap_or(&0.0)
                                        );
                                        println!(
                                            "[CARGA TOTAL]   - Monto total: ${:.2}",
                                            stats.get("total_amount").unwrap_or(&0.0)
                                        );
                                    }
                                }

                                {
                                    let mut state = state_clone.lock().unwrap();
                                    state.stats = Some(stats);
                                    state.statistics_loaded = true;
                                }
                                println!(
                                    "[CARGA TOTAL] ✓ Etapa 2/3 completada: Estadísticas generadas"
                                );

                                // ETAPA 3: Destinos populares
                                println!(
                                    "\n[CARGA TOTAL] Etapa 3/3: Obteniendo destinos populares..."
                                );

                                match filters::get_popular_destinations(CSV_PATH, 20) {
                                    Ok(destinations) => {
                                        println!(
                                            "[CARGA TOTAL] ✓ Se encontraron {} destinos populares",
                                            destinations.len()
                                        );
                                        for (i, (dest, count)) in
                                            destinations.iter().enumerate().take(5)
                                        {
                                            println!(
                                                "[CARGA TOTAL]   {}. Destino {}: {} viajes",
                                                i + 1,
                                                dest,
                                                count
                                            );
                                        }
                                        if destinations.len() > 5 {
                                            println!(
                                                "[CARGA TOTAL]   ... y {} destinos más",
                                                destinations.len() - 5
                                            );
                                        }

                                        {
                                            let mut state = state_clone.lock().unwrap();
                                            state.popular_destinations = Some(destinations);
                                            state.destinations_loaded = true;
                                            state.is_filtering = false;
                                        }
                                        println!(
                                            "[CARGA TOTAL] ✓ Etapa 3/3 completada: Destinos populares obtenidos"
                                        );
                                        println!(
                                            "[CARGA TOTAL] ===== CARGA COMPLETA FINALIZADA CON ÉXITO =====\n"
                                        );
                                    }
                                    Err(e) => {
                                        println!(
                                            "[CARGA TOTAL] ✗ ERROR en etapa 3/3: No se pudieron obtener destinos populares: {}",
                                            e
                                        );
                                        let mut state = state_clone.lock().unwrap();
                                        state.filter_error =
                                            Some(format!("Error al obtener destinos: {}", e));
                                        state.is_filtering = false;
                                        println!(
                                            "[CARGA TOTAL] === CARGA COMPLETA FINALIZADA CON ERRORES ===\n"
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                println!(
                                    "[CARGA TOTAL] ✗ ERROR en etapa 2/3: No se pudieron calcular estadísticas: {}",
                                    e
                                );
                                let mut state = state_clone.lock().unwrap();
                                state.filter_error =
                                    Some(format!("Error al obtener estadísticas: {}", e));
                                state.is_filtering = false;
                                println!(
                                    "[CARGA TOTAL] === CARGA COMPLETA FINALIZADA CON ERRORES ===\n"
                                );
                            }
                        }
                    } else {
                        println!(
                            "[CARGA TOTAL] ✗ ERROR en etapa 1/3: No se pudo abrir el archivo temporal"
                        );
                        let mut state = state_clone.lock().unwrap();
                        state.filter_error =
                            Some("No se pudo abrir el archivo de resultados".to_string());
                        state.is_filtering = false;
                        println!("[CARGA TOTAL] === CARGA COMPLETA FINALIZADA CON ERRORES ===\n");
                    }
                }
                Err(e) => {
                    println!(
                        "[CARGA TOTAL] ✗ ERROR en etapa 1/3: No se pudo aplicar el filtro: {}",
                        e
                    );
                    let mut state = state_clone.lock().unwrap();
                    state.filter_error = Some(format!("Error al filtrar: {}", e));
                    state.is_filtering = false;
                    println!("[CARGA TOTAL] === CARGA COMPLETA FINALIZADA CON ERRORES ===\n");
                }
            }
        });
    }

    // Función auxiliar para construir filtros
    fn build_filter(&self) -> TripFilter {
        println!("Construyendo filtro con parámetros:");
        println!("  - Precio mínimo: {}", self.min_price);
        println!("  - Precio máximo: {}", self.max_price);
        println!("  - Índice: {}", self.index_filter);
        println!("  - Destino: {}", self.destination_filter);
        println!("  - Operador: {}", if self.use_and { "AND" } else { "OR" });

        let min_price = self.min_price.parse::<f64>().ok();
        let max_price = self.max_price.parse::<f64>().ok();

        create_filter(
            min_price,
            max_price,
            &self.index_filter,
            &self.destination_filter,
            self.use_and,
        )
    }

    fn apply_filter(&self) {
        self.apply_filter_internal(Some(Tab::Data));
    }

    fn apply_filter_internal(&self, target_tab: Option<Tab>) {
        // Verificar si ya está filtrando
        {
            let mut state = self.state.lock().unwrap();
            if state.is_filtering {
                println!("Ya hay un proceso de filtrado en curso, ignorando solicitud");
                return;
            }
            state.is_filtering = true;
            state.filter_error = None;
            if let Some(tab) = target_tab {
                state.should_switch_tab = Some(tab);
            }

            // Limpiar el archivo temporal anterior si existe
            if let Some(old_file) = &state.temp_file {
                if Path::new(old_file).exists() {
                    let _ = std::fs::remove_file(old_file);
                    println!("Eliminado archivo temporal anterior: {}", old_file);
                }
            }
        }

        let filter = self.build_filter();
        let state_clone = Arc::clone(&self.state);

        // Ejecutar el filtrado en un hilo separado para no bloquear la UI
        thread::spawn(move || {
            // Crear un archivo temporal para los resultados
            let tmp_file = format!("{}/temp_filter_results.csv", TMP_DIR);

            println!(
                "Aplicando filtro, resultados se guardarán en: {}",
                &tmp_file
            );

            // Aplicar el filtrado y guardar a archivo - Sin límite para guardar todos
            match filters::filter_to_file(CSV_PATH, &tmp_file, filter, None) {
                Ok(count) => {
                    println!("Filtrado completado. Encontrados {} registros", count);

                    // Cargar los primeros N registros para mostrar
                    if let Ok(file) = std::fs::File::open(&tmp_file) {
                        println!("Cargando datos filtrados para visualización...");
                        let reader = std::io::BufReader::new(file);
                        let mut csv_reader = csv::ReaderBuilder::new()
                            .has_headers(true)
                            .from_reader(reader);

                        let mut trips = Vec::with_capacity(MAX_DISPLAYED_ROWS);
                        for (i, result) in csv_reader.records().take(MAX_DISPLAYED_ROWS).enumerate()
                        {
                            if i % 100 == 0 && i > 0 {
                                println!("Procesados {} registros...", i);
                            }

                            if let Ok(record) = result {
                                if record.len() >= 19 {
                                    trips.push(Trip {
                                        vendor_id: record[0].to_string(),
                                        tpep_pickup_datetime: record[1].to_string(),
                                        tpep_dropoff_datetime: record[2].to_string(),
                                        passenger_count: record[3].to_string(),
                                        trip_distance: record[4].to_string(),
                                        ratecode_id: record[5].to_string(),
                                        store_and_fwd_flag: record[6].to_string(),
                                        pu_location_id: record[7].to_string(),
                                        do_location_id: record[8].to_string(),
                                        payment_type: record[9].to_string(),
                                        fare_amount: record[10].to_string(),
                                        extra: record[11].to_string(),
                                        mta_tax: record[12].to_string(),
                                        tip_amount: record[13].to_string(),
                                        tolls_amount: record[14].to_string(),
                                        improvement_surcharge: record[15].to_string(),
                                        total_amount: record[16].to_string(),
                                        congestion_surcharge: record[17].to_string(),
                                        index: record[18].to_string(),
                                    });
                                }
                            }
                        }

                        println!("Se cargarán {} registros en la interfaz", trips.len());

                        // Actualizar los resultados
                        let mut state = state_clone.lock().unwrap();
                        state.filtered_results = trips;
                        state.results_count = count;
                        state.is_filtering = false;
                        state.current_page = 0;
                        state.total_pages = (count + MAX_DISPLAYED_ROWS - 1) / MAX_DISPLAYED_ROWS;
                        state.temp_file = Some(tmp_file);

                        println!(
                            "Paginación configurada: {} páginas totales",
                            state.total_pages
                        );
                    } else {
                        println!("ERROR: No se pudo abrir el archivo temporal de resultados");
                        let mut state = state_clone.lock().unwrap();
                        state.filter_error =
                            Some("No se pudo abrir el archivo de resultados".to_string());
                        state.is_filtering = false;
                    }
                }
                Err(e) => {
                    println!("ERROR al aplicar filtro: {}", e);
                    let mut state = state_clone.lock().unwrap();
                    state.filter_error = Some(format!("Error al filtrar: {}", e));
                    state.is_filtering = false;
                }
            }
        });
    }

    // Nueva función para cargar una página específica de datos
    fn load_page(&self, page: usize) {
        // Verificar si ya está filtrando
        let temp_file = {
            let mut state = self.state.lock().unwrap();
            if state.is_filtering {
                println!("Ya hay un proceso en curso, ignorando solicitud de cambio de página");
                return;
            }

            if page >= state.total_pages {
                println!(
                    "Número de página inválido: {} (máximo: {})",
                    page,
                    state.total_pages - 1
                );
                return;
            }

            if state.current_page == page {
                println!("La página solicitada ya está cargada");
                return;
            }

            state.is_filtering = true;

            match &state.temp_file {
                Some(file) => file.clone(),
                None => {
                    state.is_filtering = false;
                    state.filter_error =
                        Some("No hay archivo de resultados disponible".to_string());
                    return;
                }
            }
        };

        let state_clone = Arc::clone(&self.state);

        thread::spawn(move || {
            println!("Cargando página {} de resultados...", page);

            if let Ok(file) = std::fs::File::open(&temp_file) {
                let reader = std::io::BufReader::new(file);
                let mut csv_reader = csv::ReaderBuilder::new()
                    .has_headers(true)
                    .from_reader(reader);

                // Saltarse las filas anteriores a la página actual
                let start_index = page * MAX_DISPLAYED_ROWS;
                let mut current_idx = 0;

                // Saltar registros anteriores a la página actual
                println!(
                    "Saltando {} registros para llegar a la página {}...",
                    start_index, page
                );
                for _ in 0..start_index {
                    if let Err(_) = csv_reader.records().next().transpose() {
                        break; // Final del archivo o error
                    }
                    current_idx += 1;

                    if current_idx % 1000 == 0 {
                        println!("Saltados {} registros...", current_idx);
                    }
                }

                // Leer los registros de la página actual
                println!("Leyendo registros para la página {}...", page);
                let mut trips = Vec::with_capacity(MAX_DISPLAYED_ROWS);
                let mut page_idx = 0;

                for result in csv_reader.records().take(MAX_DISPLAYED_ROWS) {
                    page_idx += 1;

                    if page_idx % 100 == 0 {
                        println!("Procesados {} registros de la página...", page_idx);
                    }

                    if let Ok(record) = result {
                        if record.len() >= 19 {
                            trips.push(Trip {
                                vendor_id: record[0].to_string(),
                                tpep_pickup_datetime: record[1].to_string(),
                                tpep_dropoff_datetime: record[2].to_string(),
                                passenger_count: record[3].to_string(),
                                trip_distance: record[4].to_string(),
                                ratecode_id: record[5].to_string(),
                                store_and_fwd_flag: record[6].to_string(),
                                pu_location_id: record[7].to_string(),
                                do_location_id: record[8].to_string(),
                                payment_type: record[9].to_string(),
                                fare_amount: record[10].to_string(),
                                extra: record[11].to_string(),
                                mta_tax: record[12].to_string(),
                                tip_amount: record[13].to_string(),
                                tolls_amount: record[14].to_string(),
                                improvement_surcharge: record[15].to_string(),
                                total_amount: record[16].to_string(),
                                congestion_surcharge: record[17].to_string(),
                                index: record[18].to_string(),
                            });
                        }
                    }
                }

                println!("Cargados {} registros para la página {}", trips.len(), page);

                let mut state = state_clone.lock().unwrap();
                state.filtered_results = trips;
                state.current_page = page;
                state.is_filtering = false;

                println!("Página {} cargada correctamente", page);
            } else {
                println!(
                    "ERROR: No se pudo abrir el archivo de resultados para la página {}",
                    page
                );
                let mut state = state_clone.lock().unwrap();
                state.filter_error = Some(format!("No se pudo cargar la página {}", page));
                state.is_filtering = false;
            }
        });
    }

    fn get_statistics(&self) {
        // Verificar si ya está filtrando
        {
            let mut state = self.state.lock().unwrap();
            if state.is_filtering {
                println!("Ya hay un proceso en curso, ignorando solicitud de estadísticas");
                return;
            }
            state.is_filtering = true;
            state.filter_error = None;
            state.should_switch_tab = Some(Tab::Stats);
        }

        println!("Obteniendo estadísticas de los datos...");
        let filter = self.build_filter();
        let state_clone = Arc::clone(&self.state);

        // Obtener estadísticas en un hilo separado
        thread::spawn(move || {
            println!("Calculando estadísticas...");
            match filters::get_filter_stats(CSV_PATH, filter) {
                Ok(stats) => {
                    println!("Estadísticas calculadas correctamente:");
                    println!(
                        "  - Total registros: {}",
                        stats.get("count").unwrap_or(&0.0)
                    );
                    if let Some(count) = stats.get("count") {
                        if *count > 0.0 {
                            println!(
                                "  - Distancia promedio: {:.2}",
                                stats.get("avg_distance").unwrap_or(&0.0)
                            );
                            println!(
                                "  - Precio promedio: ${:.2}",
                                stats.get("avg_amount").unwrap_or(&0.0)
                            );
                            println!(
                                "  - Pasajeros promedio: {:.1}",
                                stats.get("avg_passengers").unwrap_or(&0.0)
                            );
                            println!(
                                "  - Monto total: ${:.2}",
                                stats.get("total_amount").unwrap_or(&0.0)
                            );
                        }
                    }

                    let mut state = state_clone.lock().unwrap();
                    state.stats = Some(stats);
                    state.statistics_loaded = true;
                    state.is_filtering = false;
                }
                Err(e) => {
                    println!("ERROR al calcular estadísticas: {}", e);
                    let mut state = state_clone.lock().unwrap();
                    state.filter_error = Some(format!("Error al obtener estadísticas: {}", e));
                    state.is_filtering = false;
                }
            }
        });
    }

    fn get_popular_destinations(&self) {
        // Verificar si ya está filtrando
        {
            let mut state = self.state.lock().unwrap();
            if state.is_filtering {
                println!("Ya hay un proceso en curso, ignorando solicitud de destinos populares");
                return;
            }
            state.is_filtering = true;
            state.filter_error = None;
            state.should_switch_tab = Some(Tab::PopularDestinations);
        }

        println!("Obteniendo destinos populares...");
        let state_clone = Arc::clone(&self.state);

        // Obtener destinos populares en un hilo separado
        thread::spawn(move || {
            println!("Analizando destinos más frecuentes...");
            match filters::get_popular_destinations(CSV_PATH, 20) {
                Ok(destinations) => {
                    println!("Se encontraron {} destinos populares", destinations.len());
                    for (i, (dest, count)) in destinations.iter().enumerate().take(5) {
                        println!("  {}. Destino {}: {} viajes", i + 1, dest, count);
                    }
                    if destinations.len() > 5 {
                        println!("  ... y {} destinos más", destinations.len() - 5);
                    }

                    let mut state = state_clone.lock().unwrap();
                    state.popular_destinations = Some(destinations);
                    state.destinations_loaded = true;
                    state.is_filtering = false;
                }
                Err(e) => {
                    println!("ERROR al obtener destinos populares: {}", e);
                    let mut state = state_clone.lock().unwrap();
                    state.filter_error =
                        Some(format!("Error al obtener destinos populares: {}", e));
                    state.is_filtering = false;
                }
            }
        });
    }

    fn export_results(&self) {
        // Verificar si ya está filtrando o si el nombre de archivo está vacío
        {
            let mut state = self.state.lock().unwrap();
            if state.is_filtering || self.export_filename.is_empty() {
                println!(
                    "No se puede exportar: {} {}",
                    if state.is_filtering {
                        "hay un proceso en curso"
                    } else {
                        ""
                    },
                    if self.export_filename.is_empty() {
                        "nombre de archivo vacío"
                    } else {
                        ""
                    }
                );
                return;
            }
            state.is_filtering = true;
            state.export_status = None;
        }

        let filter = self.build_filter();
        let filename = self.export_filename.clone();
        let output_path = format!("{}/{}", TMP_DIR, filename);
        let state_clone = Arc::clone(&self.state);

        println!("Exportando resultados a: {}", output_path);

        // Exportar en un hilo separado
        thread::spawn(move || {
            println!("Aplicando filtros y exportando datos...");
            match filters::filter_to_file(CSV_PATH, &output_path, filter, None) {
                Ok(count) => {
                    println!("Exportación completada. Se exportaron {} registros", count);
                    let mut state = state_clone.lock().unwrap();
                    state.export_status =
                        Some(format!("Exportados {} registros a {}", count, output_path));
                    state.is_filtering = false;
                }
                Err(e) => {
                    println!("ERROR al exportar: {}", e);
                    let mut state = state_clone.lock().unwrap();
                    state.export_status = Some(format!("Error al exportar: {}", e));
                    state.is_filtering = false;
                }
            }
        });
    }

    fn show_data_tab(&self, ui: &mut egui::Ui) {
        // Extraer toda la información necesaria del estado primero
        let data_info = {
            let state = self.state.lock().unwrap();
            let results = state.filtered_results.clone(); // Clonamos los resultados para usarlos después
            let current_page = state.current_page;
            let total_pages = state.total_pages;
            let total_count = state.results_count;
            let is_filtering = state.is_filtering;

            (
                results,
                current_page,
                total_pages,
                total_count,
                is_filtering,
            )
        };

        let (results, current_page, total_pages, total_count, is_filtering) = data_info;

        // Calcular índices para mostrar información sobre los registros visualizados
        let start_index = current_page * MAX_DISPLAYED_ROWS + 1;
        let end_index = std::cmp::min(start_index + results.len() - 1, total_count);

        ui.label(format!(
            "Mostrando registros {}-{} de {} resultados (Página {} de {})",
            start_index,
            end_index,
            total_count,
            current_page + 1,
            total_pages
        ));

        // Controles de paginación
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    current_page > 0 && !is_filtering,
                    egui::Button::new("← Anterior"),
                )
                .clicked()
            {
                self.load_page(current_page - 1);
                return;
            }

            // Mostrar algunos botones de página cercanos a la página actual
            let show_pages = 5; // Número de páginas para mostrar a cada lado
            let start_page = if current_page > show_pages {
                current_page - show_pages
            } else {
                0
            };

            let end_page = std::cmp::min(current_page + show_pages + 1, total_pages);

            if start_page > 0 {
                if ui
                    .add_enabled(!is_filtering, egui::Button::new("1"))
                    .clicked()
                {
                    self.load_page(0);
                    return;
                }

                if start_page > 1 {
                    ui.label("...");
                }
            }

            for page in start_page..end_page {
                let is_current = page == current_page;
                let button_text = format!("{}", page + 1);

                if is_current {
                    ui.add(
                        egui::Button::new(button_text).fill(egui::Color32::from_rgb(100, 150, 200)),
                    );
                } else if ui
                    .add_enabled(!is_filtering, egui::Button::new(button_text))
                    .clicked()
                {
                    self.load_page(page);
                    return;
                }
            }

            if end_page < total_pages {
                if end_page < total_pages - 1 {
                    ui.label("...");
                }

                let last_page = total_pages - 1;
                if ui
                    .add_enabled(!is_filtering, egui::Button::new(format!("{}", total_pages)))
                    .clicked()
                {
                    self.load_page(last_page);
                    return;
                }
            }

            if ui
                .add_enabled(
                    current_page < total_pages - 1 && !is_filtering,
                    egui::Button::new("Siguiente →"),
                )
                .clicked()
            {
                self.load_page(current_page + 1);
                return;
            }
        });

        // Crear la tabla de resultados
        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::remainder().at_least(30.0)) // ID
                    .column(Column::remainder().at_least(50.0)) // Pickup
                    .column(Column::remainder().at_least(50.0)) // Dropoff
                    .column(Column::remainder().at_least(30.0)) // Passengers
                    .column(Column::remainder().at_least(40.0)) // Distance
                    .column(Column::remainder().at_least(50.0)) // Total Amount
                    .column(Column::remainder().at_least(40.0)) // Origin
                    .column(Column::remainder().at_least(40.0)) // Destination
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("ID");
                        });
                        header.col(|ui| {
                            ui.strong("Pickup");
                        });
                        header.col(|ui| {
                            ui.strong("Dropoff");
                        });
                        header.col(|ui| {
                            ui.strong("Pasajeros");
                        });
                        header.col(|ui| {
                            ui.strong("Distancia");
                        });
                        header.col(|ui| {
                            ui.strong("Precio Total");
                        });
                        header.col(|ui| {
                            ui.strong("Origen");
                        });
                        header.col(|ui| {
                            ui.strong("Destino");
                        });
                    })
                    .body(|mut body| {
                        for trip in &results {
                            body.row(18.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(&trip.index);
                                });
                                row.col(|ui| {
                                    ui.label(&trip.tpep_pickup_datetime);
                                });
                                row.col(|ui| {
                                    ui.label(&trip.tpep_dropoff_datetime);
                                });
                                row.col(|ui| {
                                    ui.label(&trip.passenger_count);
                                });
                                row.col(|ui| {
                                    ui.label(&trip.trip_distance);
                                });
                                row.col(|ui| {
                                    ui.label(&trip.total_amount);
                                });
                                row.col(|ui| {
                                    ui.label(&trip.pu_location_id);
                                });
                                row.col(|ui| {
                                    ui.label(&trip.do_location_id);
                                });
                            });
                        }
                    });
            });

        // Añadir una segunda barra de paginación al final de la tabla
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    current_page > 0 && !is_filtering,
                    egui::Button::new("← Anterior"),
                )
                .clicked()
            {
                self.load_page(current_page - 1);
            }

            ui.label(format!("Página {} de {}", current_page + 1, total_pages));

            if ui
                .add_enabled(
                    current_page < total_pages - 1 && !is_filtering,
                    egui::Button::new("Siguiente →"),
                )
                .clicked()
            {
                self.load_page(current_page + 1);
            }
        });
    }

    fn show_stats_tab(&self, ui: &mut egui::Ui) {
        let stats_option = {
            let state = self.state.lock().unwrap();
            state.stats.clone()
        };

        if let Some(stats) = stats_option {
            ui.heading("Estadísticas de viajes filtrados");

            ui.label(format!(
                "Número total de viajes: {}",
                stats.get("count").unwrap_or(&0.0).round() as i32
            ));

            if stats.get("count").unwrap_or(&0.0) > &0.0 {
                ui.label(format!(
                    "Distancia promedio: {:.2} km",
                    stats.get("avg_distance").unwrap_or(&0.0)
                ));
                ui.label(format!(
                    "Precio promedio: ${:.2}",
                    stats.get("avg_amount").unwrap_or(&0.0)
                ));
                ui.label(format!(
                    "Pasajeros promedio: {:.1}",
                    stats.get("avg_passengers").unwrap_or(&0.0)
                ));
                ui.label(format!(
                    "Monto total: ${:.2}",
                    stats.get("total_amount").unwrap_or(&0.0)
                ));
            } else {
                ui.label("No hay datos para mostrar estadísticas.");
            }
        } else {
            ui.label("Haz clic en 'Obtener Estadísticas' para ver datos estadísticos.");
        }
    }

    fn show_popular_destinations_tab(&self, ui: &mut egui::Ui) {
        let destinations_option = {
            let state = self.state.lock().unwrap();
            state.popular_destinations.clone()
        };

        if let Some(destinations) = destinations_option {
            ui.heading("Destinos Más Populares");

            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::remainder().at_least(100.0))
                        .column(Column::remainder().at_least(100.0))
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.strong("ID Ubicación");
                            });
                            header.col(|ui| {
                                ui.strong("Número de Viajes");
                            });
                        })
                        .body(|mut body| {
                            for (dest, count) in destinations {
                                body.row(18.0, |mut row| {
                                    row.col(|ui| {
                                        ui.label(&dest);
                                    });
                                    row.col(|ui| {
                                        ui.label(format!("{}", count));
                                    });
                                });
                            }
                        });
                });
        } else {
            ui.label("Haz clic en 'Ver Destinos Populares' para ver los destinos más frecuentes.");
        }
    }

    // Método para verificar y cambiar de pestaña automáticamente
    fn check_tab_switch(&mut self) {
        let switch_to = {
            let mut state = self.state.lock().unwrap();
            let tab = state.should_switch_tab.take();
            tab
        };

        if let Some(tab) = switch_to {
            self.selected_tab = tab;
        }
    }
}

// Función auxiliar para crear filtros con los mismos parámetros
fn create_filter(
    min_price: Option<f64>,
    max_price: Option<f64>,
    index: &str,
    destination: &str,
    use_and: bool,
) -> TripFilter {
    let mut filters = Vec::new();

    // Filtro de precio
    if min_price.is_some() || max_price.is_some() {
        filters.push(TripFilter::Price {
            min: min_price,
            max: max_price,
        });
    }

    // Filtro por índice
    if !index.is_empty() {
        filters.push(TripFilter::Index(index.to_string()));
    }

    // Filtro por destino
    if !destination.is_empty() {
        filters.push(TripFilter::Destination(destination.to_string()));
    }

    // Si no hay filtros, crear uno que siempre da true
    if filters.is_empty() {
        return TripFilter::Price {
            min: None,
            max: None,
        };
    }

    // Combinar filtros con AND u OR
    if filters.len() > 1 {
        if use_and {
            TripFilter::And(filters)
        } else {
            TripFilter::Or(filters)
        }
    } else {
        // Si solo hay un filtro, lo devolvemos directamente
        filters.remove(0)
    }
}

impl eframe::App for FilterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Verificar si debemos cambiar de pestaña
        self.check_tab_switch();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Filtrar y Visualizar Datos de Viajes");

            // Panel de filtros
            egui::CollapsingHeader::new("Filtros").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Precio mínimo:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.min_price)
                            .hint_text("Mínimo")
                            .desired_width(80.0),
                    );

                    ui.label("Precio máximo:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.max_price)
                            .hint_text("Máximo")
                            .desired_width(80.0),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Índice:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.index_filter)
                            .hint_text("ID del viaje")
                            .desired_width(120.0),
                    );

                    ui.label("Destino:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.destination_filter)
                            .hint_text("ID de ubicación")
                            .desired_width(120.0),
                    );
                });

                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.use_and, true, "AND lógico");
                    ui.radio_value(&mut self.use_and, false, "OR lógico");
                });

                // Verificamos si hay un proceso en curso antes de habilitar los botones
                let is_filtering = {
                    let state = self.state.lock().unwrap();
                    state.is_filtering
                };

                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(!is_filtering, egui::Button::new("Aplicar Filtros"))
                        .clicked()
                    {
                        self.apply_filter();
                    }

                    if ui
                        .add_enabled(!is_filtering, egui::Button::new("Obtener Estadísticas"))
                        .clicked()
                    {
                        self.get_statistics();
                    }

                    if ui
                        .add_enabled(!is_filtering, egui::Button::new("Ver Destinos Populares"))
                        .clicked()
                    {
                        self.get_popular_destinations();
                    }

                    // Botón para cargar todo
                    if ui
                        .add_enabled(!is_filtering, egui::Button::new("Cargar Todo"))
                        .clicked()
                    {
                        self.load_all();
                    }
                });

                // Panel de exportación
                ui.horizontal(|ui| {
                    ui.label("Exportar a:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.export_filename)
                            .hint_text("nombre_archivo.csv")
                            .desired_width(200.0),
                    );

                    if ui
                        .add_enabled(!is_filtering, egui::Button::new("Exportar"))
                        .clicked()
                    {
                        self.export_results();
                    }
                });

                // Mostrar estado de la exportación
                let export_status = {
                    let state = self.state.lock().unwrap();
                    state.export_status.clone()
                };

                if let Some(status) = export_status {
                    ui.label(status);
                }
            });

            // Mensaje de espera durante el filtrado y errores
            {
                let state = self.state.lock().unwrap();
                if state.is_filtering {
                    ui.label("Procesando datos...");
                    ui.spinner();
                }

                // Mostrar mensajes de error
                if let Some(error) = &state.filter_error {
                    ui.label(egui::RichText::new(error).color(egui::Color32::RED));
                }
            }

            // Selector de pestañas con información de carga
            {
                let state = self.state.lock().unwrap();
                let stats_loaded = state.statistics_loaded;
                let destinations_loaded = state.destinations_loaded;
                let is_filtering = state.is_filtering;
                drop(state); // Liberar el mutex antes de interactuar con la UI

                ui.horizontal(|ui| {
                    if ui
                        .selectable_value(&mut self.selected_tab, Tab::Data, "Datos")
                        .clicked()
                    {
                        // No necesitamos hacer nada, solo cambiar la pestaña
                    }

                    let stats_text = if stats_loaded {
                        "Estadísticas ✓"
                    } else {
                        "Estadísticas"
                    };
                    if ui
                        .selectable_value(&mut self.selected_tab, Tab::Stats, stats_text)
                        .clicked()
                    {
                        if !stats_loaded && !is_filtering {
                            // Si se selecciona estadísticas pero no están cargadas, cargarlas
                            self.get_statistics();
                        }
                    }

                    let dest_text = if destinations_loaded {
                        "Destinos Populares ✓"
                    } else {
                        "Destinos Populares"
                    };
                    if ui
                        .selectable_value(
                            &mut self.selected_tab,
                            Tab::PopularDestinations,
                            dest_text,
                        )
                        .clicked()
                    {
                        if !destinations_loaded && !is_filtering {
                            // Si se selecciona destinos pero no están cargados, cargarlos
                            self.get_popular_destinations();
                        }
                    }
                });
            }

            // Contenido según la pestaña seleccionada
            match self.selected_tab {
                Tab::Data => self.show_data_tab(ui),
                Tab::Stats => self.show_stats_tab(ui),
                Tab::PopularDestinations => self.show_popular_destinations_tab(ui),
            }
        });
    }
}

// Función principal que inicia la aplicación
pub fn run_app() -> Result<(), eframe::Error> {
    println!("Iniciando aplicación gráfica...");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_resizable(true),
        ..Default::default()
    };

    println!("Lanzando interfaz gráfica");

    eframe::run_native(
        "Visualizador de Datos de Viajes",
        options,
        Box::new(|_cc| {
            println!("Inicializando aplicación");
            Ok(Box::new(FilterApp::default()))
        }),
    )
}
