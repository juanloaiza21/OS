use std::fs::File;
use std::io::{BufRead, BufReader};

use chrono::{Datelike, NaiveDate};
use eframe::{
    egui::{self, CentralPanel, Context, TextEdit, TopBottomPanel, ComboBox, RichText, Grid, ScrollArea},
    App as EframeApp,
};

#[derive(Clone)]
struct Trip {
    index: usize,
    pickup: String,
    dropoff: String,
    total_amount: f32,
    passengers: u32,
    distance: f32,
}

pub struct App {
    trips: Vec<Trip>,
    filtered_trips: Vec<Trip>,
    index_query: String,
    cost_operator: String,
    cost_value: String,

    use_date_filter: bool,
    selected_day: u32,
    selected_month: u32,
    selected_year: i32,

    has_searched: bool,
}

impl Default for App {
    fn default() -> Self {
        let trips = load_trips("trips.csv");
        Self {
            trips,
            filtered_trips: vec![], // al inicio no muestra nada
            index_query: String::new(),
            cost_operator: "<=".to_string(),
            cost_value: String::new(),

            use_date_filter: false,
            selected_day: 1,
            selected_month: 1,
            selected_year: 2020,

            has_searched: false,
        }
    }
}

fn load_trips(path: &str) -> Vec<Trip> {
    let file = File::open(path).expect("No se pudo abrir el archivo CSV.");
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let _header = lines.next();
    let mut trips = Vec::new();
    let mut index_counter = 1;

    for line in lines.flatten() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 18 {
            continue;
        }

        let pickup = parts[1].split_whitespace().next().unwrap_or("").to_string();
        let dropoff = parts[2].to_string();
        let passengers = parts[3].parse::<u32>().unwrap_or(0);
        let distance = parts[4].parse::<f32>().unwrap_or(0.0);
        let total_amount = parts[16].parse::<f32>().unwrap_or(0.0);

        trips.push(Trip {
            index: index_counter,
            pickup,
            dropoff,
            total_amount,
            passengers,
            distance,
        });

        index_counter += 1;
    }

    trips
}

impl EframeApp for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("Consulta de Viajes"); // Título de la parte superior

            ui.columns(4, |cols| {
                // Índice
                cols[0].vertical(|ui| {
                    ui.label("Índice:");
                    ui.add(TextEdit::singleline(&mut self.index_query).hint_text("Ej: 128"));
                });

                // Fecha
                cols[1].vertical(|ui| {
                    ui.checkbox(&mut self.use_date_filter, "Usar fecha:");
                    if self.use_date_filter {
                        ComboBox::from_id_source("year_combo")
                            .selected_text(self.selected_year.to_string())
                            .show_ui(ui, |ui| {
                                for year in 2009..=2025 {
                                    ui.selectable_value(&mut self.selected_year, year, year.to_string());
                                }
                            });

                        ComboBox::from_id_source("month_combo")
                            .selected_text(self.selected_month.to_string())
                            .show_ui(ui, |ui| {
                                for month in 1..=12 {
                                    ui.selectable_value(&mut self.selected_month, month, month.to_string());
                                }
                            });

                        let days_in_month = days_in_month(self.selected_year, self.selected_month);
                        ComboBox::from_id_source("day_combo")
                            .selected_text(self.selected_day.to_string())
                            .show_ui(ui, |ui| {
                                for day in 1..=days_in_month {
                                    ui.selectable_value(&mut self.selected_day, day, day.to_string());
                                }
                            });
                    }
                });

                // Costo
                cols[2].vertical(|ui| {
                    ui.label("Costo:");
                    ComboBox::from_id_source("operator_combo")
                        .selected_text(&self.cost_operator)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.cost_operator, "<=".to_string(), "<=");
                            ui.selectable_value(&mut self.cost_operator, ">=".to_string(), ">=");
                            ui.selectable_value(&mut self.cost_operator, "=".to_string(), "=");
                        });
                    ui.add(TextEdit::singleline(&mut self.cost_value).hint_text("Ej: 10"));
                });

                // Buscar y limpiar
                cols[3].vertical(|ui| {
                    ui.add_space(20.0);
                    if ui.button("Buscar").clicked() {
                        let selected_date = if self.use_date_filter {
                            NaiveDate::from_ymd_opt(self.selected_year, self.selected_month, self.selected_day)
                        } else {
                            None
                        };

                        self.filtered_trips = self.trips
                            .iter()
                            .filter(|trip| {
                                let index_match = self.index_query.is_empty()
                                    || trip.index.to_string() == self.index_query;

                                let date_match = if let Some(sel_date) = selected_date.clone() {
                                    NaiveDate::parse_from_str(&trip.pickup, "%Y-%m-%d")
                                        .map(|d| d == sel_date)
                                        .unwrap_or(false)
                                } else {
                                    true
                                };

                                let cost_match = if let Ok(val) = self.cost_value.parse::<f32>() {
                                    match self.cost_operator.as_str() {
                                        "<=" => trip.total_amount <= val,
                                        ">=" => trip.total_amount >= val,
                                        "=" => (trip.total_amount - val).abs() < f32::EPSILON,
                                        _ => true,
                                    }
                                } else {
                                    true
                                };

                                index_match && date_match && cost_match
                            })
                            .cloned()
                            .collect();

                        self.has_searched = true;
                    }

                    if ui.button("Limpiar").clicked() {
                        self.index_query.clear();
                        self.cost_value.clear();
                        self.use_date_filter = false;
                        self.filtered_trips.clear();
                        self.has_searched = false;
                    }
                });
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                if !self.has_searched {
                    ui.label(RichText::new("Aquí se mostrarán tus resultados").italics().weak());
                } else if self.filtered_trips.is_empty() {
                    ui.label(RichText::new("No se encontraron resultados.").italics().weak());
                } else {
                    Grid::new("results_grid")
                        .striped(true)
                        .min_col_width(80.0)
                        .show(ui, |ui| {
                            ui.label(RichText::new("Índice").strong());
                            ui.label(RichText::new("Fecha inicio").strong());
                            ui.label(RichText::new("Fecha fin").strong());
                            ui.label(RichText::new("Precio").strong());
                            ui.label(RichText::new("Pasajeros").strong());
                            ui.label(RichText::new("Distancia").strong());
                            ui.end_row();

                            for trip in &self.filtered_trips {
                                ui.label(trip.index.to_string());
                                ui.label(&trip.pickup);
                                ui.label(&trip.dropoff);
                                ui.label(format!("{:.2}", trip.total_amount));
                                ui.label(trip.passengers.to_string());
                                ui.label(format!("{:.2}", trip.distance));
                                ui.end_row();
                            }
                        });
                }
            });
        });
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
