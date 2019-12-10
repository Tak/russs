extern crate rand;
extern crate modulo;
extern crate num_bigint;
extern crate num_traits;

use std::convert::TryInto;

use rand::prelude::*;
use modulo::Mod;
use num_bigint::{BigInt, ToBigInt};
use num_traits::ToPrimitive;
use num_traits::identities::{Zero, One};

pub fn generate_string<T>(secret: &str, pieces_count: i32, required_pieces_count: i32, prime: i32, mut progress_callback: T) -> Vec<(i32, Vec<u8>)>
    where T: FnMut(f64) {
    let points = generate_buffer(secret, pieces_count, required_pieces_count, prime, progress_callback);
    return (0..points.len()).map(|index| {
        let mut buffer = Vec::new();
        for value in &points[index].1 {
            buffer.extend_from_slice(&value.to_le_bytes());
        }
        (points[index].0, buffer)
    }).collect();
}

pub fn generate_file<T>(secret_file_path: &str, pieces_count: i32, required_pieces_count: i32, prime: i32, mut progress_callback: T) -> Result<(), String>
    where T: FnMut(f64) {
    println!("TODO: generate file");
    return Result::Ok(());
}

pub fn interpolate_string<TPiecesCollection, TBytesCollection, TCallback>(pieces: &TPiecesCollection, prime: i32, mut progress_callback: TCallback) -> Result<String, String>
    where TCallback: FnMut(f64),
        TPiecesCollection: AsRef<[(i32, TBytesCollection)]>,
        TBytesCollection: AsRef<[u8]> {
    let mut my_pieces = pieces.as_ref();
    let buffers: Vec<(i32, Vec<i16>)> = my_pieces.iter().map(|piece| {
        let mut my_piece = piece.1.as_ref();
        let mut string: Vec<i16> = Vec::new();

        while !my_piece.is_empty() {
            let (chunk, remainder) = my_piece.split_at(std::mem::size_of::<i16>());
            my_piece = remainder;
            string.push(i16::from_le_bytes(chunk.try_into().unwrap()));
        }
        (piece.0, string)
    }).collect();

    let result = interpolate_buffer(&buffers, prime, progress_callback)?;
    return Ok(String::from_utf8(result).unwrap());
}

pub fn interpolate_file<T>(pieces: Vec<String>, destination: &str, mut progress_callback: Option<T>) -> Result<String, String>
    where T: FnMut(f64) {
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
    let my_coefficients: &[i32] = coefficients.as_ref();
    let mut pieces : Vec<(i32, i32)> = (0..(pieces_count + 1)).map(|x| {
        let mut sum = secret;
        for index in 0..my_coefficients.len() {
            sum += my_coefficients[index] * (x.pow(index as u32 + 1));
        }
        (x, Mod::modulo(sum, prime))
    }).collect();
    pieces.remove(0);
    return pieces;
}

// Solve for the 0th-order term of the lagrange polynomial partially described by points
// in the prime finite field for prime
fn  interpolate_secret<T>(points: &T, prime: i32) -> Result<i32, String>
    where T: AsRef<[(i32, i32)]> {
    let my_points: &[(i32, i32)] = points.as_ref();
    validate_points(&my_points, prime)?;

    let x_values : Vec<i32> = my_points.iter().map(|point| point.0).collect();
    let y_values : Vec<i32> = my_points.iter().map(|point| point.1).collect();

    let mut numerators: Vec<BigInt> = Vec::new();
    let mut denominators: Vec<BigInt> = Vec::new();

    for index in 0..x_values.len() {
        let mut other_x_values = x_values.clone();
        let this_x = other_x_values.remove(index);

        numerators.push(multiply_all(&other_x_values.iter().map(|x| 0 - *x).collect::<Vec<i32>>()));
        denominators.push(multiply_all(&other_x_values.iter().map(|x| this_x - *x).collect::<Vec<i32>>()));
    }

    let denominator = multiply_all(&denominators);
    let mut numerator = BigInt::zero();
    for index in 0..x_values.len() {
        numerator += divide_and_apply_modulus(&Mod::modulo(&numerators[index] * &denominator * &y_values[index].to_bigint().unwrap(), prime), &denominators[index], prime);
    }

    let result = Mod::modulo(divide_and_apply_modulus(&numerator, &denominator, prime) + prime, prime);
    return match result.to_i32() {
        None => Err(format!("Error interpolating secret: integer overflow for {}", result)),
        Some(value) => Ok(value),
    }
}

fn  divide_and_apply_modulus<T>(numerator: &T, denominator: &T, prime: i32) -> BigInt
    where T: Into<BigInt> + Clone {
    return numerator.clone().into() * modular_multiplicative_inverse(&denominator.clone().into(), &prime.to_bigint().unwrap()).0;
}

