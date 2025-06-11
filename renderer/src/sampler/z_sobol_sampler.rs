use crate::sampler::{Sampler, sobol_matrices::*};

struct FastOwenScrambler {
    seed: u32,
}
impl FastOwenScrambler {
    fn new(seed: u32) -> Self {
        Self { seed }
    }

    fn reverse_bits_32(mut n: u32) -> u32 {
        n = n.rotate_right(16);
        n = ((n & 0x00ff00ff) << 8) | ((n & 0xff00ff00) >> 8);
        n = ((n & 0x0f0f0f0f) << 4) | ((n & 0xf0f0f0f0) >> 4);
        n = ((n & 0x33333333) << 2) | ((n & 0xcccccccc) >> 2);
        n = ((n & 0x55555555) << 1) | ((n & 0xaaaaaaaa) >> 1);
        n
    }

    fn randomize(&self, mut v: u32) -> u32 {
        v = Self::reverse_bits_32(v);
        v ^= v.wrapping_mul(0x3d20adea);
        v = v.wrapping_add(self.seed);
        v = v.wrapping_mul((self.seed >> 16) | 1);
        v ^= v.wrapping_mul(0x05526c56);
        v ^= v.wrapping_mul(0x53a22864);
        Self::reverse_bits_32(v)
    }
}

// SobolSampler implementation exactly following PBRT-v4 SobolSampler
#[derive(Clone, Debug)]
pub struct ZSobolSampler {
    dimension: u32,
    seed: u32,
    log2_spp: u32,
    n_base4_digits: u32,
    morton_index: u32,
}
impl ZSobolSampler {
    fn log2_int(v: u32) -> u32 {
        if v == 0 { 0 } else { 31 - v.leading_zeros() }
    }

    // Round up to next power of 2
    fn round_up_pow2(v: u32) -> u32 {
        if v <= 1 {
            1
        } else {
            1 << (32 - (v - 1).leading_zeros())
        }
    }

    fn encode_morton2(x: u32, y: u32) -> u32 {
        fn left_shift2(mut x: u64) -> u64 {
            x &= 0xffffffff;
            x = (x ^ (x << 16)) & 0x0000ffff0000ffff;
            x = (x ^ (x << 8)) & 0x00ff00ff00ff00ff;
            x = (x ^ (x << 4)) & 0x0f0f0f0f0f0f0f0f;
            x = (x ^ (x << 2)) & 0x3333333333333333;
            x = (x ^ (x << 1)) & 0x5555555555555555;
            x
        }
        ((left_shift2(y as u64) as u32) << 1) | (left_shift2(x as u64) as u32)
    }

    fn mix_bits(mut v: u64) -> u64 {
        v ^= v >> 31;
        v = v.wrapping_mul(0x7fb5d329728ea185);
        v ^= v >> 27;
        v = v.wrapping_mul(0x81dadef4bc2dd44d);
        v ^= v >> 33;
        v
    }

    fn hash(dimension: u32, seed: u32) -> u64 {
        const M: u64 = 0xc6a4a7935bd1e995;
        const R: u32 = 47;

        let mut buf = [0u8; 8];
        buf[..4].copy_from_slice(&dimension.to_le_bytes());
        buf[4..].copy_from_slice(&seed.to_le_bytes());

        let mut h: u64 = 8_u64.wrapping_mul(M);

        let mut k = u64::from_le_bytes(buf);
        k = k.wrapping_mul(M);
        k ^= k >> R;
        k = k.wrapping_mul(M);

        h ^= k;
        h = h.wrapping_mul(M);

        h ^= h >> R;
        h = h.wrapping_mul(M);
        h ^= h >> R;

        h
    }

