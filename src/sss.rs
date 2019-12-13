extern crate rand;
extern crate modulo;
extern crate num_bigint;
extern crate num_traits;

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::convert::TryInto;

use rand::prelude::*;
use modulo::Mod;
use num_bigint::{BigInt, ToBigInt};
use num_traits::ToPrimitive;
use num_traits::identities::{Zero, One};

pub const VERSION: i32 = 1;
const BUFFER_SIZE: usize = 8192;
const MAX_SECRET_FILENAME_LENGTH: usize = BUFFER_SIZE - 50;

#[allow(unused_mut)]
pub fn generate_string<TCollection, TProgress>(secret: &TCollection, pieces_count: i32, required_pieces_count: i32, prime: i32, mut progress_callback: TProgress) -> Vec<(i32, Vec<u8>)>
    where TCollection: AsRef<[u8]> + ?Sized,
        TProgress: FnMut(f64) {
    return generate_buffer(secret, pieces_count, required_pieces_count, prime, progress_callback).iter().map(|point| {
        (point.0, point.1.iter().map(|value| value.to_le_bytes().to_vec()).flatten().collect())
    }).collect();
}

fn open_file<P: AsRef<Path>>(path: P) -> Result<File, String> {
    return match File::open(&path) {
        Err(error) => Err(format!("Error opening {}: {}", path.as_ref().to_str().unwrap(), error)),
        Ok(file) => Ok(file),
    }
}

fn create_file<P: AsRef<Path>>(path: P) -> Result<File, String> {
    return match File::create(&path) {
        Err(error) => Err(format!("Error opening {}: {}", path.as_ref().to_str().unwrap(), error)),
        Ok(file) => Ok(file),
    }
}

fn get_file_size<P: AsRef<Path>>(path: P) -> Result<f64, String> {
    let my_path = path.as_ref();
    return match my_path.metadata() {
        Err(error) => Err(format!("Error getting file size for {}: {}", my_path.to_str().unwrap(), error)),
        Ok(metadata) => Ok(metadata.len() as f64),
    }
}

//# Process a secret file and generate an output file per piece
//# Format:
//# version\n          (text)
//# pieceIndex\n       (text)
//# prime\n            (text)
//# originalFilename\n (text)
//# raw binary data
pub fn generate_file<T>(secret_file_name: &str, pieces_count: i32, required_pieces_count: i32, prime: i32, mut progress_callback: T) -> Result<Vec<String>, String>
    where T: FnMut(f64) {
    let parse_error = format!("Error parsing file name: {}", secret_file_name);
    let secret_path = Path::new(secret_file_name);
    let secret_file = open_file(secret_file_name)?;
    let mut progress: f64 = 0.0;


    let basename: String;
    match secret_path.file_name() {
        None => return Err(parse_error),
        Some(path) => basename = String::from(path.to_str().unwrap()),
    }
    let total_progress = get_file_size(secret_file_name)?;

    let piece_names: Vec<PathBuf> = (0..pieces_count).map(|index| {
        secret_path.with_file_name(format!("{}-{}.shard", secret_path.file_stem().unwrap().to_str().unwrap(), index + 1).as_str())
    }).collect();

    let mut piece_files: Vec<File> = Vec::new();
    for path in &piece_names {
        piece_files.push(create_file(path)?);
    }

    // Actual writing begins here
    for index in 0..piece_files.len() {
        // Write header
        let file = &piece_files[index];
        write_file(file, &format!("{}\n", VERSION))?;
        write_file(file, &format!("{}\n", index + 1))?;
        write_file(file, &format!("{}\n", prime))?;
        write_file(file, &format!("{}\n", basename))?;
    }

    let mut buffer = [0 as u8; BUFFER_SIZE];
    let mut length: usize;

    loop {
        length = read_file(&secret_file, &mut buffer[..])?;
        if length == 0 {
            break;
        }

        let result = generate_string(&buffer[0..length], pieces_count, required_pieces_count, prime, |_|{});
        for index in 0..result.len() {
            // Write bodies
            write_file(&piece_files[index], &result[index].1)?;
        }
        progress += length as f64;
        progress_callback(progress / total_progress);
    }

    return Result::Ok(piece_names.iter().map(|path| String::from(path.to_str().unwrap())).collect());
}