// https://en.wikipedia.org/wiki/Extended_Euclidean_algorithm
fn  modular_multiplicative_inverse<T>(a: &T, z: &T) -> (BigInt, BigInt)
    where T: Into<BigInt> + Clone {
    let mut x = BigInt::zero();
    let mut last_x = BigInt::one();
    let mut y = BigInt::one();
    let mut last_y = BigInt::zero();
    let mut a: BigInt = a.clone().into();
    let mut z: BigInt = z.clone().into();

    while z != BigInt::zero() {
        let integer_quotient = &a / &z;
        let new_a = z.clone();
        z = Mod::modulo(&a , &z);
        a = new_a;

        let new_x = &last_x - (&integer_quotient * &x);
        last_x = x;
        x = new_x;

        let new_y = &last_y - (&integer_quotient * &y);
        last_y = y;
        y = new_y;
    }

    return (last_x, last_y);
}

fn  multiply_all<TValues, TElement>(values: &TValues) -> BigInt
    where TValues: AsRef<[TElement]>,
        TElement: Into<BigInt> + Clone {
    let mut total = BigInt::one();
    let my_values: &[TElement] = values.as_ref();

    for value in my_values {
        total *= value.clone().into();
    }

    return total;
}

//# Generate the first piecesCount values for the polynomial for each byte in secret
fn generate_buffer<TSecret, TProgress>(secret: TSecret, total_pieces: i32, required_pieces: i32, prime: i32, mut progress_callback: TProgress) -> Vec<(i32, Vec<i16>)>
    where TSecret: AsRef<[u8]>,
        TProgress: FnMut(f64) {
    let mut result: Vec<(i32, Vec<i16>)> = (0..total_pieces).map(|index| (index + 1, Vec::new())).collect();
    let my_secret = secret.as_ref();
    let total_progress = my_secret.len() as f64;

    for i in 0..my_secret.len() {
        for point in generate_points(my_secret[i] as i32, total_pieces, &generate_coefficients(required_pieces, prime), prime) {
            result[point.0 as usize - 1].1.push(point.1 as i16)
        }
        progress_callback(i as f64 / total_progress);
    }

    return result;
}

fn  validate_points<T>(points: &T, prime: i32) -> Result<(), String>
    where T: AsRef<[(i32, i32)]> {
    let my_points: &[(i32, i32)] = points.as_ref();

    if my_points.len() < 2 {
        return Err(format!("Insufficient number of inputs ({})", my_points.len()));
    }
    if my_points.iter().any(|point| point.1 >= prime) {
        return Err(format!("Prime {} must be greater than all values {:?}", prime, my_points));
    }

    return Ok(());
}

fn validate_buffers<TContainer, TByteBuffer>(buffers: &TContainer) -> Result<(), String>
    where TContainer: AsRef<[(i32, TByteBuffer)]>,
        TByteBuffer: AsRef<[i16]> {
    let my_buffers = buffers.as_ref();
    let length = my_buffers[0].1.as_ref().len();

    return if buffers.as_ref().iter().all(|buffer| buffer.1.as_ref().len() == length) {
        Ok(())
    } else {
        Err(String::from("Differing buffer lengths"))
    }
}

//# Solve for each set of points in points and return an ordered array of solutions
fn interpolate_buffer<TContainer, TPointBuffer, TProgress>(points: &TContainer, prime: i32, mut progress_callback: TProgress) -> Result<Vec<u8>, String>
    where TContainer: AsRef<[(i32, TPointBuffer)]>,
        TPointBuffer: AsRef<[i16]>,
        TProgress: FnMut(f64) {
    let my_points = points.as_ref();
    validate_buffers(&my_points)?;

    let point_count = my_points[0].1.as_ref().len();
    let mut result: Vec<u8> = Vec::new();

    for i in 0..point_count {
        match interpolate_secret(&my_points.iter().map(|point| (point.0, point.1.as_ref()[i] as i32)).collect::<Vec<(i32, i32)>>(), prime) {
            Err(message) => return Err(message),
            Ok(value) => result.push(value as u8),
        }
        progress_callback(i as f64 / point_count as f64);
    }

    return Ok(result);
}



#[cfg(test)]
mod  tests {
    use super::*;

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

    //    it "validates single inputs" do
    #[test]
    fn  test_validate_points() {
        let prime = 5717;
        assert!(validate_points(&[(1, 1)], prime).is_err()); // Not enough points
        assert!(validate_points(&[(1, 50001), (2, 20000), (3, 30000)], prime).is_err()); // Prime too small for y-values
    }

