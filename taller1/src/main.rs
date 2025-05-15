use fork::{fork , Fork}; // Para crear el proceo child
use nix::sys::wait::waitpid; // Para esperar el proceso hijo
use nix::unistd::{close, pipe, read, write}; // Para crear el pipe
use std::os::unix::io::RawFd; // Para usar los file descriptors
use std::{process, ptr};
use nix::unistd::Pid; // Para usar el pid

fn main() {
    let (read_fd,write_fd) = pipe().expect("No se pudo crear el pipe"); //Crea el pipe
    match fork() {
        //Proceso hijo
        Ok(Fork::Child) => {
            // Proceso hijo: suma de 500 a 999
            close(read_fd).expect("No se pudo cerrar el pipe de lectura"); //Cierra extremo de
            //lectura en el proceso hijo
            let sum_child: i64 = (500..1000).sum();
            println! ("Suma del hijo: {}", sum_child);

            //Envia la suma al padre
            let bytes = sum_child.to_ne_bytes(); //Convierte la suma a bytes
            write(write_fd, &bytes).expect("Error al escribir en el pipe");
            close(write_fd).expect("No se pudo cerrar el pipe de escritura"); //Cierra extremo del
            //pipe de escritura
            
            // Termina el proceso hijo
            process::exit(0);
        },
        Ok(Fork::Parent(child_pid)) => {
            // Proceso padre: suma de 0 a 499
            close(write_fd).expect("No se pudo cerrar el pipe de escritura"); //Cierra extremo de
            //escritura en el proceso padre
            let sum_parent: i64 = (0..500).sum();
            println! ("Suma del padre: {}", sum_parent);

            let pid = Pid::from_raw(child_pid); //Convierte el pid a Pid
            waitpid(Some(pid), None).expect("Error al esperar el proceso hijo"); //Espera el
         
            //Leer el   resultado del hijo
            let mut buffer = [0; 8]; //Buffer para leer la suma del hijo
            read(read_fd, &mut buffer).expect("Error al leer del pipe"); //Lee del pipe
            let sum_child = i64::from_ne_bytes(buffer); //Convierte los bytes a i64
            close(read_fd).expect("No se pudo cerrar el pipe de lectura"); //Cierra extremo de lectura
            //Sumar ambos procesos
            let total_sum = sum_parent + sum_child;
            println! ("Suma total: {}", total_sum); //Imprime la suma total
        },
        Err(_) => {
            eprintln!("Error al crear el fork");
            process::exit(1);
        }
    }
}
