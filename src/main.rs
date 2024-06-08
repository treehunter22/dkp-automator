#[macro_use]
extern crate lazy_static;

extern crate google_sheets4 as sheets4;

pub mod points;
pub mod sanitise;
pub mod sheets;

fn clear() {
    print!("\x1B[2J");
}

#[tokio::main]
async fn main() {
    let Some(lines) = sanitise::get_valid_lines() else {
        return;
    };

    let Some(names) = sheets::get_names_from_sheets().await else {
        return;
    };
}
