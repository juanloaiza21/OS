use std::time::Instant;
use nix::sys::wait::waitpid;
use nix::unistd::{fork, Fork, pipe, read, write, close};
use std::process;
use std::os::fd::RawFd;


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

    let mut pipes = Vec::new();
    for _ in 0..4 {}
        let (read_fd, write_fd) = pipe().unwrap();
        pipes.push((read_fd, write_fd));
    }

    for i in 0..4 {
        match unsafe {fork()} {
        Ok(Fork::Child) => {
            println!(""Soy el hijo {} con PID {}", i, process::id()");
            
            //Cerramos pipes que no estan en uso en este momento:
            for j in 0..4 {
                if j != i {
                    close(pipes[j].0).unwrap(); // Cerramos el extremo de lectura
                    close(pipes[j].1).unwrap(); // Cerramos el extremo de escritura
                }
            }

            close(pipes[i].0).unwrap(); // Cerramos el extremo de lectura

            //Cada hijo hace un trabajo diferente segun el indiced
            match i {
                0 => {
                    let mut aux = 0.0;
                    for j in 0..counter_process0 {
                        let termino = signo / (2.0 * j as f64 + 1.0);
                        aux += termino;
                        signo *= -1.0; // Alternamos el signo en cada iteración
                    }
                    write(pipes[i].1, &aux.to_ne_bytes()).unwrap(); // Enviamos el resultado al
                    // padre
                },
                1 => {
                    let mut aux = 0.0;
                    for j in counter_process0..counter_process1 {
                        let termino = signo / (2.0 * j as f64 + 1.0);
                        aux += termino;
                        signo *= -1.0; // Alternamos el signo en cada iteración
                    }
                    write(pipes[i].1, &aux.to_ne_bytes()).unwrap(); // Enviamos el resultado al padre
                    
                },
                2 => {
                    let mut aux = 0.0;
                    for j in counter_process1..counter_process2 {
                        let termino = signo / (2.0 * j as f64 + 1.0);
                        aux += termino;
                        signo *= -1.0; // Alternamos el signo en cada iteración
                    }
                    write(pipes[i].1, &aux.to_ne_bytes()).unwrap(); // Enviamos el resultado al padre
                },
                3 => {
                    let mut aux = 0.0;
                    for j in counter_process2..counter_process3 {
                        let termino = signo / (2.0 * j as f64 + 1.0);
                        aux += termino;
                        signo *= -1.0; // Alternamos el signo en cada iteración
                    }
                    write(pipes[i].1, &aux.to_ne_bytes()).unwrap(); // Enviamos el resultado al
                    // padre
                },
                _ => unreachable!(),
            }
            process::exit(0); // Salimos del proceso hijo
        },
        Ok(Fork::Parent { child, .. }) => {
                println!("Padre: creé el hijo {} con PID {:?}", i, child);
                child_pids.push(child);
        },
        Err(_) => {
            println!("Error al crear el proceso hijo");
            process::exit(1);
        }
    }
    for i in 0..4 {
        close(pipes[i].1).unwrap();
    }

    //Realizar la suma de los resultados de los hijos
    for i in 0..4 {
        let mut buffer = [0u8; 8]; // Buffer para leer el resultado
        read(pipes[i].0, &mut buffer).unwrap(); // Leemos el resultado del hijo
        let aux = f64::from_ne_bytes(buffer); // Convertimos el resultado a f64
        suma += aux; // Sumamos el resultado al total
    }
    //Multiplicamos por 4 para obtener π
    suma * 4.0
    
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


