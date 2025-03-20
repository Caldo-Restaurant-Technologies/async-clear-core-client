pub mod controller;
mod interface;
pub mod io;
pub mod motor;
mod send_recv;

pub(crate) fn num_to_bytes<T: ToString>(number: T) -> Vec<u8> {
    number.to_string().chars().map(|c| c as u8).collect()
}

pub(crate) fn ascii_to_int(bytes: &[u8]) -> isize {
    let sign = if bytes[0] == 45 { -1 } else { 1 };
    let int = bytes
        .iter()
        .filter(|&&x| (48..=57).contains(&x))
        .fold(0, |mut acc, x| {
            let num = x - 48;
            acc *= 10;
            acc += num as isize;
            acc
        });
    int * sign
}

pub(crate) fn int_to_byte(number: u8) -> u8 {
    number + 48
}
