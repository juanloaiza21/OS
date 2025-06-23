fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Buscador en un chilión de datos",
        options,
        Box::new(|_cc| Box::new(practica1::app::App::default())),

    )
}
