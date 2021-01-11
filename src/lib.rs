use rand::rngs::StdRng;
use rand::Rng;

pub mod in_degree_relative_distribution;
pub mod max_in_degree_distribution;
pub mod max_out_degree_distribution;
pub mod out_degree_relative_distribution;
pub mod connectivity_distribution;

pub struct SampledDistribution {
    cumulative_distribution: Vec<(f64, f64)>,
}

impl SampledDistribution {

    pub fn sample(&self, rng: &mut StdRng) -> f64 {
        let p_x = rng.gen_range((self.cumulative_distribution[0].1)..(self.cumulative_distribution[self.cumulative_distribution.len() - 1].1));
        let mut below = (0.0, 0.0);
        let mut above = (0.0, 0.0);
        for i in 1..self.cumulative_distribution.len() {
            let low = self.cumulative_distribution[i-1].1;
            let high = self.cumulative_distribution[i].1;
            if low <= p_x && p_x <= high {
                below = self.cumulative_distribution[i-1].clone();
                above = self.cumulative_distribution[i].clone();
            }
        }
        if above.1 == below.1 || above.0 == below.0 {
            below.0
        } else {
            let fraction_of_interval = (p_x - below.1) / (above.1 - below.1);
            let offset = fraction_of_interval * (above.0 - below.0);
            below.0 + offset
        }
    }

}