use std::error::Error;

fn digest_input() -> Result<u8, Box<dyn Error>> {
    Ok(65u8)
}


fn main() -> Result<(), Box<dyn Error>>{
    let result = digest_input()?;
    println!("{}",result);
    Ok(())
}