fn read_file<T>(mut file: &File, data: &mut T) -> Result<usize, String>
    where T: AsMut<[u8]> + ?Sized {
    return match file.read(data.as_mut()) {
        Err(error) => Err(format!("Error reading file: {}", error)),
        Ok(read) => Ok(read),
    }
}

fn write_file<T>(mut file: &File, data: &T) -> Result<usize, String>
    where T: AsRef<[u8]> + ?Sized {
    return match file.write(data.as_ref()) {
        Err(error) => Err(format!("Error writing file: {}", error)),
        Ok(written) => Ok(written),
    }
}

fn validate_piece_files<T>(piece_files: &T) -> Result<(), String>
    where T: AsRef<[String]> {
    let files = piece_files.as_ref();
    let length= get_file_size(&files[0])?;
    for file in &files[1..] {
        if get_file_size(file)? != length {
            return Err(format!("Mismatching file lengths: {} vs. {}", length, get_file_size(file)?));
        }
    }

    return Ok(());
}

fn validate_header<TInts, TStrings>(versions: &TInts, indices: &TInts, primes: &TInts, buffers: &Vec<[u8; BUFFER_SIZE]>, filenames: &TStrings) -> Result<(), String>
    where TInts: AsRef<[i32]> + ?Sized,
        TStrings: AsRef<[String]> + ?Sized,
        {
    // TODO: More detailed error messages

    if versions.as_ref().iter().any(|version| *version != VERSION) {
        return Err(String::from("Invalid versions for input files"));
    }

    let my_indices = indices.as_ref();
    if (1..my_indices.len()).any(|i| my_indices[i..].iter().any(|value| *value == my_indices[i - 1])) {
        return Err(String::from("Duplicate indices in input files"));
    }

    let my_primes = primes.as_ref();
    let prime = my_primes[0];
    if my_primes.iter().any(|value| *value != prime) {
        return Err(String::from("Differing primes in input files"));
    }

    let my_filenames = filenames.as_ref();
    let filename = &my_filenames[0];
    if filename.len() > MAX_SECRET_FILENAME_LENGTH {
        return Err(format!("Original filenames are too long: {}", filename.len()));
    }
    if my_filenames.iter().any(|value| value != filename) {
        return Err(String::from("Differing filenames in input files"));
    }

    if buffers.iter().any(|buffer| buffer.as_ref().len() % 2 != 0) {
        return Err(String::from("Input buffer has invalid (odd) length"));
    }

    if buffers.len() != my_indices.len() {
        return Err(String::from("Internal error reading header"));
    }

    return Ok(());
}

pub fn read_headers<T>(pieces: &T) -> Result<(i32, String, Vec<i32>, Vec<[u8; BUFFER_SIZE]>, usize), String>
    where T: AsRef<[File]> {

    let mut indices: Vec<i32> = Vec::new();
    let mut filenames: Vec<String> = Vec::new();
    let mut primes: Vec<i32> = Vec::new();
    let mut versions: Vec<i32> = Vec::new();
    let mut buffers: Vec<[u8; BUFFER_SIZE]> = Vec::new();
    let mut buffer_length = 0 as usize;

    for piece in pieces.as_ref() {
        let mut header = [0 as u8; BUFFER_SIZE];
        read_file(&piece, &mut header[..])?;

        let headers: Vec<&[u8]> = header.splitn(5, |byte| *byte == '\n' as u8).collect();
        if headers.len() < 5 {
            return Err(String::from("Malformed header in input file"));
        }
        match String::from_utf8_lossy(headers[0]).into_owned().parse::<i32>() {
            Err(error) => return Err(format!("Error parsing header version: {}", error)),
            Ok(version) => versions.push(version),
        }
        match String::from_utf8_lossy(headers[1]).into_owned().parse::<i32>() {
            Err(error) => return Err(format!("Error parsing header version: {}", error)),
            Ok(index) => indices.push(index),
        }
        match String::from_utf8_lossy(headers[2]).into_owned().parse::<i32>() {
            Err(error) => return Err(format!("Error parsing header prime: {}", error)),
            Ok(prime) => primes.push(prime),
        }
        filenames.push(String::from_utf8_lossy(headers[3]).into_owned());

        let mut buffer = [0 as u8; BUFFER_SIZE];
        buffer[0..headers[4].len()].copy_from_slice(headers[4]);
        let read = read_file(&piece, &mut buffer[headers[4].len()..])?;
        if buffer_length == 0 {
            buffer_length = headers[4].len() + read;
        } else if headers[4].len() + read != buffer_length {
            return Err(format!("Mismatched buffer sizes in input files"));
        }
        buffers.push(buffer);
    }
    validate_header(&versions, &indices, &primes, &buffers, &filenames)?;

    return Ok((primes[0], filenames[0].clone(), indices, buffers, buffer_length));
}

