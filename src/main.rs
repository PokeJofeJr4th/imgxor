#![warn(clippy::nursery, clippy::pedantic)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

use clap::{Parser, Subcommand};
use image::Rgb;
use image::{io::Reader, ImageBuffer};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::preview::preview;

mod preview;

#[derive(Parser)]
/// Simple two-way image encryption. Use a password to encode or decode an image.
struct Args {
    /// Path to the source image
    img_file: String,
    /// Encryption Password
    mask_password: String,
    #[command(subcommand)]
    sub_command: SubCommand,
}

#[derive(Clone, Subcommand)]
enum SubCommand {
    /// View the image without writing it to a file. Press `q` to exit the program, or any other key to reload the terminal window.
    Preview,
    /// Save the image to a file
    Save {
        /// File to save to
        out_file: String,
    },
}

fn main() {
    let Args {
        img_file,
        mask_password,
        sub_command,
    } = Args::parse();

    println!("Opening {img_file}...");
    let mut img = Reader::open(Path::new(&img_file))
        .map_err(|err| format!("Error opening {img_file}: {err}"))
        .unwrap()
        .decode()
        .map_err(|err| format!("Error decoding {img_file}: {err}"))
        .unwrap()
        .to_rgb8();
    let width = img.width();
    let height = img.height();
    println!("Generating Encryption Mask...");
    let mut rng = seed_rng_with_string(&mask_password);
    let mask = ImageBuffer::from_fn(width, height, |_, _| Rgb(rng.gen::<[u8; 3]>()));

    println!("Encrypting...");
    for x in 0..width {
        for y in 0..height {
            let img_px = img.get_pixel_mut(x, y);
            let mask_px = mask.get_pixel(x, y);
            for i in 0..3 {
                img_px.0[i] ^= mask_px.0[i]
            }
        }
    }
    match sub_command {
        SubCommand::Save { out_file } => {
            println!("Saving {out_file}...");
            img.save(&out_file)
                .map_err(|err| format!("Error saving {out_file}: {err}"))
                .unwrap();
            println!("Done!");
        }
        SubCommand::Preview => preview(&img),
    }
}

fn seed_rng_with_string(seed_string: &str) -> ChaCha8Rng {
    // Create a hasher and hash the input string
    let mut hasher = DefaultHasher::new();
    seed_string.hash(&mut hasher);
    let seed = hasher.finish();

    // Create an RNG with the hashed seed
    let rng = ChaCha8Rng::seed_from_u64(seed);

    // Return the RNG as a trait object
    rng
}