    fn get_sample_index(&self) -> u64 {
        const PERMUTATIONS: [[u8; 4]; 24] = [
            [0, 1, 2, 3],
            [0, 1, 3, 2],
            [0, 2, 1, 3],
            [0, 2, 3, 1],
            [0, 3, 2, 1],
            [0, 3, 1, 2],
            [1, 0, 2, 3],
            [1, 0, 3, 2],
            [1, 2, 0, 3],
            [1, 2, 3, 0],
            [1, 3, 2, 0],
            [1, 3, 0, 2],
            [2, 1, 0, 3],
            [2, 1, 3, 0],
            [2, 0, 1, 3],
            [2, 0, 3, 1],
            [2, 3, 0, 1],
            [2, 3, 1, 0],
            [3, 1, 2, 0],
            [3, 1, 0, 2],
            [3, 2, 1, 0],
            [3, 2, 0, 1],
            [3, 0, 2, 1],
            [3, 0, 1, 2],
        ];

        let mut sample_index = 0;

        let pow2_samples = self.log2_spp & 1 == 1;
        let last_digit = if pow2_samples { 1 } else { 0 };
        let mut i = self.n_base4_digits as i32 - 1;
        while i >= last_digit {
            let digit_shift = 2 * i - (if pow2_samples { 1 } else { 0 });
            let digit = (self.morton_index as u64 >> digit_shift) & 3;

            let higher_digits = self.morton_index as u64 >> (digit_shift + 2);
            let p =
                (Self::mix_bits(higher_digits ^ (0x55555555 * self.dimension as u64)) >> 24) % 24;

            let digit = PERMUTATIONS[p as usize][digit as usize] as u64;
            sample_index |= digit << digit_shift;
            i -= 1;
        }

        sample_index
    }

    fn sobol_sample<R>(a: u64, dimension: usize, randomizer: R) -> f32
    where
        R: Fn(u32) -> u32,
    {
        let mut v = 0;
        let mut i = dimension * SOBOL_MATRIX_SIZE;
        let mut a = a;
        while a != 0 {
            if (a & 1) != 0 {
                v ^= SOBOL_MATRICES_32[i];
            }
            a >>= 1;
            i += 1;
        }

        v = randomizer(v);

        const FLOAT_ONE_MINUS_EPSILON: f32 = f32::from_bits(0x3f7fffff); // 0x1.fffffep-1
        (v as f32 * f32::from_bits(0x2f800000)).min(FLOAT_ONE_MINUS_EPSILON) // 0x1p-32
    }
}
impl Sampler for ZSobolSampler {
    fn new(spp: u32, resolution: glam::UVec2, seed: u32) -> Self {
        let log2_spp = Self::log2_int(spp);
        let res = Self::round_up_pow2(resolution.x.max(resolution.y));
        let log4_spp = log2_spp.div_ceil(2);
        let n_base4_digits = Self::log2_int(res) + log4_spp;

        Self {
            dimension: 0,
            seed,
            log2_spp,
            n_base4_digits,
            morton_index: 0,
        }
    }

    fn start_pixel_sample(&mut self, p: glam::UVec2, sample_index: u32) {
        self.dimension = 0;
        self.morton_index = (Self::encode_morton2(p.x, p.y) << self.log2_spp) | sample_index;
    }

    fn get_1d(&mut self) -> f32 {
        let sample_index = self.get_sample_index();
        self.dimension += 1;

        let sample_hash = Self::hash(self.dimension, self.seed);
        Self::sobol_sample(sample_index, 0, |v| {
            let randomizer = FastOwenScrambler::new(sample_hash as u32);
            randomizer.randomize(v)
        })
    }

    fn get_2d(&mut self) -> glam::Vec2 {
        let sample_index = self.get_sample_index();
        self.dimension += 2;

        let bits = Self::hash(self.dimension, self.seed);
        let sample_hash = glam::uvec2(bits as u32, (bits >> 32) as u32);
        glam::vec2(
            Self::sobol_sample(sample_index, 0, |v| {
                let randomizer = FastOwenScrambler::new(sample_hash.x);
                randomizer.randomize(v)
            }),
            Self::sobol_sample(sample_index, 1, |v| {
                let randomizer = FastOwenScrambler::new(sample_hash.y);
                randomizer.randomize(v)
            }),
        )
    }

    fn get_2d_pixel(&mut self) -> glam::Vec2 {
        self.get_2d()
    }
}