fn binary_buffer_to_points<T>(buffer: &T) -> Vec<i16>
    where T: AsRef<[u8]> + ?Sized {
    let my_buffer = buffer.as_ref();
    return (0..(my_buffer.len() / 2)).map(|input_index| {
        let buffer_index = input_index * 2;
        i16::from_le_bytes(my_buffer[buffer_index..(buffer_index + 2)].try_into().unwrap())
    }).collect();
}

#[allow(unused_mut)]
pub fn interpolate_string<TPiecesCollection, TBytesCollection, TCallback>(pieces: &TPiecesCollection, prime: i32, mut progress_callback: TCallback) -> Result<String, String>
    where TCallback: FnMut(f64),
        TPiecesCollection: AsRef<[(i32, TBytesCollection)]> + ?Sized,
        TBytesCollection: AsRef<[u8]> {
    let point_buffers: Vec<(i32, Vec<i16>)> = pieces.as_ref().iter().map(|piece| {
        (piece.0, binary_buffer_to_points(&piece.1))
    }).collect();
    let result = interpolate_buffer(&point_buffers, prime, progress_callback)?;
    return Ok(String::from_utf8(result).unwrap());
}

//# Solve for each value encoded in a set of files and write a file built from the solution
//# See generate_file for format
pub fn interpolate_file<T, TProgress>(pieces: &T, destination: &str, mut progress_callback: TProgress) -> Result<String, String>
    where T: AsRef<[String]> + ?Sized,
        TProgress: FnMut(f64) {
    let mut files: Vec<File> = Vec::new();
    let my_pieces = pieces.as_ref();
    validate_piece_files(&my_pieces)?;
    for piece in my_pieces {
        files.push(open_file(piece)?);
    }
    let total_progress = get_file_size(&my_pieces[0])?;
    let mut progress = 0.0;

    let (prime, output_filename, indices, mut buffers, buffer_length) = read_headers(&files)?;

    let destination_path = Path::new(destination).join(&output_filename);
    let output_file = create_file(&destination_path)?;
    let mut end_of_file = false;
    let mut read = buffer_length;

    while !end_of_file {
        let point_buffers: Vec<(i32, Vec<i16>)> = indices.iter().map(|x| *x).zip(buffers.iter().map(|buffer| {
            binary_buffer_to_points(&buffer[0..read])
        })).collect();
        let result = interpolate_buffer(&point_buffers, prime, |_|{})?;
        write_file(&output_file, &result)?;
        progress += read as f64;
        progress_callback(progress / total_progress);

        for index in 0..files.len() {
            read = read_file(&files[index], &mut buffers[index][..])?;
            if read == 0 {
                end_of_file = true;
            }
        }
    }

    return Ok(String::from(destination_path.as_os_str().to_str().unwrap()));
}

//    Generate (requiredPiecesCount - 1) polynomial coefficients less than prime
fn  generate_coefficients(required_pieces_count: i32, prime: i32) -> Vec<i32> {
    return (1..required_pieces_count).map(|_|
        rand::thread_rng().gen_range(0, prime)
    ).collect();
}

