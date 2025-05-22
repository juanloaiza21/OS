use std::time::Instant;
use nix::sys::wait::waitpid;
use nix::unistd::{pipe, read, write, close};
use fork::{fork, Fork}; // Fix: Import Fork from the fork module
use std::process;
use std::os::fd::RawFd;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::mem;
use nix::unistd::{ Pid};
use nix::sys::wait::waitpid;
use std::sync::atomic::{AtomicU32, Ordering};
use std::process;
use shared_memory::ShmemConf;


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

    // Usamos Option<OwnedFd> para poder extraer los descriptores de manera segura
    let mut pipes = Vec::new();
    for _ in 0..4 {
        let (read_fd, write_fd) = pipe().unwrap();
        pipes.push((Some(read_fd), Some(write_fd)));
    }

    // Define child_pids vector
    let mut child_pids = Vec::new();

    for i in 0..4 {
        match unsafe {fork()} {
        Ok(Fork::Child) => {
            println!("Soy el hijo {} con PID {}", i, process::id());
            
            //Cerramos pipes que no estan en uso en este momento:
            for j in 0..4 {
                if j != i {
                    // Extraemos el descriptor de lectura y lo cerramos si existe
                    if let Some(fd_read) = pipes[j].0.take() {
                        close(fd_read).unwrap();
                    }
                    
                    // Extraemos el descriptor de escritura y lo cerramos si existe
                    if let Some(fd_write) = pipes[j].1.take() {
                        close(fd_write).unwrap();
                    }
                }
            }

            // Cerramos el extremo de lectura del pipe actual
            if let Some(fd_read) = pipes[i].0.take() {
                close(fd_read).unwrap();
            }

            //Cada hijo hace un trabajo diferente segun el indice
            match i {
                0 => {
                    let mut aux = 0.0;
                    for j in 0..counter_process0 {
                        let termino = signo / (2.0 * j as f64 + 1.0);
                        aux += termino;
                        signo *= -1.0; // Alternamos el signo en cada iteración
                    }
                    
                    // Extraemos el descriptor de escritura si existe
                    if let Some(fd_write) = pipes[i].1.take() {
                        write(fd_write, &aux.to_ne_bytes()).unwrap();
                    }
                },
                1 => {
                    let mut aux = 0.0;
                    for j in counter_process0..counter_process1 {
                        let termino = signo / (2.0 * j as f64 + 1.0);
                        aux += termino;
                        signo *= -1.0; // Alternamos el signo en cada iteración
                    }
                    
                    // Extraemos el descriptor de escritura si existe
                    if let Some(fd_write) = pipes[i].1.take() {
                        write(fd_write, &aux.to_ne_bytes()).unwrap();
                    }
                },
                2 => {
                    let mut aux = 0.0;
                    for j in counter_process1..counter_process2 {
                        let termino = signo / (2.0 * j as f64 + 1.0);
                        aux += termino;
                        signo *= -1.0; // Alternamos el signo en cada iteración
                    }
                    
                    // Extraemos el descriptor de escritura si existe
                    if let Some(fd_write) = pipes[i].1.take() {
                        write(fd_write, &aux.to_ne_bytes()).unwrap();
                    }
                },
                3 => {
                    let mut aux = 0.0;
                    for j in counter_process2..counter_process3 {
                        let termino = signo / (2.0 * j as f64 + 1.0);
                        aux += termino;
                        signo *= -1.0; // Alternamos el signo en cada iteración
                    }
                    
                    // Extraemos el descriptor de escritura si existe
                    if let Some(fd_write) = pipes[i].1.take() {
                        write(fd_write, &aux.to_ne_bytes()).unwrap();
                    }
                },
                _ => unreachable!(),
            }
            process::exit(0); // Salimos del proceso hijo
        },
        Ok(Fork::Parent(child)) => {
                println!("Padre: creé el hijo {} con PID {:?}", i, child);
                child_pids.push(child);
        },
        Err(_) => {
            println!("Error al crear el proceso hijo");
            process::exit(1);
        }
        }
    }
    
    for i in 0..4 {
        // Cerramos los extremos de escritura en el padre
        if let Some(fd_write) = pipes[i].1.take() {
            close(fd_write).unwrap();
        }
    }

    //Realizar la suma de los resultados de los hijos
    for i in 0..4 {
        let mut buffer = [0u8; 8]; // Buffer para leer el resultado
        
        // Extraemos el descriptor de lectura si existe
        if let Some(fd_read) = pipes[i].0.take() {
            read(fd_read, &mut buffer).unwrap();
            let aux = f64::from_ne_bytes(buffer); // Convertimos el resultado a f64
            suma += aux; // Sumamos el resultado al total
        }
    }
    //Multiplicamos por 4 para obtener π
    suma * 4.0
}


