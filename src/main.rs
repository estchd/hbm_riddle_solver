#![feature(integer_atomics)]
#![feature(duration_millis_float)]

mod sign_iterator;

use std::ops::{BitAnd, BitXor};
use std::sync::atomic::{AtomicI64, Ordering};
use hexhex::hex;
use nonempty::NonEmpty;
use rand::Rng;
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};
use sha2::Digest;
use crate::sign_iterator::{SignIterator};

static HASHES: &'static [&'static str] = &[
    "41de5c372b0589bbdb80571e87efa95ea9e34b0d74c6005b8eab495b7afd9994",
    "31da6223a100ed348ceb3254ceab67c9cc102cb2a04ac24de0df3ef3479b1036"
];

static ALLOWED_CHARS: &'static str = "abcdefghijklmnopqrstuvwxyz -";

static LINE_CONSTANTS: [Option<&'static str>; 4] = [
    None,
    None,
    None,
    None
];

static LINE_OPTIONS: [Option<&[&'static str]>; 4] = [
    //None,
    Some(&["you", "yourself", "mountains", "an echo", "echo"]),
    //None,
    Some(&["radio", "noise", "smog", "glare", "skyglow", "light pollution", "sunlight", "sun-rays", "wind", "uv-ray", "uv ray", "uv-rays", "uv rays", "gravity", "radiation", "clouds"]),
    //None,
    Some(&["you", "entropy", "operator", "nature", "scientist", "the observer", "observer", "euphemia li britannia", "digamma crystal", "digamma laser crystal", "electricity", "current", "silex", "murky anvil"]),
    //None,
    Some(&[
        "smoke ring",
        "cigarette smoke",
        "cigar smoke",
        "tobacco smoke",
        "smoke",
        "fire",
    ])
];

#[cfg_attr(feature = "hotpath", hotpath::measure)]
fn generate(current: &[[u8; 15]; 4], allowed_chars: &NonEmpty<char>) -> [String; 4] {
    let mut strings: [String; 4] = [
        String::with_capacity(15),
        String::with_capacity(15),
        String::with_capacity(15),
        String::with_capacity(15)
    ];

    for i in 0..4 {
        for l in 0..15 {
            let value = current[i][l];

            if value == 0 {
                break;
            }

            strings[i].push(allowed_chars[(value - 1 )as usize]);
        }
    }

    strings
}

#[cfg_attr(feature = "hotpath", hotpath::main(percentiles = [100]))]
fn main() {
    let allowed_chars: NonEmpty<char> = NonEmpty::from_vec(ALLOWED_CHARS
        .chars()
        .collect::<Vec<char>>()).unwrap();
    let possible_hashes: Vec<Vec<u8>> = HASHES.iter()
        .map(|hash| hexhex::decode(hash).unwrap())
        .collect();

    let char_random_hashes = allowed_chars
        .iter()
        .map(|char| {
            let char_string = format!("{}", char);
            let bytes = char_string.as_bytes();

            let mut random = Random::new();
            random.set_seed(bytes[0] as i64);

            let random_value = random.next_int(0xFFFFFF);

            format!("{}", random_value)
        })
        .collect::<Vec<String>>();

    let sign_iterator = SignIterator::from_readable_config(allowed_chars.clone(), &LINE_OPTIONS, &LINE_CONSTANTS);

    let result = sign_iterator.par_bridge().into_par_iter().find_first(|sign_indices| {
        check_solution(sign_indices, &allowed_chars, &possible_hashes, &char_random_hashes)
    });

    if let Some(result) = result {
        let text = generate(&result, &allowed_chars);

        println!("ANSWER FOUND, HOORAY!!!!");
        println!("ANSWER:");
        println!("Question 1: {}", text[0]);
        println!("Question 2: {}", text[1]);
        println!("Question 3: {}", text[2]);
        println!("Question 4: {}", text[3]);
        println!();

        let hash = smoosh((&text[0], result[0][0] - 1), (&text[1], result[1][0] - 1), (&text[2], result[2][0] - 1), (&text[3], result[3][0] - 1), &char_random_hashes);
        println!("Correct Hash: {}", hex(hash));
    }
    else {
        eprintln!("NO ANSWER FOUND, CONGRATS FOR THE WASTED TIME");
    }
}

#[cfg_attr(feature = "hotpath", hotpath::measure)]
fn check_solution(sign_indices: &[[u8; 15]; 4], allowed_chars: &NonEmpty<char>, possible_hashes: &Vec<Vec<u8>>, char_random_hashes: &Vec<String>) -> bool {
    let sign = generate(sign_indices, &allowed_chars);

    if sign[0].len() == 0 || sign[1].len() == 0 || sign[2].len() == 0 || sign[3].len() == 0 {
        return false;
    }

    let result = smoosh((&sign[0], sign_indices[0][0] - 1), (&sign[1], sign_indices[1][0] - 1), (&sign[2], sign_indices[2][0] - 1), (&sign[3], sign_indices[3][0] - 1), &char_random_hashes);

    let mut found: bool = false;

    if result.len() != possible_hashes[0].len() {
        panic!("Invalid length: {}, expected: {}", result.len(), possible_hashes[0].len());
    }

    for possible_hash in possible_hashes {
        if &result == possible_hash {
            found = true;
            break;
        }
    }

    found
}

#[cfg_attr(feature = "hotpath", hotpath::measure)]
fn smoosh(line_1: (&str, u8), line_2: (&str, u8), line_3: (&str, u8), line_4: (&str, u8), char_random_hashes: &[String]) -> Vec<u8> {
    if line_1.0.len() == 0 || line_2.0.len() == 0 || line_3.0.len() == 0 || line_4.0.len() == 0 {
        return vec![];
    }

    let rand_1_string: &str;
    let rand_2_string: &str;
    let rand_3_string: &str;
    let rand_4_string: &str;

    #[cfg(feature = "hotpath")]
    hotpath::measure_block!("riddle_solver::smoosh::char_random_hashes", {
        rand_1_string = &char_random_hashes[line_1.1 as usize];
        rand_2_string = &char_random_hashes[line_2.1 as usize];
        rand_3_string = &char_random_hashes[line_3.1 as usize];
        rand_4_string = &char_random_hashes[line_4.1 as usize];
    });

    #[cfg(not(feature = "hotpath"))]
    {
        rand_1_string = &char_random_hashes[line_1.1 as usize];
        rand_2_string = &char_random_hashes[line_2.1 as usize];
        rand_3_string = &char_random_hashes[line_3.1 as usize];
        rand_4_string = &char_random_hashes[line_4.1 as usize];
    }

    let mut s: String;

    #[cfg(not(feature = "hotpath"))]
    {
        s = String::with_capacity(
            line_1.0.len() +
                line_2.0.len() +
                line_3.0.len() +
                line_4.0.len() +
                rand_1_string.len() +
                rand_2_string.len() +
                rand_3_string.len() +
                rand_4_string.len()
        );

        s = s + line_1.0;
        s = s + &rand_1_string;
        s = s + line_2.0;
        s = s + &rand_2_string;
        s = s + line_3.0;
        s = s + &rand_3_string;
        s = s + line_4.0;
        s = s + &rand_4_string;
    }

    #[cfg(feature = "hotpath")]
    hotpath::measure_block!("riddle_solver::smoosh::input_string", {
        s = String::with_capacity(
            line_1.0.len() +
                line_2.0.len() +
                line_3.0.len() +
                line_4.0.len() +
                rand_1_string.len() +
                rand_2_string.len() +
                rand_3_string.len() +
                rand_4_string.len()
        );

        s = s + line_1.0;
        s = s + &rand_1_string;
        s = s + line_2.0;
        s = s + &rand_2_string;
        s = s + line_3.0;
        s = s + &rand_3_string;
        s = s + line_4.0;
        s = s + &rand_4_string;
    });

    get_hash(&s)
}

#[cfg_attr(feature = "hotpath", hotpath::measure)]
fn get_hash(input: &str) -> Vec<u8> {
    sha2::Sha256::digest(input.as_bytes()).to_vec()
}

static SEED_UNIQUIFIER: AtomicI64 = AtomicI64::new(8682522807148012);

fn seed_uniquifier() -> i64 {
    loop {
        let current: i64 = SEED_UNIQUIFIER.load(Ordering::Relaxed);

        let next : i64 = current.wrapping_mul(1181783497276652981);

        if SEED_UNIQUIFIER.compare_exchange(current, next, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
            return next;
        }
    }
}

static MULTIPLIER: i64 = 0x5DEECE66D;
static ADDEND: i64 = 0xB;
static MASK: i64 = (1 << 48) - 1;

struct Random {
    seed: i64,
    have_next_next_gaussian: bool,
}

impl Random {
    pub fn new() -> Self {
        Self::from_seed(seed_uniquifier().bitxor(rand::rng().random_range(0..i64::MAX)))
    }

    pub fn from_seed(seed: i64) -> Self {
        Self {
            seed: Self::initial_scramble(seed),
            have_next_next_gaussian: false,
        }
    }

    fn initial_scramble(seed: i64) -> i64 {
        seed.bitxor(MULTIPLIER).bitand(MASK)
    }

    pub fn set_seed(&mut self, seed: i64) {
        self.seed = Self::initial_scramble(seed);
        self.have_next_next_gaussian = false;
    }

    pub fn next_int(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            panic!("Invalid Bound: {}", bound);
        }

        let mut r = self.next(31);
        let m = bound - 1;

        if bound.bitand(m) == 0 {
            let mul_bound = (bound as i64).wrapping_mul(r as i64);
            r = (mul_bound >> 31) as i32;
        }
        else {
            let mut u = r;

            loop {
                r = u % bound;

                let value = u.wrapping_sub(r).wrapping_add(m);

                if value >= 0 {
                    break;
                }

                u = self.next(31);
            }
        }

        r
    }

    fn next(&mut self, bits: i32) -> i32 {
        let oldseed: i64 = self.seed;
        let newseed: i64 = oldseed.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND).bitand(MASK);

        self.seed = newseed;

        let shift_index: u32 = 48 - bits as u32;

        let shifted = newseed as u64 >> shift_index;

        shifted as i32
    }
}