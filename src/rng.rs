pub struct Rng {
    pub fastrand_rng: fastrand::Rng,
    cache: u32,
    cache_lsb_left: u8,
}

impl Rng {
    pub fn random_seed() -> u64 {
        fastrand::Rng::new().u64(..)
    }
    pub fn new_seeded(seed: u64) -> Self {
        Self { fastrand_rng: fastrand::Rng::with_seed(seed), cache: 0, cache_lsb_left: 0 }
    }
    pub fn gen_bits(&mut self, bits: u8) -> u32 {
        assert!(bits <= 32);
        if self.cache_lsb_left < bits {
            self.cache = self.fastrand_rng.u32(..);
            self.cache_lsb_left = 32;
        }
        let ret = self.cache & !(!0 << bits);
        self.cache_lsb_left -= bits;
        self.cache >>= bits;
        ret
    }
    pub fn gen_bool(&mut self) -> bool {
        self.gen_bits(1) != 0
    }
}
