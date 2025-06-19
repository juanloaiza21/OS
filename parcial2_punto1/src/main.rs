use shared_memory::ShmemConf;
use std::time::Instant;
use std::{process, result};
use std::os::fd::{RawFd, AsRawFd, FromRawFd, OwnedFd};
use std::mem;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;

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

//fn calcular_pi_leibniz_1_hilo_(iteraciones: u64) -> f64 {

    //Leibnz
  //  let handle = thread::spawn(move || {
    //    calcular_pi_leibniz_un_proceso(iteraciones);
 
    //});

    //handle.join().unwrap() as f64

//}

fn calcular_pi_leibniz_2_hilos(iteraciones: u64) -> f64 {
    let rango = iteraciones / 2;
    
    let mut handles = Vec::new();
    
    let rangos = [
        (0, rango),             
        (rango, iteraciones)   
    ];
    
    for (inicio, fin) in rangos {
        let signo_inicial = if inicio % 2 == 0 { 1.0 } else { -1.0 };
        
        let handle = thread::spawn(move || {
            let mut suma = 0.0;
            let mut signo = signo_inicial;
            
            for j in inicio..fin {
                let termino = signo / (2.0 * j as f64 + 1.0);
                suma += termino;
                signo *= -1.0;
            }
            
            suma
        });
        
        handles.push(handle);
    }
    
    let mut suma_total = 0.0;
    for handle in handles {
        suma_total += handle.join().unwrap();
    }
    4.0 * suma_total
}

fn calcular_pi_leibniz_4_hilo_(iteraciones: u64) -> f64{
    let range = iteraciones / 4;
    let range1 = range * 2;
    let range2 = range * 3;
    
    let mut handles = Vec::new();
    
    let rangos = [
        (0, range),
        (range, range1),
        (range1, range2),            
        (range2, iteraciones)   
    ];
    
    for (inicio, fin) in rangos {
        let signo_inicial = if inicio % 2 == 0 { 1.0 } else { -1.0 };
        
        let handle = thread::spawn(move || {
            let mut suma = 0.0;
            let mut signo = signo_inicial;
            
            for j in inicio..fin {
                let termino = signo / (2.0 * j as f64 + 1.0);
                suma += termino;
                signo *= -1.0;
            }
            
            suma
        });
        
        handles.push(handle);
    }
    
    let mut suma_total = 0.0;
    for handle in handles {
        suma_total += handle.join().unwrap();
    }
    4.0 * suma_total

}

fn calcular_pi_leibniz_8_hilo_(iteraciones: u64) -> f64{
    let range = iteraciones / 8;
    let range1 = range * 2;
    let range2 = range * 3;
    let range3 = range * 4;
    let range4 = range * 5;
    let range5 = range * 6;
    let range6 = range * 7;
    
    let mut handles = Vec::new();
    
    let rangos = [
        (0, range),
        (range, range1),
        (range1, range2),
        (range2, range3),
        (range3, range4),
        (range4, range5),
        (range5, range6),           
        (range6, iteraciones)   
    ];
    
    for (inicio, fin) in rangos {
        let signo_inicial = if inicio % 2 == 0 { 1.0 } else { -1.0 };
        
        let handle = thread::spawn(move || {
            let mut suma = 0.0;
            let mut signo = signo_inicial;
            
            for j in inicio..fin {
                let termino = signo / (2.0 * j as f64 + 1.0);
                suma += termino;
                signo *= -1.0;
            }
            
            suma
        });
        
        handles.push(handle);
    }
    
    let mut suma_total = 0.0;
    for handle in handles {
        suma_total += handle.join().unwrap();
    }
    4.0 * suma_total

}
   


fn main () {
    let iteraciones = 4_000_000_000;
    let mut inicio = Instant::now(); 
    let mut pi_aproximado1 = calcular_pi_leibniz_un_proceso(iteraciones);
    let mut fin = Instant::now();
    let mut total = fin.duration_since(inicio);
    println!("--------------------------------------------------");
    println!("                 Tiempo base                       ");
    println!("Aproximación de π después de {} iteraciones: {} \n", iteraciones, pi_aproximado1);
    print!("Tiempo total de ejecucion sincrono: {} segundos o {} milis\n", total.as_secs(), total.as_millis());
    println!("--------------------------------------------------");

    //1 hilo
    //inicio = Instant::now();
    //pi_aproximado1 = calcular_pi_leibniz_1_hilo_(iteraciones);
    println!("--------------------------------------------------");
    println!("                 1 hilo                      ");
    println!("Aproximación de π después de {} iteraciones: {} \n", iteraciones, pi_aproximado1);
    print!("Tiempo total de ejecucion sincrono: {} segundos o {} milis\n", total.as_secs(), total.as_millis());
    println!("--------------------------------------------------");
    fin = Instant::now();   

    //2 hilo
    inicio = Instant::now();
    pi_aproximado1 = calcular_pi_leibniz_2_hilos(iteraciones);
    fin = Instant::now();   
    total = fin.duration_since(inicio);
    println!("--------------------------------------------------");
    println!("                 2 hilos                     ");
    println!("Aproximación de π después de {} iteraciones: {} \n", iteraciones, pi_aproximado1);
    print!("Tiempo total de ejecucion sincrono: {} segundos o {} milis\n", total.as_secs(), total.as_millis());
    println!("--------------------------------------------------");

    

    //4 hilos 
    inicio = Instant::now();
    pi_aproximado1 = calcular_pi_leibniz_4_hilo_(iteraciones);
    fin = Instant::now();   
    total = fin.duration_since(inicio);
    println!("--------------------------------------------------");
    println!("                 4 hilos                     ");
    println!("Aproximación de π después de {} iteraciones: {} \n", iteraciones, pi_aproximado1);
    print!("Tiempo total de ejecucion sincrono: {} segundos o {} milis\n", total.as_secs(), total.as_millis());
    println!("--------------------------------------------------");

    //8 hilos 
    inicio = Instant::now();
    pi_aproximado1 = calcular_pi_leibniz_8_hilo_(iteraciones);
    fin = Instant::now();   
    total = fin.duration_since(inicio);
    println!("--------------------------------------------------");
    println!("                 8 hilos                     ");
    println!("Aproximación de π después de {} iteraciones: {} \n", iteraciones, pi_aproximado1);
    print!("Tiempo total de ejecucion sincrono: {} segundos o {} milis\n", total.as_secs(), total.as_millis());
    println!("--------------------------------------------------");
}
