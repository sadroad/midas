use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("tailwindcss")
        .args(["-i", "src/main.css", "-o", "assets/output.css", "--minify"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
    match status {
        Ok(s) => {
            if !s.success() {
                eprintln!("TailwindCSS failed to build: {}", s);
            }
        }
        Err(e) => {
            eprintln!("Failed to execute TailwindCSS: {}", e);
        }
    }
    Ok(())
}
