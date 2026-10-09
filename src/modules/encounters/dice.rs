use rand::RngExt;

pub trait Dice {
    fn d(self, sides: &u32) -> u32;
}

impl Dice for u32 {
    fn d(self, sides: &u32) -> u32 {
        let mut rng = rand::rng();
        (0..self).map(|_| rng.random_range(1..=*sides)).sum()
    }
}
