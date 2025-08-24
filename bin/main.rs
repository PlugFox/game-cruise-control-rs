// Check is windows, if not - show error message and close program
#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("This program currently supports only on Windows.");
    std::process::exit(1);
}

#[cfg(target_os = "windows")]
fn main() {
    println!("Press Page Up to start cruise control. WASD to stop.");
}
