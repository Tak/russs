pub fn generate_string<T>(secret: &str, pieces_count: u32, required_pieces_count: u32, prime: u32, progress_callback: Option<T>) -> Result<Vec<Vec<u8>>, String>
    where T: Fn(f64) {
    println!("TODO: generate string");
    let mut pieces: Vec<Vec<u8>> = Vec::new();
    for i in 1..pieces_count {
        let mut piece: Vec<u8> = Vec::new();
        for u in 1..4 as u8 {
            piece.push(u);
        }
        pieces.push(piece);
    }
    return Result::Ok(pieces);
}

pub fn generate_file<T>(secret_file_path: &str, pieces_count: u32, required_pieces_count: u32, prime: u32, progress_callback: Option<T>) -> Result<(), String>
    where T: Fn(f64) {
    println!("TODO: generate file");
    return Result::Ok(());
}

pub fn interpolate_strings<T>(pieces: Vec<(i32, Vec<u8>)>, prime: i32, progress_callback: Option<T>) -> Result<Vec<u8>, String>
    where T: Fn(f64) {
                   return Result::Ok(pieces[0].1.clone());
                                                          }

pub fn interpolate_file<T>(pieces: Vec<String>, destination: &str, progress_callback: Option<T>) -> Result<String, String>
    where T: Fn(f64) {
    return Result::Ok(format!("{}/{}", destination, "secret.mp4"));
}
