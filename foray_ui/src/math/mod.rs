pub mod round_nice;
mod vector;
pub use vector::Point;
pub use vector::Vector;

fn n_in_range(start: f32, stop: f32, delta: f32) -> i32 {
    ((stop - start).abs() / delta).ceil() as i32
}

/// create a vector of linearly spaced values in the range start..=stop, seperated by the given
/// delta
pub fn linspace_delta(start: f32, stop: f32, delta: f32) -> Vec<f32> {
    let n = n_in_range(start, stop, delta);
    if start.is_nan() || stop.is_nan() || delta.is_nan() {
        panic!("Encountered nan!{:?}", (start, stop, delta))
    }

    let dir_delta = (stop - start).signum() * delta;
    (0..=n)
        .map(|i| start + i as f32 * dir_delta)
        .filter(|v| match (stop - start).is_sign_positive() {
            true => *v <= stop,
            false => *v >= stop,
        })
        .collect()
}

/// create a vector of linearly spaced values in the range start..=stop
pub fn linspace(start: f32, stop: f32, num: i32) -> Vec<f32> {
    if start.is_nan() || stop.is_nan() {
        panic!("Encountered nan!{:?}", (start, stop, num))
    }
    (0..num)
        .map(|i| i as f32 / (num - 1).max(1) as f32)
        .map(|c| start * (1. - c) + c * stop)
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn range() {
        assert_eq!(n_in_range(0., 1., 0.25), 4);
        assert_eq!(n_in_range(0., 10., 6.), 2);
        assert_eq!(n_in_range(0., 60., 30.), 2);
        assert_eq!(n_in_range(0., 1., 2.), 1);
        assert_eq!(n_in_range(0., 1., 50.), 1);
    }
    #[test]
    fn linspace_by_num() {
        let pos = linspace(0., 2., 3);
        assert_eq!(pos, vec![0., 1.0, 2.0]);
        let neg = linspace(0., -2., 3);
        assert_eq!(neg, vec![0., -1., -2.]);
        let neg2 = linspace(-2., 0., 3);
        assert_eq!(neg2, vec![-2., -1., 0.]);
        let one = linspace(0., 1., 1);
        assert_eq!(one, vec![0.]);
    }
    #[test]
    fn linspace_by_delta() {
        let pos = linspace_delta(0., 2., 1.0);
        assert_eq!(pos, vec![0., 1.0, 2.0]);
        let neg = linspace_delta(0., -2., 1.0);
        assert_eq!(neg, vec![0., -1., -2.]);
        let neg2 = linspace_delta(-2., 0., 1.0);
        assert_eq!(neg2, vec![-2., -1., 0.]);
        let one = linspace_delta(0., 1., 50.);
        assert_eq!(one, vec![0.]);
        let unequal = linspace_delta(0., 100., 30.);
        assert_eq!(unequal, vec![0., 30., 60., 90.]);
    }
}
