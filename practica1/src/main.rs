mod data;

use data::filters::TripFilter;
use std::env;
use std::error::Error;
use std::path::Path;
use std::time::Instant;

fn print_usage() {
    println!("Uso:");
    println!(
        "  {} build <ruta_csv> <directorio_hash_table>",
        env::args().nth(0).unwrap_or_default()
    );
    println!(
        "  {} filter-price <ruta_csv> <archivo_salida> <precio_min> <precio_max>",
        env::args().nth(0).unwrap_or_default()
    );
    println!(
        "  {} filter-index <ruta_csv> <archivo_salida> <index>",
        env::args().nth(0).unwrap_or_default()
    );
    println!(
        "  {} filter-dest <ruta_csv> <archivo_salida> <destino>",
        env::args().nth(0).unwrap_or_default()
    );
    println!(
        "  {} popular-dests <ruta_csv> <número_de_resultados>",
        env::args().nth(0).unwrap_or_default()
    );
    println!("\nEjemplos:");
    println!(
        "  {} build data/yellow_tripdata.csv hash_table_dir",
        env::args().nth(0).unwrap_or_default()
    );
    println!(
        "  {} filter-price data/yellow_tripdata.csv resultados/viajes_caros.csv 50 100",
        env::args().nth(0).unwrap_or_default()
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let command = &args[1];

    match command.as_str() {
        "build" => {
            if args.len() < 4 {
                println!("Error: Faltan argumentos para el comando build");
                print_usage();
                return Ok(());
            }

            let csv_path = &args[2];
            let hash_dir = &args[3];

            let start_time = Instant::now();
            println!("Iniciando construcción de la hash table...");

            let count = data::disk_hash::build_hash_table_from_csv(csv_path, hash_dir)?;

            let duration = start_time.elapsed();
            println!("Construcción completada en {:.2?}", duration);
            println!("Se procesaron {} registros", count);
        }

        "filter-price" => {
            if args.len() < 6 {
                println!("Error: Faltan argumentos para el comando filter-price");
                print_usage();
                return Ok(());
            }

            let csv_path = &args[2];
            let output_file = &args[3];
            let min_price = args[4].parse::<f64>().unwrap_or(0.0);
            let max_price = args[5].parse::<f64>().unwrap_or(f64::MAX);

            let start_time = Instant::now();
            println!(
                "Filtrando viajes por precio (${} - ${})...",
                min_price, max_price
            );

            let filter = TripFilter::Price {
                min: Some(min_price),
                max: Some(max_price),
            };

            let count = data::filters::filter_to_file(csv_path, output_file, filter, None)?;

            let duration = start_time.elapsed();
            println!("Filtrado completado en {:.2?}", duration);
            println!("Se encontraron {} viajes que cumplen el criterio", count);
            println!("Resultados guardados en: {}", output_file);
        }

        "filter-index" => {
            if args.len() < 5 {
                println!("Error: Faltan argumentos para el comando filter-index");
                print_usage();
                return Ok(());
            }

            let csv_path = &args[2];
            let output_file = &args[3];
            let index = &args[4];

            let start_time = Instant::now();
            println!("Buscando viaje con index: {}...", index);

            let filter = TripFilter::Index(index.to_string());

            let count = data::filters::filter_to_file(csv_path, output_file, filter, Some(1))?;

            let duration = start_time.elapsed();
            println!("Búsqueda completada en {:.2?}", duration);

            if count > 0 {
                println!("Se encontró el viaje con index: {}", index);
                println!("Resultado guardado en: {}", output_file);
            } else {
                println!("No se encontró ningún viaje con index: {}", index);
            }
        }

        "filter-dest" => {
            if args.len() < 5 {
                println!("Error: Faltan argumentos para el comando filter-dest");
                print_usage();
                return Ok(());
            }

            let csv_path = &args[2];
            let output_file = &args[3];
            let destination = &args[4];

            let start_time = Instant::now();
            println!("Filtrando viajes con destino: {}...", destination);

            let filter = TripFilter::Destination(destination.to_string());

            let count = data::filters::filter_to_file(csv_path, output_file, filter, None)?;

            let duration = start_time.elapsed();
            println!("Filtrado completado en {:.2?}", duration);
            println!(
                "Se encontraron {} viajes con destino {}",
                count, destination
            );
            println!("Resultados guardados en: {}", output_file);

            // Adicionalmente, mostrar estadísticas
            println!("\nEstadísticas para viajes a este destino:");
            let stats = data::filters::get_filter_stats(
                csv_path,
                TripFilter::Destination(destination.to_string()),
            )?;

            println!(
                "  - Total de viajes: {}",
                stats.get("count").unwrap_or(&0.0)
            );
            println!(
                "  - Distancia promedio: {:.2} millas",
                stats.get("avg_distance").unwrap_or(&0.0)
            );
            println!(
                "  - Precio promedio: ${:.2}",
                stats.get("avg_amount").unwrap_or(&0.0)
            );
            println!(
                "  - Pasajeros promedio: {:.2}",
                stats.get("avg_passengers").unwrap_or(&0.0)
            );
        }

        "popular-dests" => {
            if args.len() < 4 {
                println!("Error: Faltan argumentos para el comando popular-dests");
                print_usage();
                return Ok(());
            }

            let csv_path = &args[2];
            let limit = args[3].parse::<usize>().unwrap_or(10);

            let start_time = Instant::now();
            println!("Obteniendo los {} destinos más populares...", limit);

            let popular_dests = data::filters::get_popular_destinations(csv_path, limit)?;

            let duration = start_time.elapsed();
            println!("Análisis completado en {:.2?}", duration);
            println!("\nDestinos más populares:");

            for (i, (dest, count)) in popular_dests.iter().enumerate() {
                println!("{}. Destino ID: {} - {} viajes", i + 1, dest, count);
            }
        }

        _ => {
            println!("Comando desconocido: {}", command);
            print_usage();
        }
    }

    Ok(())
}
