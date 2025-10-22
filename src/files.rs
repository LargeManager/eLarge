use std::fs;

pub fn read_file(path: &str) -> Result<String, ()> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(e) => {
            println!("Failed to read file: {}", e);
            Err(())
        }
    }
}
