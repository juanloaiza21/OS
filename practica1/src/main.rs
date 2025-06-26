mod data;
mod visual;

fn main() {
    // Lanzar interfaz gráfica
    if let Err(e) = visual::run_app() {
        eprintln!("Error al iniciar la aplicación: {}", e);
    }
}
