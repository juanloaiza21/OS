use crate::data::filters::{self, TripFilter};
use crate::data::trip_struct::Trip;
use eframe::{self, egui};
use egui_extras::{Column, TableBuilder};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

const CSV_PATH: &str = "../data/data.csv";
const TMP_DIR: &str = "../../tmp/";
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
        if !Path::new(TMP_DIR).exists() {
            fs::create_dir_all(TMP_DIR).expect("No se pudo crear el directorio tmp");
        }

        Self {
            min_price: String::new(),
            max_price: String::new(),
            index_filter: String::new(),
            destination_filter: String::new(),
            use_and: true,
            state: Arc::new(Mutex::new(FilterState::default())),
            selected_tab: Tab::default(),
            export_filename: "filtered_data.csv".to_string(),
        }
    }
}

impl FilterApp {
    fn build_filter(&self) -> TripFilter {
        let mut filters = Vec::new();

        // Filtro de precio
        let min_price = self.min_price.parse::<f64>().ok();
        let max_price = self.max_price.parse::<f64>().ok();
        if min_price.is_some() || max_price.is_some() {
            filters.push(TripFilter::Price {
                min: min_price,
                max: max_price,
            });
        }

        // Filtro por índice
        if !self.index_filter.is_empty() {
            filters.push(TripFilter::Index(self.index_filter.clone()));
        }

        // Filtro por destino
        if !self.destination_filter.is_empty() {
            filters.push(TripFilter::Destination(self.destination_filter.clone()));
        }

        // Si no hay filtros, crear uno que siempre da true
        if filters.is_empty() {
            // Si no se especifica ningún filtro, usamos un filtro de precio con límites infinitos
            return TripFilter::Price {
                min: None,
                max: None,
            };
        }

        // Combinar filtros con AND u OR
        if filters.len() > 1 {
            if self.use_and {
                TripFilter::And(filters)
            } else {
                TripFilter::Or(filters)
            }
        } else {
            // Si solo hay un filtro, lo devolvemos directamente
            filters.remove(0)
        }
    }

