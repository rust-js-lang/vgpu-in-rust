//! IEEE-754 binary32 to binary16 conversion used by WGSL host packing.
pub fn float32_to_float16(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = (bits >> 16) & 0x8000;
    let mantissa = bits & 0x007fffff;
    let exponent = (bits >> 23) & 0xff;
    if exponent == 0xff {
        return (sign | if mantissa != 0 { 0x7e00 } else { 0x7c00 }) as u16;
    }
    let half_exponent = exponent as i32 - 127 + 15;
    if half_exponent >= 0x1f {
        return (sign | 0x7c00) as u16;
    }
    if half_exponent <= 0 {
        if half_exponent < -10 {
            return sign as u16;
        }
        return (sign | round_even(mantissa | 0x00800000, (14 - half_exponent) as u32)) as u16;
    }
    let rounded = round_even(mantissa, 13);
    if rounded == 0x0400 {
        let exponent = half_exponent + 1;
        return (sign
            | if exponent >= 0x1f {
                0x7c00
            } else {
                (exponent as u32) << 10
            }) as u16;
    }
    (sign | ((half_exponent as u32) << 10) | rounded) as u16
}
fn round_even(value: u32, shift: u32) -> u32 {
    let truncated = value >> shift;
    let remainder = value & ((1 << shift) - 1);
    let halfway = 1 << (shift - 1);
    if remainder > halfway || (remainder == halfway && truncated & 1 != 0) {
        truncated + 1
    } else {
        truncated
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ieee_half_specials_subnormals_and_rounding_match_the_binary_contract() {
        for (value, bits) in [
            (0.0, 0x0000),
            (-0.0, 0x8000),
            (1.0, 0x3c00),
            (-2.0, 0xc000),
            (65504.0, 0x7bff),
            (65520.0, 0x7c00),
            (f32::INFINITY, 0x7c00),
            (f32::NEG_INFINITY, 0xfc00),
            (2.0_f32.powi(-24), 1),
            (2.0_f32.powi(-25), 0),
            (1.0004883, 0x3c00),
            (1.0014648, 0x3c02),
        ] {
            assert_eq!(float32_to_float16(value), bits);
        }
        assert_eq!(float32_to_float16(f32::NAN) & 0x7fff, 0x7e00);
    }
}