    //        it "creates a product of inputs" do
    #[test]
    fn  test_multiply_all() {
        let empty_values: [i32; 0] = [];
        assert_eq!(multiply_all(&empty_values), BigInt::one());

        let test_data = [
            ([1, 2, 3], 6),
            ([2, -1, 2], -4),
            ([0, -43, 112], 0),
        ];

        for test_datum in &test_data {
            assert_eq!(multiply_all(&test_datum.0), test_datum.1.to_bigint().unwrap());
        }
    }

    //    it "calculates modular multiplicative inverse given known inputs" do
    #[test]
    fn test_modular_multiplicative_inverse() {
        let test_data = [
            ((-4, 3617), 904),
            ((-4, 7211), -1803),
        ];

        for test_datum in &test_data {
            let inverse = modular_multiplicative_inverse(&(test_datum.0).0.to_bigint().unwrap(), &(test_datum.0).1.to_bigint().unwrap()).0;
            assert_eq!(inverse, test_datum.1.to_bigint().unwrap());
            assert_eq!(Mod::modulo((test_datum.0).0 * inverse, (test_datum.0).1), BigInt::one());
        }
    }


    // TODO: Move to integration tests
    // It's not straightforward to do integration tests with an executable crate,
    // need to reorganize into a lib + an executable before that will be feasible

    //    it "successfully roundtrips a single random value" do
    #[test]
    fn test_interpolate_points() -> Result<(), String> {
        let prime = 5717;
        let secret = thread_rng().gen_range(0, prime);
        let number_of_pieces = 8;
        let required_pieces = 5;
        println!("Secret is {}", secret);


        let points = generate_points(secret, number_of_pieces, &generate_coefficients(required_pieces, prime), prime);
        for point in &points {
            assert!(point.1 < prime);
        }

        assert_eq!(interpolate_secret(&points, prime)?, secret);
        Ok(())
    }

//    it "generates expected point buffer given known inputs" do
//    # https://en.wikipedia.org/wiki/Shamir%27s_Secret_Sharing
    #[test]
    fn test_generate_buffer() {
        let secret = "1234";
        let total_pieces = 6;
        let required_pieces = 3;
        let prime = 1613;

        let pieces = generate_buffer(secret, total_pieces, required_pieces, prime, |_|{});

        assert_eq!(pieces.len(), total_pieces as usize);
    }


    //    it "validates buffers" do
    #[test]
    fn test_validate_buffers() {
        let secret: Vec<u8> = (1..32).map(|_| random()).collect();
        let prime = 5717;

        let mut buffers = generate_buffer(&secret, 5, 3, prime, |_|{});
        buffers[0].1.remove(1);

        assert!(interpolate_buffer(&buffers, prime, |_|{}).is_err());
    }

    fn roundtrip_buffer<TSecret, TProgress>(secret: &TSecret, mut progress_callback: TProgress) -> Result<Vec<u8>, String>
        where TSecret: AsRef<[u8]>,
            TProgress: FnMut(f64) {
        let total_pieces = 8;
        let required_pieces = 5;
        let prime = 5717;
        let mut last_progress: f64 = 0.0;

        let pieces = generate_buffer(secret, total_pieces, required_pieces, prime, |progress| {
            assert!(progress >= last_progress);
            last_progress = progress;
            progress_callback(progress);
        });

        last_progress = 0.0;
        return interpolate_buffer(&pieces, prime, |progress| {
            assert!(progress >= last_progress);
            last_progress = progress;
            progress_callback(progress);
        });
    }

    //    it "successfully roundtrips a random buffer" do
    #[test]
    fn test_roundtrip_buffer() {
        let secret: Vec<u8> = (0..32).map(|_| random::<u8>()).collect();
        let calculated_secret = roundtrip_buffer(&secret, |_|{}).unwrap();
        assert_eq!(secret, calculated_secret);
    }

    fn roundtrip_string<T>(secret: &str, progress_callback: T) -> Result<String, String>
        where T: Fn(f64) {
        let total_pieces = 8;
        let required_pieces = 5;
        let prime = 5717;
        let mut last_progress: f64 = 0.0;

        let pieces = generate_string(secret, total_pieces, required_pieces, prime, |progress| {
            assert!(progress >= last_progress);
            last_progress = progress;
            progress_callback(progress);
        });

        last_progress = 0.0;
        return interpolate_string(&pieces, prime, |progress| {
            assert!(progress >= last_progress);
            last_progress = progress;
            progress_callback(progress);
        });
    }

    //    it "successfully roundtrips a string" do
    #[test]
    fn test_roundtrip_string() {
        let secret: String = String::from("1234567890123456789012");
        let calculated_secret = roundtrip_string(secret.as_str(), |_|{}).unwrap();
        assert_eq!(secret, calculated_secret);
    }


}