    fn apply_filter(&self) {
        // Verificar si ya está filtrando
        {
            let mut state = self.state.lock().unwrap();
            if state.is_filtering {
                return;
            }
            state.is_filtering = true;
            state.filter_error = None;
        }

        let filter = self.build_filter();
        let state_clone = Arc::clone(&self.state);

        // Ejecutar el filtrado en un hilo separado para no bloquear la UI
        thread::spawn(move || {
            // Crear un archivo temporal para los resultados
            let tmp_file = format!("{}/temp_filter_results.csv", TMP_DIR);

            // Aplicar el filtrado y guardar a archivo
            match filters::filter_to_file(CSV_PATH, &tmp_file, filter, Some(MAX_DISPLAYED_ROWS * 2))
            {
                Ok(count) => {
                    // Cargar los primeros N registros para mostrar
                    if let Ok(file) = std::fs::File::open(&tmp_file) {
                        let reader = std::io::BufReader::new(file);
                        let mut csv_reader = csv::ReaderBuilder::new()
                            .has_headers(true)
                            .from_reader(reader);

                        let mut trips = Vec::with_capacity(MAX_DISPLAYED_ROWS);
                        for result in csv_reader.records().take(MAX_DISPLAYED_ROWS) {
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

                        // Actualizar los resultados
                        let mut state = state_clone.lock().unwrap();
                        state.filtered_results = trips;
                        state.results_count = count;
                        state.is_filtering = false;

                        // Eliminar el archivo temporal
                        let _ = std::fs::remove_file(&tmp_file);
                    } else {
                        let mut state = state_clone.lock().unwrap();
                        state.filter_error =
                            Some("No se pudo abrir el archivo de resultados".to_string());
                        state.is_filtering = false;
                    }
                }
                Err(e) => {
                    let mut state = state_clone.lock().unwrap();
                    state.filter_error = Some(format!("Error al filtrar: {}", e));
                    state.is_filtering = false;
                }
            }
        });
    }

    fn get_statistics(&self) {
        // Verificar si ya está filtrando
        {
            let mut state = self.state.lock().unwrap();
            if state.is_filtering {
                return;
            }
            state.is_filtering = true;
            state.filter_error = None;
        }

        let filter = self.build_filter();
        let state_clone = Arc::clone(&self.state);

        // Obtener estadísticas en un hilo separado
        thread::spawn(move || match filters::get_filter_stats(CSV_PATH, filter) {
            Ok(stats) => {
                let mut state = state_clone.lock().unwrap();
                state.stats = Some(stats);
                state.is_filtering = false;
            }
            Err(e) => {
                let mut state = state_clone.lock().unwrap();
                state.filter_error = Some(format!("Error al obtener estadísticas: {}", e));
                state.is_filtering = false;
            }
        });
    }

    fn get_popular_destinations(&self) {
        // Verificar si ya está filtrando
        {
            let mut state = self.state.lock().unwrap();
            if state.is_filtering {
                return;
            }
            state.is_filtering = true;
            state.filter_error = None;
        }

        let state_clone = Arc::clone(&self.state);

        // Obtener destinos populares en un hilo separado
        thread::spawn(
            move || match filters::get_popular_destinations(CSV_PATH, 20) {
                Ok(destinations) => {
                    let mut state = state_clone.lock().unwrap();
                    state.popular_destinations = Some(destinations);
                    state.is_filtering = false;
                }
                Err(e) => {
                    let mut state = state_clone.lock().unwrap();
                    state.filter_error =
                        Some(format!("Error al obtener destinos populares: {}", e));
                    state.is_filtering = false;
                }
            },
        );
    }

    fn export_results(&self) {
        // Verificar si ya está filtrando o si el nombre de archivo está vacío
        {
            let mut state = self.state.lock().unwrap();
            if state.is_filtering || self.export_filename.is_empty() {
                return;
            }
            state.is_filtering = true;
            state.export_status = None;
        }

        let filter = self.build_filter();
        let filename = self.export_filename.clone();
        let output_path = format!("{}/{}", TMP_DIR, filename);
        let state_clone = Arc::clone(&self.state);

        // Exportar en un hilo separado
        thread::spawn(move || {
            match filters::filter_to_file(CSV_PATH, &output_path, filter, None) {
                Ok(count) => {
                    let mut state = state_clone.lock().unwrap();
                    state.export_status =
                        Some(format!("Exportados {} registros a {}", count, output_path));
                    state.is_filtering = false;
                }
                Err(e) => {
                    let mut state = state_clone.lock().unwrap();
                    state.export_status = Some(format!("Error al exportar: {}", e));
                    state.is_filtering = false;
                }
            }
        });
    }

    fn show_data_tab(&self, ui: &mut egui::Ui) {
        let state = self.state.lock().unwrap();
        let results = &state.filtered_results;

        ui.label(format!(
            "Mostrando {} de {} resultados",
            results.len(),
            state.results_count
        ));

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
                        for trip in results.iter() {
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
    }

    fn show_stats_tab(&self, ui: &mut egui::Ui) {
        let state = self.state.lock().unwrap();

        if let Some(stats) = &state.stats {
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
        let state = self.state.lock().unwrap();

        if let Some(destinations) = &state.popular_destinations {
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
                            for (dest, count) in destinations.iter() {
                                body.row(18.0, |mut row| {
                                    row.col(|ui| {
                                        ui.label(dest);
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
}

impl eframe::App for FilterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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

                ui.horizontal(|ui| {
                    let state = self.state.lock().unwrap();
                    let is_filtering = state.is_filtering;
                    drop(state); // Liberar el candado antes de las operaciones UI

                    if ui.button("Aplicar Filtros").clicked() && !is_filtering {
                        self.apply_filter();
                    }

                    if ui.button("Obtener Estadísticas").clicked() && !is_filtering {
                        self.get_statistics();
                    }

                    if ui.button("Ver Destinos Populares").clicked() && !is_filtering {
                        self.get_popular_destinations();
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

                    let state = self.state.lock().unwrap();
                    let is_filtering = state.is_filtering;
                    drop(state);

                    if ui.button("Exportar").clicked() && !is_filtering {
                        self.export_results();
                    }
                });

                // Mostrar estado de la exportación
                let state = self.state.lock().unwrap();
                if let Some(status) = &state.export_status {
                    ui.label(status);
                }
            });

            // Mensaje de espera durante el filtrado
            let state = self.state.lock().unwrap();
            if state.is_filtering {
                ui.label("Procesando datos...");
                ui.spinner();
            }

            // Mostrar mensajes de error
            if let Some(error) = &state.filter_error {
                ui.label(egui::RichText::new(error).color(egui::Color32::RED));
            }

            // Selector de pestañas
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, Tab::Data, "Datos");
                ui.selectable_value(&mut self.selected_tab, Tab::Stats, "Estadísticas");
                ui.selectable_value(
                    &mut self.selected_tab,
                    Tab::PopularDestinations,
                    "Destinos Populares",
                );
            });

            // Liberar el candado antes de mostrar contenido de pestañas
            drop(state);

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
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Visualizador de Datos de Viajes",
        options,
        Box::new(|_cc| Ok(Box::new(FilterApp::default()))),
    )
}
