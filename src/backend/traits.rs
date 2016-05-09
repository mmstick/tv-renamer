use std::error::Error;
use std::io::{self, Write};
use std::iter;
use std::process;

/// A trait that adds the ability for numbers to find their digit count and to convert them to padded strings.
pub trait Digits {
    /// Counts the number of digits in a number. **Example:** {{0 = 0}, {1 = 1}, {10 = 2}, {100 = 3}}
    fn digits(&self) -> usize;

    /// Converts a number into a padded String, using `pad` as the character to pad with and `limit` as the size.
    fn to_padded_string(&self, pad: char, limit: usize) -> String;
}

impl Digits for usize {
    fn digits(&self) -> usize {
        let mut digits = if *self == 1 || *self % 10 == 0 { 1 } else { 0 };
        let mut temp = 1;
        while temp < *self {
            digits += 1;
            temp = (temp << 3) + (temp << 1);
        }
        digits
    }

    fn to_padded_string(&self, pad: char, limit: usize) -> String {
        if self.digits() < limit {
            let mut output = String::with_capacity(limit);
            output.push_str(&iter::repeat(pad).take(limit - self.digits()).collect::<String>());
            output.push_str(&self.to_string());
            output
        } else {
            self.to_string()
        }
    }
}

pub trait Try {
    type Succ;

    /// Unwrap or abort program with an exit status of 1.
    fn try(self, message: &[u8], stderr: &mut io::Stderr) -> Self::Succ;
}

pub trait TryAndIgnore {
    type Succ;

    /// Print an error message and ignore the error.
    fn try_and_ignore(self, message: &[u8], stderr: &mut io::Stderr) -> Self::Succ;
}

impl<T, U: Error> Try for Result<T, U> {
    type Succ = T;

    fn try(self, message: &[u8], stderr: &mut io::Stderr) -> T {
        self.unwrap_or_else(|e| {
            let mut stderr = stderr.lock();
            let _ = stderr.write(b"tv-renamer: ");
            let _ = stderr.write(message);
            let _ = stderr.write(e.description().as_bytes());
            let _ = stderr.write(b"\n");
            let _ = stderr.flush();
            process::exit(1);
        })
    }
}

impl<T, U: Error> TryAndIgnore for Result<T, U> {
    type Succ = T;

    fn try_and_ignore(self, message: &[u8], stderr: &mut io::Stderr) -> T {
        self.unwrap_or_else(|_| {
            let mut stderr = stderr.lock();
            let _ = stderr.write(b"tv-renamer: ");
            let _ = stderr.write(message);
            let _ = stderr.write(b"\n");
            let _ = stderr.flush();
            process::exit(1);
        })
    }
}
impl<T> Try for Option<T> {
    type Succ = T;

    fn try(self, message: &[u8], stderr: &mut io::Stderr) -> T {
        self.unwrap_or_else(|| {
            let mut stderr = stderr.lock();
            let _ = stderr.write(b"tv-renamer: ");
            let _ = stderr.write(message);
            let _ = stderr.write(b"\n");
            let _ = stderr.flush();
            process::exit(1);
        })
    }
}

#[test]
fn test_digits() {
    assert_eq!(1.digits(), 1);
    assert_eq!(9.digits(), 1);
    assert_eq!(10.digits(), 2);
    assert_eq!(100.digits(), 3);
}
#[test]
fn test_padded_digits() {
    assert_eq!(  5.to_padded_string('0', 2).as_str(), "05");
    assert_eq!( 10.to_padded_string('0', 2).as_str(), "10");
    assert_eq!(100.to_padded_string('0', 2).as_str(), "100");
    assert_eq!( 10.to_padded_string('0', 3).as_str(), "010");
    assert_eq!(  5.to_padded_string('0', 3).as_str(), "005");
}
