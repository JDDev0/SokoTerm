use std::fmt::Write as _;

#[cfg(test)]
mod tests;

pub fn number_to_string_leading_ascii(digits: u32, num: u32, leading_zeros: bool) -> String {
    if digits == 0 {
        panic!("Not enough digits");
    }else if digits > 9 {
        panic!("Too many digits");
    }

    let digits_10s = 10_u32.pow(digits);
    if num < digits_10s {
        return if leading_zeros {
            format!("{:01$}", num, digits as usize)
        } else {
            format!("{:1$}", num, digits as usize)
        }
    }

    let leading_digit = num / (digits_10s / 10) - 10;
    if leading_digit > 25 {
        panic!("Number too large");
    }

    let mut out = ((b'A' + leading_digit as u8) as char).to_string();

    if digits > 1 {
        let _ = write!(out, "{:01$}", num % (digits_10s / 10), digits as usize - 1);
    }

    out
}
