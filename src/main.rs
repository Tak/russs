extern crate gtk;

mod ui;
mod sss;
use crate::ui::UI;

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    UI::run();
}

