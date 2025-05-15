use fork::{fork , Fork};
fn main() {
    match fork() {
        Ok(Fork::Parent(child)) => {
            println!("I am the child process with PID: {}", child);
        }
        Ok(Fork::Child) => println!("I'm a new child process"), 
        Err(e) => {
            eprintln!("Fork failed: {}", e);
            std::process::exit(1);
        }
    }
}   
