extern crate rand;

use rand::prelude::*;

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

pub fn interpolate_strings<TPiecesCollection, TBytesCollection, TCallback>(pieces: &TPiecesCollection, prime: i32, progress_callback: Option<TCallback>) -> Result<Vec<u8>, String>
    where TCallback: Fn(f64),
        TPiecesCollection: AsRef<[(i32, TBytesCollection)]>,
        TBytesCollection: AsRef<[u8]> {
                   return Result::Ok(Vec::new());
}

pub fn interpolate_file<T>(pieces: Vec<String>, destination: &str, progress_callback: Option<T>) -> Result<String, String>
    where T: Fn(f64) {
    return Result::Ok(format!("{}/{}", destination, "secret.mp4"));
}

//    Generate (requiredPiecesCount - 1) polynomial coefficients less than prime
fn  generate_coefficients(required_pieces_count: i32, prime: i32) -> Vec<i32> {
    return (1..required_pieces_count).map(|_|
        rand::thread_rng().gen_range(0, prime)
    ).collect();
}

// Generate the first pieces_count points on the polynomial described by coefficients
fn  generate_points<T>(secret: i32, pieces_count: i32, coefficients: &T, prime: i32) -> Vec<(i32, i32)>
    where T: AsRef<[i32]> {
    let my_coefficients: &[i32] = coefficients.as_ref().into();
    let mut pieces : Vec<(i32, i32)> = (0..(pieces_count + 1)).map(|x| {
        let mut sum = secret;
        for index in 0..my_coefficients.len() {
            sum += my_coefficients[index] * (x.pow(index as u32 + 1));
        }
        (x, sum % prime)
    }).collect();
    pieces.remove(0);
    return pieces;
}



#[cfg(test)]
mod  tests {
    use super::*;
    use std::borrow::Borrow;

    //    it "generates n-1 coefficients, all less than prime" do
    #[test]
    fn  test_generate_coefficients() {
        let required_pieces = 6;
        let prime = 1613;
        let coefficients = generate_coefficients(required_pieces, prime);

        assert_eq!(coefficients.len() as i32, required_pieces - 1);
        for coefficient in coefficients {
            assert!(coefficient < prime);
        }
    }

    //    it "generates expected points given known inputs" do
    //    https://en.wikipedia.org/wiki/Shamir%27s_Secret_Sharing
    #[test]
    fn  test_generate_points() {
        let secret = 1234;
        let number_of_pieces = 6;
        let coefficients = [166, 94];
        let prime = 1613;
        let expected_y_values = [1494, 329, 965, 176, 1188, 775];

        let points = generate_points(secret, number_of_pieces, &coefficients, prime);

        assert_eq!(points.len(), 6);
        for index in 0..points.len() {
            assert_eq!(points[index].0, index as i32 + 1);
            assert_eq!(points[index].1, expected_y_values[index]);
        }
    }

}
