use std::time::Instant;
use fork::{fork , Fork}; // Para crear el proceo child
use nix::sys::wait::waitpid; // Para esperar el proceso hijo
use nix::unistd::{close, pipe, read, write}; // Para crear el pipe
use std::os::unix::io::RawFd; // Para usar los file descriptors
use std::{process, ptr};
use nix::unistd::Pid; // Para usar el pid

fn calcular_pi_leibniz_un_proceso(iteraciones: u64) -> f64 {
    let mut suma = 0.0;
    let mut signo = 1.0;

    for i in 0..iteraciones {
        let termino = signo / (2.0 * i as f64 + 1.0);
        suma += termino;
        signo *= -1.0; // Alternamos el signo en cada iteración
    }

    4.0 * suma // Multiplicamos por 4 para obtener π
}

//Usando pipes
fn calcular_pi_leibniz_4_procesos_pipelines(iteraciones: u64) -> f64 {
    //Calculamos el rango de cada proceso:
    let rango = iteraciones / 4;
    let counter_process0 = rango; //Rango 1 va de 0 a esta var
    let counter_process1 = rango * 2; //Rango 2 va de la var anterior a la proxima, asi sucesivamente
    let counter_process2 = rango * 3;
    let counter_process3 = iteraciones;
    
    //Creamos la variable de suma y signo
    let mut suma = 0.0;
    let mut signo = 1.0;

    //Creamos los procesos
    let (read_fd, write_fd) = pipe().unwrap(); // Creamos el pipe
    // Creamos el primer proceso
    let pid1 = fork::fork().unwrap();
    if pid1.is_child() {
        // Proceso hijo 1
        close(read_fd).unwrap(); // Cerramos el pipe de lectura
        for i in 0..counter_process0 {
            let mut aux = 0.0;
            let termino = signo / (2.0 * i as f64 + 1.0);
            aux += termino;
            signo *= -1.0; // Alternamos el signo en cada iteración
        }
        write(write_fd, &aux.to_ne_bytes()).unwrap(); // Enviamos la suma al pipe
        close(write_fd).unwrap(); // Cerramos el pipe de escritura
        process::exit(0); // Terminamos el proceso hijos
    }
    //Leemos el resultado del primer proceso
    let mut buffer = [0; 8]; // Buffer para leer el resultado
    read(read_fd, &mut buffer).unwrap(); // Leemos el resultado del pipe
    let mut aux = f64::from_ne_bytes(buffer); // Convertimos el resultado a f64
    waitpid(pid1.unwrap(), None).unwrap(); // Esperamos a que el proceso hijo termine
    close(read_fd).unwrap(); // Cerramos el pipe de lectura
    //Agregamos a la suma
    suma += aux; // Agregamos el resultado a la suma

    
    
    //Resultado
    suma * 4.0 // Multiplicamos por 4 para obtener π
}

fn main() {
    let iteraciones = 279_000_000;
    let mut inicio = Instant::now(); 
    let pi_aproximado = calcular_pi_leibniz_un_proceso(iteraciones);
    let mut fin = Instant::now();
    let mut total = fin.duration_since(inicio);
    println!("Aproximación de π después de {} iteraciones: {} \n", iteraciones, pi_aproximado);
    print!("Tiempo total de ejecucion sincrono: {} segundos o {} milis\n", total.as_secs(), total.as_millis());
}


