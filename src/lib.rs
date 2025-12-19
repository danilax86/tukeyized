/// Values that can be filtered using Tukey's range test.
///
/// # Usage
///
/// ```
/// use tukeyized::Tukey;
/// let values = [1.0, 6.0, 3.0, 8888.0, 3.0, 2.0, 8.0, -19292.0];
/// let filtered = values.tukeyize();
/// assert_eq!(filtered, vec![1.0, 6.0, 3.0, 3.0, 2.0, 8.0]);
/// ```
pub trait Tukey {
    /// Removes extreme values using Tukey's method.
    ///
    /// Values outside the inclusive range `[Q1 - 1.5 * IQR, Q3 + 1.5 * IQR]` are removed.
    fn tukeyize(&self) -> Vec<f64>;
}

impl Tukey for [f64] {
    /// Removes extreme values using Tukey's method.
    fn tukeyize(&self) -> Vec<f64> {
        trim(self)
    }
}

impl Tukey for Vec<f64> {
    /// Removes extreme values using Tukey's method.
    fn tukeyize(&self) -> Vec<f64> {
        self.as_slice().tukeyize()
    }
}

/// Removes extreme values using Tukey's method.
///
/// The quartiles are computed as medians of the lower and upper halves of the sorted data.
fn trim(values: &[f64]) -> Vec<f64> {
    if values.len() < 3 {
        return values.to_vec();
    }
    let mut order = values.to_vec();
    order.sort_by(|a, b| {
        a.partial_cmp(b).unwrap_or_else(|| {
            panic!("Cannot compare values {a} and {b} because at least one is NaN")
        })
    });
    let (q1, q3) = hinge(&order);
    let range = q3 - q1;
    let min = q1 - (1.5 * range);
    let max = q3 + (1.5 * range);
    values
        .iter()
        .copied()
        .filter(|x| *x >= min && *x <= max)
        .collect()
}

/// Calculates Tukey-style quartiles from already-sorted values.
fn hinge(values: &[f64]) -> (f64, f64) {
    let mid = values.len() / 2;
    if values.len().is_multiple_of(2){
        return (middle(&values[..mid]), middle(&values[mid..]));
    }
    (middle(&values[..mid]), middle(&values[mid + 1..]))
}

/// Calculates the median of already-sorted values.
fn middle(values: &[f64]) -> f64 {
    if values.is_empty() {
        panic!("Cannot calculate a median of an empty array");
    }
    let mid = values.len() / 2;
    if values.len() % 2 == 1 {
        return values[mid];
    }
    (values[mid - 1] + values[mid]) / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn roll(state: &mut u64) -> f64 {
        *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let bits = (*state >> 11) | 0x3ff0000000000000;
        f64::from_bits(bits) - 1.0
    }

    fn make(seed: &str) -> Vec<f64> {
        let mut state = seed
            .bytes()
            .fold(0u64, |s, b| s.wrapping_add(u64::from(b)).wrapping_mul(257));
        let mut values = Vec::new();
        for _ in 0..(seed.chars().count() * 29) {
            values.push((roll(&mut state) * 2000.0) - 1000.0);
        }
        let peak = (roll(&mut state) + 1.0) * 1_000_000.0;
        let pit = -peak;
        values.push(peak);
        values.push(pit);
        values
    }

    #[test]
    fn array_removes_extreme_values_using_tukey_method() {
        let values = make("test");
        let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let min = values.iter().copied().fold(f64::INFINITY, f64::min);
        let result = values.tukeyize();
        assert!(
            !result.contains(&max) && !result.contains(&min),
            "The outliers were not removed"
        );
    }

    #[test]
    fn array_keeps_all_values_when_interquartile_range_is_zero() {
        let mut state = "áßç"
            .bytes()
            .fold(0u64, |s, b| s.wrapping_add(u64::from(b)).wrapping_mul(31));
        let item = (roll(&mut state) * 1000.0) - 500.0;
        let values = vec![item, item, item, item, item];
        let result = values.tukeyize();
        assert_eq!(
            result, values,
            "The values were removed even though there were no outliers"
        );
    }

    #[test]
    fn array_returns_the_original_values_when_there_are_too_few_elements() {
        let values = "a".bytes().map(f64::from).collect::<Vec<f64>>();
        let result = values.tukeyize();
        assert_eq!(
            result, values,
            "The values were changed even though there were too few elements"
        );
    }

    #[test]
    fn array_produces_the_same_result_when_called_concurrently() {
        let values = make("somerandomseed");
        let model = values.tukeyize();
        let left = values.clone();
        let right = values.clone();
        let one = thread::spawn(move || left.tukeyize());
        let two = thread::spawn(move || right.tukeyize());
        let a = one.join().unwrap();
        let b = two.join().unwrap();
        assert!(
            a == model && b == model,
            "The concurrent calls produced different results"
        );
    }
}
