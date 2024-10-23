use std::{cmp::Ordering::Equal, fmt::Display};

use getset::CopyGetters;
use num::{Float, Integer};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Debug, Clone, Default, CopyGetters)]
pub struct Statistics<T: Float + PartialOrd> {
    #[getset(get_copy = "pub")]
    mean: T,
    #[getset(get_copy = "pub")]
    median: T,
    #[getset(get_copy = "pub")]
    standard_deviation: T,
    #[getset(get_copy = "pub")]
    population_variance: T,
    #[getset(get_copy = "pub")]
    population_standard_deviation: T,
    #[getset(get_copy = "pub")]
    variance: T,
    #[getset(get_copy = "pub")]
    minimum: T,
    #[getset(get_copy = "pub")]
    maximum: T,
}

impl<T: Float + Send + Sync + PartialOrd> Statistics<T> {
    fn median_fn(values: &[T]) -> T {
        let mut v = Vec::from(values);
        v.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(Equal));
        match v.len().is_even() {
            true => (v[v.len() / 2 - 1] + v[v.len() / 2]) / num::cast(2.0).unwrap(),
            false => v[v.len() / 2],
        }
    }
    
    pub fn par_statistics_of(values: &[T]) -> Option<Self> {
        if values.is_empty() {
            return None;
        }
        
        let mean = statistical::mean(values);
        let mut median = T::zero();
        let mut standard_deviation = T::zero();
        let mut population_variance = T::zero();
        let mut population_standard_deviation = T::zero();
        let mut variance = T::zero();
        let mut minimum = T::max_value();
        let mut maximum = T::min_value();
        rayon::scope(|s| {
            s.spawn(|_| median = Self::median_fn(values));
            s.spawn(|_| standard_deviation = statistical::standard_deviation(values, Some(mean)));
            s.spawn(|_| population_variance = statistical::population_variance(values, Some(mean)));
            s.spawn(|_| population_standard_deviation = statistical::population_standard_deviation(values, Some(mean)));
            s.spawn(|_| variance = statistical::variance(values, Some(mean)));
            s.spawn(|_| values.into_par_iter().min_by(|a, b| a.partial_cmp(b).unwrap_or(Equal)).unwrap().clone_into(&mut minimum));
            s.spawn(|_| values.into_par_iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(Equal)).unwrap().clone_into(&mut maximum));
        });

        Some(Self {
            mean,
            median,
            standard_deviation,
            population_variance,
            population_standard_deviation,
            variance,
            minimum,
            maximum,
        })
    }
}

// impl<T: Float + PartialOrd> Statistics<T> {
//     pub fn statistics_of(values: &[T]) -> Option<Self> {
//         if values.is_empty() {
//             return None;
//         }

//         let mean = statistical::mean(values);
        
//         Some(Self {
//             mean,
//             median: statistical::median(values),
//             standard_deviation: statistical::standard_deviation(values, Some(mean)),
//             population_variance: statistical::population_variance(values, Some(mean)),
//             population_standard_deviation: statistical::population_standard_deviation(values, Some(mean)),
//             variance: statistical::variance(values, Some(mean)),
//             minimum: values.into_iter().min_by(|a, b| a.partial_cmp(b).unwrap_or(Equal)).unwrap().to_owned(),
//             maximum: values.into_iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(Equal)).unwrap().to_owned(),
//         })
//     }
// }

impl<T: Float + PartialOrd + Display> std::fmt::Display for Statistics<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, r#"Mean: {}
Median: {}
Standard deviation: {}
Population variance: {}
Population standard deviation: {}
Variance: {}
Minimum: {}
Maximum: {}
"#,
        self.mean,
        self.median,
        self.standard_deviation,
        self.population_variance,
        self.population_standard_deviation,
        self.variance,
        self.minimum,
        self.maximum
    )
    }
}