// Generate the first pieces_count points on the polynomial described by coefficients
fn  generate_points<T>(secret: i32, pieces_count: i32, coefficients: &T, prime: i32) -> Vec<(i32, i32)>
    where T: AsRef<[i32]> + ?Sized {
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
    where T: AsRef<[(i32, i32)]> + ?Sized {
    let my_points: &[(i32, i32)] = points.as_ref();
    validate_points(&my_points, prime)?;

    let x_values : Vec<i32> = my_points.iter().map(|point| point.0).collect();
    let y_values : Vec<i32> = my_points.iter().map(|point| point.1).collect();
    let prime_long = prime as i64;
    let prime_big = prime.to_bigint().unwrap();

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
        numerator += divide_and_apply_modulus(&Mod::modulo(&numerators[index] * &denominator * &y_values[index].to_bigint().unwrap(), &prime_big), &denominators[index], &prime_big);
    }

    let result = Mod::modulo(divide_and_apply_modulus(&numerator, &denominator, &prime_big) + prime_long, prime_long);
    return match result.to_i32() {
        None => Err(format!("Error interpolating secret: integer overflow for {}", result)),
        Some(value) => Ok(value),
    }
}

fn  divide_and_apply_modulus(numerator: &BigInt, denominator: &BigInt, prime: &BigInt) -> i64 {
    return (numerator * modular_multiplicative_inverse(denominator, prime).0).to_i64().unwrap();
}

// https://en.wikipedia.org/wiki/Extended_Euclidean_algorithm
fn  modular_multiplicative_inverse<T>(a: &T, z: &T) -> (i64, i64)
    where T: Into<BigInt> + Clone {
    let mut x: i64 = 0;
    let mut last_x: i64 = 1;
    let mut y: i64 = 1;
    let mut last_y: i64 = 0;
    let mut a: BigInt = a.clone().into();
    let mut z: BigInt = z.clone().into();

    while z != BigInt::zero() {
        let integer_quotient: i64 = (&a / &z).to_i64().unwrap();
        let new_a = z;
        z = Mod::modulo(&a , &new_a);
        a = new_a;

        let new_x = last_x - (integer_quotient * x);
        last_x = x;
        x = new_x;

        let new_y = last_y - (integer_quotient * y);
        last_y = y;
        y = new_y;
    }

    return (last_x.into(), last_y.into());
}

fn  multiply_all<TValues, TElement>(values: &TValues) -> BigInt
    where TValues: AsRef<[TElement]> + ?Sized ,
        TElement: Into<BigInt> + Clone {
    let mut total = BigInt::one();
    let my_values: &[TElement] = values.as_ref();

    for value in my_values {
        total *= value.clone().into();
    }

    return total;
}

