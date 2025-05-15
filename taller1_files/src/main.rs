use fork::{fork, Fork};
use nix::sys::wait::waitpid;
use nix::unistd::Pid;
use std::fs::{File, remove_file};
use std::io::{Read, Write};
use std::path::Path;
use std::process;

fn main() {
    // Definir el nombre del archivo temporal para la comunicación
    let file_path = "sum_result.tmp";
    
    match fork() {
        Ok(Fork::Child) => {
            // Proceso hijo: suma del 500 al 999
            
            // Calcular la suma del rango 500-999
            let sum_child: i64 = (500..1000).sum();
            println!("Hijo: La suma de 500 a 999 es {}", sum_child);
            
            // Escribir el resultado en un archivo
            let mut file = File::create(file_path)
                .expect("Error al crear el archivo temporal");
                
            // Convertir el número a bytes y escribir en el archivo
            let buf = sum_child.to_ne_bytes();
            file.write_all(&buf)
                .expect("Error al escribir el resultado en el archivo");
                
            // Asegurar que los datos se escriban al disco
            file.flush()
                .expect("Error al vaciar el buffer del archivo");
            
            // Salir del proceso hijo
            process::exit(0);
        },
        Ok(Fork::Parent(child_pid)) => {
            // Proceso padre: suma del 0 al 499
            
            // Calcular la suma del rango 0-499
            let sum_parent: i64 = (0..500).sum();
            println!("Padre: La suma de 0 a 499 es {}", sum_parent);
            
            // Esperar a que el hijo termine
            let pid = Pid::from_raw(child_pid);
            waitpid(Some(pid), None)
                .expect("Error al esperar al proceso hijo");
            
            // Leer el resultado del proceso hijo desde el archivo
            let mut file = File::open(file_path)
                .expect("Error al abrir el archivo temporal");
                
            let mut buf = [0; 8]; // 8 bytes para un i64
            file.read_exact(&mut buf)
                .expect("Error al leer el resultado del archivo");
                
            let sum_child = i64::from_ne_bytes(buf);
            
            // Eliminar el archivo temporal
            if Path::new(file_path).exists() {
                remove_file(file_path).expect("No se pudo eliminar el archivo temporal");
            }
            
            // Sumar ambos resultados
            let total = sum_parent + sum_child;
            println!("Total: La suma de 0 a 999 es {}", total);
        },
        Err(e) => {
            eprintln!("Error al hacer fork: {}", e);
            process::exit(1);
        }
    }
}
