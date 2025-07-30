use rand::prelude::*;
use rand::seq::SliceRandom;

const DARK_PREFIXES: &[&str] = &[
    "Grim", "Dark", "Shadow", "Blood", "Death", "Void", "Black", "Fell", 
    "Bone", "Doom", "Wraith", "Plague", "Blight", "Sorrow", "Vile", "Cruel"
];

const DARK_SUFFIXES: &[&str] = &[
    "maw", "claw", "fang", "bane", "reaper", "stalker", "slayer", "hunter",
    "ripper", "render", "gore", "scourge", "fiend", "spawn", "wretch", "ghoul"
];

const DARK_TITLES: &[&str] = &[
    "the Corrupted", "the Fallen", "the Damned", "the Cursed", "the Wicked",
    "the Vile", "the Sinister", "the Malevolent", "the Twisted", "the Tainted",
    "Soulrender", "Fleshripper", "Bonecrusher", "Deathbringer", "Plaguespread"
];

pub fn generate_dark_name() -> String {
    let mut rng = thread_rng();
    
    match rng.gen_range(0..3) {
        0 => {
            // Prefix + Suffix format
            let prefix = DARK_PREFIXES.choose(&mut rng).unwrap();
            let suffix = DARK_SUFFIXES.choose(&mut rng).unwrap();
            format!("{}{}", prefix, suffix)
        },
        1 => {
            // Simple name + title format
            let prefix = DARK_PREFIXES.choose(&mut rng).unwrap();
            let title = DARK_TITLES.choose(&mut rng).unwrap();
            format!("{} {}", prefix, title)
        },
        _ => {
            // Single title format  
            DARK_TITLES.choose(&mut rng).unwrap().to_string()
        }
    }
}