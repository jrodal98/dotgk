use dotgk::settings; fn main() { match settings::load_settings() { Ok(s) => println!("Settings loaded: {:?}", s), Err(e) => println!("Error: {:?}", e) } }
