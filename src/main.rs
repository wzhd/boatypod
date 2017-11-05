extern crate boatypod;

fn main() {
    if let Err(_) = boatypod::run() {
        eprintln!("Error.");
    }
}
