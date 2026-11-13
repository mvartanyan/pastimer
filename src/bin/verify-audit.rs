fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("Usage: cargo run --bin verify-audit -- audit.json")?;
    let audit = serde_json::from_slice(&std::fs::read(path)?)?;
    println!("{}", pastimer::audit::verify(&audit)?);
    Ok(())
}