fn calcular_pi_leibniz_4_procesos_shmem(iteraciones: u64) -> f64 {
    
    // Calculamos el rango para cada proceso
    let rango = iteraciones / 4;
    let counter_process0 = rango;
    let counter_process1 = rango * 2;
    let counter_process2 = rango * 3;
    let counter_process3 = iteraciones;
    
    // Creamos la región de memoria compartida
    let shmem = ShmemConf::new()
        .size(4 * std::mem::size_of::<f64>() + std::mem::size_of::<AtomicU32>())
        .os_id("pi_calculation")
        .create()
        .expect("No se pudo crear la memoria compartida");
    
    // Puntero a la memoria compartida
    let ptr = shmem.as_ptr();
    
    // Ahora obtenemos un puntero al contador atómico
    let counter_offset = 4 * std::mem::size_of::<f64>();
    let counter_ptr = unsafe { ptr.add(counter_offset) as *mut AtomicU32 };
    
    // Inicializamos el contador atómico
    unsafe { std::ptr::write(counter_ptr, AtomicU32::new(0)) };
    
    // Vector para almacenar los PIDs de los hijos
    let mut child_pids = Vec::new();
    
    // Creamos 4 procesos
    for i in 0..4 {
        match unsafe { fork() } {
            Ok(Fork::Child) => {
                println!("Soy el hijo {} con PID {}", i, process::id());
                
                let mut suma = 0.0;
                let mut signo = 1.0;
                
                // Calculamos el signo inicial para este proceso
                if i > 0 {
                    match i {
                        1 => {
                            if counter_process0 % 2 == 1 {
                                signo = -1.0;
                            }
                        },
                        2 => {
                            if counter_process1 % 2 == 1 {
                                signo = -1.0;
                            }
                        },
                        3 => {
                            if counter_process2 % 2 == 1 {
                                signo = -1.0;
                            }
                        },
                        _ => {}
                    }
                }
                
                // Calculamos la parte correspondiente a este proceso
                match i {
                    0 => {
                        for j in 0..counter_process0 {
                            let termino = signo / (2.0 * j as f64 + 1.0);
                            suma += termino;
                            signo *= -1.0;
                        }
                    },
                    1 => {
                        for j in counter_process0..counter_process1 {
                            let termino = signo / (2.0 * j as f64 + 1.0);
                            suma += termino;
                            signo *= -1.0;
                        }
                    },
                    2 => {
                        for j in counter_process1..counter_process2 {
                            let termino = signo / (2.0 * j as f64 + 1.0);
                            suma += termino;
                            signo *= -1.0;
                        }
                    },
                    3 => {
                        for j in counter_process2..counter_process3 {
                            let termino = signo / (2.0 * j as f64 + 1.0);
                            suma += termino;
                            signo *= -1.0;
                        }
                    },
                    _ => unreachable!(),
                }
                
                // Escribimos el resultado en la memoria compartida
                let result_ptr = unsafe { ptr.add(i * std::mem::size_of::<f64>()) as *mut f64 };
                unsafe { std::ptr::write(result_ptr, suma) };
                
                // Incrementamos el contador atómico
                let counter = unsafe { &*counter_ptr };
                counter.fetch_add(1, Ordering::SeqCst);
                
                // Salimos del proceso hijo
                process::exit(0);
            },
            Ok(Fork::Parent(child)) => {
                println!("Padre: creé el hijo {} con PID {:?}", i, child);
                // Aquí es donde está el error: child ya es un Pid de nix, así que lo guardamos así
                child_pids.push(child);
            },
            Err(_) => {
                println!("Error al crear el proceso hijo");
                process::exit(1);
            }
        }
    }
    
    // Esperamos a que todos los hijos terminen
    for pid in child_pids {
        // Aquí usamos directamente el pid que ya es de tipo Pid
        waitpid(pid, None).expect("Error al esperar por el hijo");
    }
    
    // Leemos y sumamos los resultados
    let mut suma_total = 0.0;
    for i in 0..4 {
        let result_ptr = unsafe { ptr.add(i * std::mem::size_of::<f64>()) as *const f64 };
        let resultado = unsafe { std::ptr::read(result_ptr) };
        suma_total += resultado;
    }
    
    // Multiplicamos por 4 para obtener pi
    4.0 * suma_total
}

fn main() {
    let iteraciones = 279_000_000;
    let mut inicio = Instant::now(); 
    let pi_aproximado1 = calcular_pi_leibniz_un_proceso(iteraciones);
    let mut fin = Instant::now();
    let mut total = fin.duration_since(inicio);
    println!("Aproximación de π después de {} iteraciones: {} \n", iteraciones, pi_aproximado1);
    print!("Tiempo total de ejecucion sincrono: {} segundos o {} milis\n", total.as_secs(), total.as_millis());
    println!("--------------------------------------------------");
    inicio = Instant::now();
    let pi_aproximado2 = calcular_pi_leibniz_4_procesos_pipelines(iteraciones);
    fin = Instant::now();
    total = fin.duration_since(inicio);
    println!("Aproximación de π después de {} iteraciones: {} \n", iteraciones, pi_aproximado2);
    print!("Tiempo total de ejecucion asincrono (pipelines): {} segundos o {} milis\n", total.as_secs(), total.as_millis());
    println!("--------------------------------------------------");
}
