use core::time;

use image::{Pixel, RgbImage};

use self::terminal_handler::TermHandler;
use crossterm::event;

mod terminal_handler {
    use std::io::stdout;

    use crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };

    /// Handles raw mode and alt screen
    /// DO NOT copy or clone
    pub struct TermHandler;

    impl TermHandler {
        pub fn new() -> Self {
            execute!(stdout(), EnterAlternateScreen).unwrap();
            enable_raw_mode().unwrap();
            Self
        }
    }

    impl Drop for TermHandler {
        fn drop(&mut self) {
            disable_raw_mode().unwrap();
            execute!(stdout(), LeaveAlternateScreen).unwrap();
        }
    }
}

pub fn preview(img: &RgbImage) {
    let terminal_handler = TermHandler::new();
    let zero_secs = time::Duration::from_secs(0);
    'waiting: loop {
        let lines = approximate_image(img, crossterm::terminal::size().unwrap(), false);
        // clear the screen
        print!("\x1B[2J\x1B[1;1H");
        // print out the current window
        for l in lines {
            print!("{l}\r\n");
        }
        while matches!(event::poll(zero_secs), Ok(false)) {}
        while matches!(event::poll(zero_secs), Ok(true)) {
            if let Ok(event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Char('q'),
                kind: event::KeyEventKind::Press,
                ..
            })) = event::read()
            {
                break 'waiting;
            }
        }
    }
    drop(terminal_handler);
}

/// approximate the image as a list of terminal lines, given a specific terminal size
pub fn approximate_image(img: &RgbImage, term_size: (u16, u16), verbose: bool) -> Vec<String> {
    let (img_width, img_height) = img.dimensions();
    let (term_width, term_height) = get_img_viewport(
        (img_width * 2, img_height),
        (term_size.0 as u32 - 1, term_size.1 as u32),
    );
    let mut termlines = Vec::new();
    for row in 0..(term_height - 1) {
        let row_iter = (row * img_height / term_height)
            ..=(((row + 1) * img_height / term_height).min(img_height - 1));
        if verbose {
            print!("image rows: {row_iter:?}\r\n");
        }
        let mut current_line = String::new();
        for col in 0..term_width {
            let col_iter = (col * img_width / term_width)
                ..=(((col + 1) * img_width / term_width).min(img_width - 1));
            if verbose {
                print!("image columns: {col_iter:?}\r\n");
            }
            let mut pixels_in_chunk = Vec::new();
            for y in row_iter.clone() {
                for x in col_iter.clone() {
                    pixels_in_chunk.push(img.get_pixel(x, y).channels());
                }
            }
            let pix_len = pixels_in_chunk.len().max(1);
            let (mut r, mut g, mut b) = (0, 0, 0);
            for px in pixels_in_chunk {
                r += px[0] as usize;
                g += px[1] as usize;
                b += px[2] as usize;
            }
            let (r, g, b) = (r / pix_len, g / pix_len, b / pix_len);
            current_line.push_str(&format!(
                "\x1b[38;5;{};38;2;{r};{g};{b}m█",
                // "\x1b[38;5;{}m█\x1b[0m",
                rgb_to_256((r, g, b))
            ));
        }
        termlines.push(current_line);
    }
    termlines
}

/// transform rgb values to 8-bit colors
///
/// source: <https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797>
#[allow(clippy::cast_possible_truncation)]
pub const fn rgb_to_256((r, g, b): (usize, usize, usize)) -> u8 {
    let (r, g, b) = (
        ((r * 3) >> 7) as u8,
        ((g * 3) >> 7) as u8,
        ((b * 3) >> 7) as u8,
    );
    ((r * 36) + (g * 6) + b) + 16
}

/// given the size of an image and the terminal, figure out how big to make the image
const fn get_img_viewport((img_w, img_h): (u32, u32), (term_w, term_h): (u32, u32)) -> (u32, u32) {
    // start with the image size
    // if the image is too tall, squash it
    let (img_w_2, img_h_2) = if img_h > term_h {
        (img_w * term_h / img_h, term_h)
    } else {
        (img_w, img_h)
    };
    // if the image is too long, squish it
    if img_w_2 > term_w {
        (term_w, img_h_2 * term_w / img_w_2)
    } else {
        (img_w_2, img_h_2)
    }
}

#[cfg(test)]
mod tests {
    use crate::preview::get_img_viewport;

    #[test]
    fn img_viewport() {
        assert_eq!(get_img_viewport((2, 2), (4, 4)), (2, 2));
        assert_eq!(get_img_viewport((4, 4), (2, 2)), (2, 2));
        assert_eq!(get_img_viewport((2, 4), (4, 4)), (2, 4));
        assert_eq!(get_img_viewport((10, 10), (8, 4)), (4, 4));
        assert_eq!(get_img_viewport((20, 10), (8, 4)), (8, 4));
    }
}
