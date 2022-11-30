use std::process::Command;

fn main() {
    let _output = Command::new("cargo run --bin client 1")
        .spawn()
        .expect("failed to execute process");

    // println!("status: {}", output.status);
    // println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    // println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
}