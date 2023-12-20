//! Attempts to identify how similar two given strings are.
//! Currently uses Levenshtein.
#[derive(PartialEq)]
pub enum Similarity {
    Identical(f64),
    Similar(f64),
    Dissimilar,
}

// https://stackoverflow.com/questions/49037111
trait InRange {
    fn in_range(self, begin: Self, end: Self) -> bool;
}

impl InRange for f64 {
    fn in_range(self, begin: f64, end: f64) -> bool {
        self >= begin && self < end
    }
}

/// Adapted from <https://github.com/wooorm/levenshtein-rs>
pub fn edit_distance(a: &str, b: &str) -> Similarity {
    let mut result = 0;

    if a == b {
        return Similarity::Identical(1.);
    }

    let length_a = a.chars().count();
    let length_b = b.chars().count();

    if length_a == 0 || length_b == 0 {
        return Similarity::Dissimilar
    }

    let mut cache: Vec<usize> = (1..).take(length_a).collect();
    let mut distance_a;
    let mut distance_b;

    /* Loop. */
    for (index_b, code_b) in b.chars().enumerate() {
        result = index_b;
        distance_a = index_b;

        for (index_a, code_a) in a.chars().enumerate() {
            distance_b = if code_a == code_b {
                distance_a
            } else {
                distance_a + 1
            };

            distance_a = cache[index_a];

            result = if distance_a > result {
                if distance_b > result {
                    result + 1
                } else {
                    distance_b
                }
            } else if distance_b > distance_a {
                distance_a + 1
            } else {
                distance_b
            };

            cache[index_a] = result;
        }
    }

    #[allow(clippy::cast_precision_loss)]
    let dist = result as f64 / length_a as f64;

    if dist.in_range(0., 0.2) {
        Similarity::Similar(dist)
    } else {
        Similarity::Dissimilar
    }
}