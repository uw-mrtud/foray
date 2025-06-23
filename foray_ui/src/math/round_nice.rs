/// Find the nearest "nice" number.
///
/// Rounded *away* from zero
///
/// nice numbers:
/// 1x10^n,
/// 2x10^n,
/// 5x10^n
/// where n is all integers
pub fn round_nice(x: f32) -> f32 {
    let nice = 10.0f32.powf(x.abs().log10().ceil());

    let scale = if x.abs() <= 0.2 * nice {
        0.2
    } else if x.abs() <= 0.5 * nice {
        0.5
    } else {
        1.0
    };
    x.signum() * scale * nice
}

#[cfg(test)]
mod test {
    use super::*;
    use float_cmp::approx_eq;

    fn approx_compare(a: &[f32], b: &[f32]) {
        a.iter().copied().map(round_nice).zip(b).for_each(|(a, b)| {
            assert!(approx_eq!(f32, a, *b, ulps = 2));
        });
    }

    #[test]
    fn nice_10() {
        approx_compare(&[10.1, 10.0, 9.9], &[20.0, 10.0, 10.0]);
        approx_compare(&[-10.1, -10.0, -9.9], &[-20.0, -10.0, -10.0]);
    }
    #[test]
    fn nice_5() {
        approx_compare(&[5.1, 5.0, 4.9], &[10.0, 5.0, 5.0]);
        approx_compare(&[-5.1, -5.0, -4.9], &[-10.0, -5.0, -5.0]);
    }
    #[test]
    fn nice_2() {
        approx_compare(&[2.1, 2.0, 1.9], &[5.0, 2.0, 2.0]);
        approx_compare(&[-2.1, -2.0, -1.9], &[-5.0, -2.0, -2.0]);
    }
    #[test]
    fn nice_01() {
        approx_compare(&[0.011, 0.01, 0.0099], &[0.02, 0.01, 0.01]);
        approx_compare(&[-0.011, -0.01, -0.0099], &[-0.02, -0.01, -0.01]);
    }
}