//# Generate the first piecesCount values for the polynomial for each byte in secret
fn generate_buffer<TSecret, TProgress>(secret: &TSecret, total_pieces: i32, required_pieces: i32, prime: i32, mut progress_callback: TProgress) -> Vec<(i32, Vec<i16>)>
    where TSecret: AsRef<[u8]> + ?Sized,
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
    where T: AsRef<[(i32, i32)]> + ?Sized {
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
    where TContainer: AsRef<[(i32, TByteBuffer)]> + ?Sized,
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
    where TContainer: AsRef<[(i32, TPointBuffer)]> + ?Sized,
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
            assert_eq!(inverse, test_datum.1 as i64);
            assert_eq!(Mod::modulo((test_datum.0).0 * inverse, (test_datum.0).1), 1);
        }
    }

    // TODO: Move to integration tests
    // It's not straightforward to do integration tests with an executable crate,
    // need to reorganize into a lib + an executable before that will be feasible

    fn choose_n_from<T>(source: &Vec<T>, n: usize) -> Vec<T>
        where T: Clone {
        let mut source_copy = source.clone();
        return (0..n).map(|_| source_copy.remove(thread_rng().gen_range(0, source_copy.len())).clone()).collect();
    }

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

        assert_eq!(interpolate_secret(&choose_n_from(&points, required_pieces as usize), prime)?, secret);
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
        where TSecret: AsRef<[u8]> + ?Sized,
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
        return interpolate_buffer(&choose_n_from(&pieces, required_pieces as usize), prime, |progress| {
            assert!(progress >= last_progress);
            last_progress = progress;
            progress_callback(progress);
        });
    }

    //    it "reports progress for buffers" do
    #[test]
    fn test_report_progress_buffers() {
        let secret = (0..32).map(|_| random::<u8>()).collect::<Vec<u8>>();
        let mut progress_callbacks = 0;
        roundtrip_buffer(&secret, |_| progress_callbacks += 1).unwrap();
        assert_eq!(progress_callbacks, 2 * secret.len());
    }

    //    it "successfully roundtrips a random buffer" do
    #[test]
    fn test_roundtrip_buffer() {
        let secret: Vec<u8> = (0..32).map(|_| random::<u8>()).collect();
        let calculated_secret = roundtrip_buffer(&secret, |_|{}).unwrap();
        assert_eq!(secret, calculated_secret);
    }

    fn roundtrip_string<T>(secret: &str, mut progress_callback: T) -> Result<String, String>
        where T: FnMut(f64) {
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
        return interpolate_string(&choose_n_from(&pieces, required_pieces as usize), prime, |progress| {
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

    //    it "reports progress for strings" do
    #[test]
    fn test_report_progress_string() {
        let secret: String = String::from("1234567890123456789012");
        let mut progress_callbacks = 0;
        roundtrip_string(secret.as_str(), |_| progress_callbacks += 1).unwrap();
        assert_eq!(progress_callbacks, secret.len() * 2);
    }

    //    it "validates files" do
    #[test]
    fn test_validate_file() {
        let destination = Path::new(file!()).parent().unwrap().parent().unwrap().join("tests").join("data");
        let total_pieces = 8;
        let required_pieces = 5;
        let prime = 5717;
        let input = destination.join("testInput");

        let pieces = generate_file(input.to_str().unwrap(), total_pieces, required_pieces, prime, |_|{}).unwrap();

        let test_data = [
            String::from(input.with_file_name("testInput-differingPrime.shard").to_str().unwrap()),
            String::from(input.with_file_name("testInput-differingVersion.shard").to_str().unwrap()),
            String::from(input.with_file_name("testInput-differingFilename.shard").to_str().unwrap()),
            String::from(input.with_file_name("testInput-invalidFilename.shard").to_str().unwrap()),
            pieces[0].clone(),
        ];

        for test_datum in &test_data {
            let mut test_pieces = pieces.clone();
            test_pieces.push(test_datum.clone());
            assert!(interpolate_file(&test_pieces, destination.to_str().unwrap(), |_|{}).is_err());
        }
    }

    //    it "successfully roundtrips a file" do
    #[test]
    fn test_roundtrip_file() {
        let destination = Path::new(file!()).parent().unwrap().parent().unwrap().join("tests").join("data");
        let input = destination.join("testInput");
        let output = input.with_file_name("testOutput");
        let total_pieces = 8;
        let required_pieces = 5;
        let prime = 5717;
        let mut progress_callbacks = 0;

        // Because the shards preserve the original file path, we copy the input file to the expected output path
        std::fs::copy(&input, &output).unwrap();

        let pieces = generate_file(output.to_str().unwrap(), total_pieces, required_pieces, prime, |_| progress_callbacks += 1).unwrap();
        for piece in &pieces {
            assert!(Path::new(&piece).exists());
        }

        std::fs::remove_file(&output).unwrap();
        assert!(!output.exists());
        assert!(progress_callbacks > 0);
        progress_callbacks = 0;

        let result = interpolate_file(&choose_n_from(&pieces, required_pieces as usize), destination.to_str().unwrap(), |_| progress_callbacks += 1).unwrap();
        assert_eq!(result.as_str(), output.to_str().unwrap());

        let mut input_data: Vec<u8> = Vec::new();
        let mut output_data: Vec<u8> = Vec::new();
        File::open(&input).unwrap().read_to_end(&mut input_data).unwrap();
        File::open(&output).unwrap().read_to_end(&mut output_data).unwrap();
        assert_eq!(input_data, output_data);
        assert!(progress_callbacks > 0);
    }
}
