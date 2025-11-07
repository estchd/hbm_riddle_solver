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

/// ****************************************************************************************************************************************
/// DO NOT CHANGE ANYTHING ABOVE UNLESS YOU KNOW WHAT YOU ARE DOING
/// YOU HAVE BEEN WARNED
/// ****************************************************************************************************************************************




/// THESE HASHES ARE THE ACTUAL SOLUTION HASHES FROM THE MOD
/// DO NOT CHANGE THESE UNLESS THE MOD CODE ITSELF CHANCES
/// IF THAT HAPPENS IT IS VERY LIKELY THAT THIS SOLVERS ALGORITHM ISN'T VALID ANYMORE EITHER
static HASHES: &'static [&'static str] = &[
    "41de5c372b0589bbdb80571e87efa95ea9e34b0d74c6005b8eab495b7afd9994",
    "31da6223a100ed348ceb3254ceab67c9cc102cb2a04ac24de0df3ef3479b1036"
];

/// The charset that is used for brute forcing.
/// Add new characters here if you think your answer contains them.
/// Add characters between the "" symbols.
/// Keep in mind that each new character adds exponentially more answers that need to be checked
/// While adding the same character more than once won't crash this algorithm, it will make brute forcing take unnecessarily longer.
static ALLOWED_CHARS: &'static str = "abcdefghijklmnopqrstuvwxyz -#'123456789_,.";

/// Questions that are to be solved by a single answer
/// To use this, replace one of the None entries by Some("YOUR ANSWER HERE")
/// Each line corresponds to a question, from 1. to 4.
/// This supersedes everything else. If you put Some for a question here, putting DICTIONARY_LINES or LINE_OPTIONS for that question won't matter.
static LINE_CONSTANTS: [Option<&'static str>; 4] = [
    None, // 1. Question
    None, // 2. Question
    None, // 3. Question
    None  // 4. Question
];

/// Specifies, which questions are to be solved by trying the LINE_OPTIONS below for that question
/// If you set the true of a line to false, that question will be solved by brute-forcing.
/// Each line corresponds to a question, from 1. to 4.
/// You do not need to empty the LINE_OPTIONS for a question to make it be brute-forced.
static DICTIONARY_LINES: [bool; 4] = [
    true, // 1. Question
    true, // 2. Question
    true, // 3. Question
    true  // 4. Question
];

