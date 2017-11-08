extern crate boatypod;

fn main() {
    if let Err(e) = boatypod::run() {
        eprintln!("Error: {}", e.description());
    }
}
