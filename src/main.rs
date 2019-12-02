extern crate gtk;

mod ui;
use crate::ui::ui::UI;

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    UI::run();
}