/// Specifies the options that are tried for each question
/// Each line corresponds to a question, from 1. to 4.
static LINE_OPTIONS: [&[&'static str]; 4] = [
    &["no one", "noone", "nobody", "sound", "air", "scream", "voice", "maxwell", "matter", "the world", "world", "the void", "void", "the echo", "your echo", "half-life scientists", "scientists", "you", "yourself", "mountains", "an echo", "echo"],
    &["atmosphere", "capitalism", "smog", "xrays", "x-ray", "xray", "x-rays", "electromagnetic", "electromagnetism", "infrared", "microwaves", "fallout", "decoy", "belief", "skybox", "faith", "ignorance", "illusion","radio", "noise", "smog", "glare", "skyglow", "light pollution", "sunlight", "sun-rays", "wind", "uv-ray", "uv ray", "uv-rays", "uv rays", "gravity", "radiation", "clouds"],
    &["screen", "fourier", "ears", "ear", "music", "sound", "current", "voltage", "separation of isotopes by laser exitation", "chaos", "starcontrol", "radar", "exposure chamber", "fel", "free electron laser", "who", "java", "the author", "author", "the bobcat", "bob", "electrons", "photons", "decoder", "time", "seed", "hash", "you", "entropy", "operator", "nature", "scientist", "the observer", "observer", "euphemia li britannia", "digamma crystal", "digamma laser crystal", "electricity", "current", "silex", "murky anvil"],
    &["lildip", "lil dip", "duchess gambit","duchessgambit", "hoboy03new", "celestium industries", "doctor schrabauer", "DrNostalgia", "ffi-brand cigarette", "soyuz", "obj_tester", "numbernine", "dyx", "minelittlepony", "pisp", "tile.obj_tester.name", "mask man", "maskman", "balls-o-tron", "radon", "isotopes", "orbitals", "electrons", "atoms", "amber", "ambers", "flame", "balefire", "smoke ring", "cigarette smoke", "cigar smoke", "tobacco smoke", "smoke", "fire"]
];

/// QUESTIONS:
///
/// 1: When we scream into the distance, who will answer?
/// 2: The invisible force, sometimes subtle and sometimes not, covers our eyes directed at the skies.
/// 3: The fluorescent oscilloscope gives us the passcode of random numbers. Who holds the answers?
/// 4: Ever revolving, never still, a sight for the patient that reeks of tobacco, dancing perpetually.



/// HOURS WASTED ON THESE QUESTIONS: 24 and counting





/// ****************************************************************************************************************************************
/// DO NOT CHANGE ANYTHING BELOW UNLESS YOU KNOW WHAT YOU ARE DOING
/// YOU HAVE BEEN WARNED
/// ****************************************************************************************************************************************

#[cfg_attr(feature = "hotpath", hotpath::measure)]
fn generate(current: &[[u8; 15]; 4], allowed_chars: &NonEmpty<u8>) -> [Vec<u8>; 4] {
    let mut strings: [Vec<u8>; 4] = [
        Vec::with_capacity(15),
        Vec::with_capacity(15),
        Vec::with_capacity(15),
        Vec::with_capacity(15)
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

#[cfg_attr(feature = "hotpath", hotpath::main(percentiles = [100], timeout = 1000))]
fn main() {
    let allowed_chars: NonEmpty<char> = NonEmpty::from_vec(ALLOWED_CHARS
        .chars()
        .collect::<Vec<char>>()).unwrap();

    let allowed_chars_bytes: NonEmpty<u8> =  NonEmpty::from_vec(allowed_chars
        .iter()
        .map(|char| {
            format!("{}", char).into_bytes()[0]
        })
        .collect::<Vec<u8>>()).unwrap();

    let possible_hashes: Vec<Vec<u8>> = HASHES.iter()
        .map(|hash| hexhex::decode(hash).unwrap())
        .collect();

    let char_random_hashes = allowed_chars_bytes
        .iter()
        .map(|char| {
            let mut random = Random::new();
            random.set_seed(*char as i64);

            let random_value = random.next_int(0xFFFFFF);

            format!("{}", random_value).into_bytes()
        })
        .collect::<Vec<Vec<u8>>>();

    let line_options: [&[&str]; 4];

    let mut all_options: Vec<&str> = Vec::new();

    for options in LINE_OPTIONS {
        all_options.extend_from_slice(options);
    }

    #[cfg(feature = "combine_dictionary_options")]
    {
        line_options = [
            all_options.as_slice(),
            all_options.as_slice(),
            all_options.as_slice(),
            all_options.as_slice(),
        ];
    }

    #[cfg(not(feature = "combine_dictionary_options"))]
    {
        line_options = LINE_OPTIONS;
    }

    let mut enabled_line_options: [Option<&[&str]>; 4] = [None; 4];

    for i in 0..4 {
        if DICTIONARY_LINES[i] {
            enabled_line_options[i] = Some(&line_options[i]);
        }
    }

    let sign_iterator: SignIterator;
    sign_iterator = SignIterator::from_readable_config(allowed_chars, &enabled_line_options, &LINE_CONSTANTS);

    let result: Option<[[u8; 15]; 4]>;

    #[cfg(feature = "split")]
    {
        let iterators = sign_iterator.split::<16>();

        result = iterators.into_par_iter().find_map_first(|mut iterator| {
            iterator.find(|sign_indices| {
                check_solution(sign_indices, &allowed_chars_bytes, &possible_hashes, &char_random_hashes)
            })
        });
    }

    #[cfg(not(feature = "split"))]
    {
        result = sign_iterator.par_bridge().into_par_iter().find_first(|sign_indices| {
            check_solution(sign_indices, &allowed_chars_bytes, &possible_hashes, &char_random_hashes)
        })
    }

    if let Some(result) = result {
        let text = generate(&result, &allowed_chars_bytes);

        let text_1 = String::from_utf8(text[0].clone()).unwrap();
        let text_2 = String::from_utf8(text[1].clone()).unwrap();
        let text_3 = String::from_utf8(text[2].clone()).unwrap();
        let text_4 = String::from_utf8(text[3].clone()).unwrap();

        println!("ANSWER FOUND, HOORAY!!!!");
        println!("ANSWER:");
        println!("Question 1: {}", text_1);
        println!("Question 2: {}", text_2);
        println!("Question 3: {}", text_3);
        println!("Question 4: {}", text_4);
        println!();

        let line_1 = Line {
            string_bytes: &text[0],
            char_random_string_bytes: &char_random_hashes[(result[0][0] - 1) as usize],
        };

        let line_2 = Line {
            string_bytes: &text[1],
            char_random_string_bytes: &char_random_hashes[(result[1][0] - 1) as usize],
        };

        let line_3 = Line {
            string_bytes: &text[2],
            char_random_string_bytes: &char_random_hashes[(result[2][0] - 1) as usize],
        };

        let line_4 = Line {
            string_bytes: &text[3],
            char_random_string_bytes: &char_random_hashes[(result[3][0] - 1) as usize],
        };

        let hash = smoosh([line_1, line_2, line_3, line_4]);
        println!("Correct Hash: {}", hex(hash));
    }
    else {
        eprintln!("NO ANSWER FOUND, CONGRATS FOR THE WASTED TIME");
    }
}

#[cfg_attr(feature = "hotpath", hotpath::measure)]
fn check_solution(sign_indices: &[[u8; 15]; 4], allowed_chars: &NonEmpty<u8>, possible_hashes: &Vec<Vec<u8>>, char_random_hashes: &Vec<Vec<u8>>) -> bool {
    let sign = generate(sign_indices, &allowed_chars);

    if sign[0].len() == 0 || sign[1].len() == 0 || sign[2].len() == 0 || sign[3].len() == 0 {
        return false;
    }

    let line_1 = Line {
        string_bytes: &sign[0],
        char_random_string_bytes: &char_random_hashes[(sign_indices[0][0] - 1) as usize],
    };

    let line_2 = Line {
        string_bytes: &sign[1],
        char_random_string_bytes: &char_random_hashes[(sign_indices[1][0] - 1) as usize],
    };

    let line_3 = Line {
        string_bytes: &sign[2],
        char_random_string_bytes: &char_random_hashes[(sign_indices[2][0] - 1) as usize],
    };

    let line_4 = Line {
        string_bytes: &sign[3],
        char_random_string_bytes: &char_random_hashes[(sign_indices[3][0] - 1) as usize],
    };

    let result = smoosh([line_1, line_2, line_3, line_4]);

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

struct Line<'a, 'b> {
    string_bytes: &'a [u8],
    char_random_string_bytes: &'b [u8],
}

#[cfg_attr(feature = "hotpath", hotpath::measure)]
fn smoosh(lines: [Line; 4]) -> Vec<u8> {
    if  lines[0].string_bytes.len() == 0 ||
        lines[1].string_bytes.len() == 0 ||
        lines[2].string_bytes.len() == 0 ||
        lines[3].string_bytes.len() == 0
    {
        return vec![];
    }

    let mut s: Vec<u8>;

    #[cfg(not(feature = "hotpath"))]
    {
        let len =
            lines[0].string_bytes.len() +
            lines[1].string_bytes.len() +
            lines[2].string_bytes.len() +
            lines[3].string_bytes.len() +
            lines[0].char_random_string_bytes.len() +
            lines[1].char_random_string_bytes.len() +
            lines[2].char_random_string_bytes.len() +
            lines[3].char_random_string_bytes.len();

        s = vec![0u8; len];

        s.extend_from_slice(lines[0].string_bytes);
        s.extend_from_slice(lines[0].char_random_string_bytes);
        s.extend_from_slice(lines[1].string_bytes);
        s.extend_from_slice(lines[1].char_random_string_bytes);
        s.extend_from_slice(lines[2].string_bytes);
        s.extend_from_slice(lines[2].char_random_string_bytes);
        s.extend_from_slice(lines[3].string_bytes);
        s.extend_from_slice(lines[3].char_random_string_bytes);
    }

    #[cfg(feature = "hotpath")]
    hotpath::measure_block!("riddle_solver::smoosh::input_string", {
       let len =
            lines[0].string_bytes.len() +
            lines[1].string_bytes.len() +
            lines[2].string_bytes.len() +
            lines[3].string_bytes.len() +
            lines[0].char_random_string_bytes.len() +
            lines[1].char_random_string_bytes.len() +
            lines[2].char_random_string_bytes.len() +
            lines[3].char_random_string_bytes.len();

        s = Vec::with_capacity(len);

        s.extend_from_slice(lines[0].string_bytes);
        s.extend_from_slice(lines[0].char_random_string_bytes);
        s.extend_from_slice(lines[1].string_bytes);
        s.extend_from_slice(lines[1].char_random_string_bytes);
        s.extend_from_slice(lines[2].string_bytes);
        s.extend_from_slice(lines[2].char_random_string_bytes);
        s.extend_from_slice(lines[3].string_bytes);
        s.extend_from_slice(lines[3].char_random_string_bytes);
    });

    let hash = get_hash(&s);

    #[cfg(feature = "debug_hash")]
    {
        println!();
        println!("{}", String::from_utf8(s.clone()).unwrap());
        println!("{}", hex(&hash));
        println!();
    }

    hash
}

#[cfg_attr(feature = "hotpath", hotpath::measure)]
fn get_hash(input: &[u8]) -> Vec<u8> {
    sha2::Sha256::digest(input).to_vec()
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