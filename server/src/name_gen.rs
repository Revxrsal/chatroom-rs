use rand::{Rng, thread_rng};

const ADJECTIVES: &'static [&str] = &[
    "excited",
    "happy",
    "fast",
    "sweepy",
    "sweaty",
    "swifty",
    "slow",
    "smart",
    "intelligent",
    "cool",
    "brave",
    "blue",
    "green",
];

const NAMES: &'static [&str] = &[
    "octopus",
    "cat",
    "tiger",
    "lion",
    "fish",
    "axolotl",
    "sheep",
    "salmon",
    "squirrel",
    "chocolate",
    "horse",
    "camel",
    "apple",
];

pub fn gen_new_name() -> String {
    let adj = ADJECTIVES[thread_rng().gen_range(0..ADJECTIVES.len())];
    let name = NAMES[thread_rng().gen_range(0..NAMES.len())];
    format!("{}-{}", adj, name)
}