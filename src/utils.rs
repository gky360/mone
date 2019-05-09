use num::{Num, ToPrimitive};
use std::fmt;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct NumBytes<T: Num>(T);

impl<T: Num> From<T> for NumBytes<T> {
    fn from(val: T) -> NumBytes<T> {
        NumBytes(val)
    }
}

fn integer_len(v: f64) -> usize {
    let v = v.trunc() as u64;
    let mut p = 10;
    for l in 1.. {
        if v < p {
            return l;
        }
        p *= 10;
    }
    return 0;
}

impl<T: Num + ToPrimitive> NumBytes<T> {
    const UNITS: [&'static str; 9] = ["B", "KiB", "MiB", "GiB", "TiB", "Pib", "Eib", "ZiB", "YiB"];
    const VALUE_LEN: usize = 4;

    pub fn pretty(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut v: f64 = self.0.to_f64().ok_or(fmt::Error)?;
        for u in Self::UNITS.iter() {
            if v < 1024.0 {
                let ret = match integer_len(v) {
                    // with decimal points
                    l @ (1...2) => write!(
                        f,
                        "{:l$.r$} {:<3}",
                        v,
                        u,
                        l = l,
                        r = Self::VALUE_LEN - l - 1
                    ),
                    // without decimal points
                    3...4 => write!(f, "{:>4} {:<3}", v.trunc(), u),
                    _ => Err(fmt::Error),
                };
                return ret;
            }
            v = v / 1024.0;
        }
        Err(fmt::Error)
    }
}

impl<T: Num + ToPrimitive> fmt::Display for NumBytes<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.pretty(f)
    }
}

impl<T: Num + ToPrimitive> fmt::Debug for NumBytes<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.pretty(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_len() {
        assert_eq!(integer_len(573.0458984375), 3);
    }

    #[test]
    fn test_pretty() {
        assert_eq!(
            format!("{}", NumBytes::from((1.234 * 1024.0) as u64)),
            "1.23 KiB"
        );
        assert_eq!(format!("{}", NumBytes::from(1023)), "1023 B  ");
        assert_eq!(format!("{}", NumBytes::from(123)), " 123 B  ");
        assert_eq!(format!("{}", NumBytes::from(600_882_176)), " 573 MiB");
    }
}
