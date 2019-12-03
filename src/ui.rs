extern crate gtk;
extern crate gdk;
extern crate gio;
extern crate base64;

use crate::sss::sss;

use gtk::prelude::*;
//use gio::prelude::*;

use gtk::*;
use gdk::Event;

// All UI action is on the main thread in any case
thread_local!(static INSTANCE: UI = UI::new());

pub struct UI {
    builder: Builder,
}

impl UI {
    pub fn new() -> UI {
        return UI{ builder: Builder::new_from_string(include_str!("ui.glade")) };
    }

    pub fn run() {
        INSTANCE.with(|i| i.connect_signals_from_builder());
        let main_window: Window = UI::get_object("mainWindow");
        main_window.show_all();
        INSTANCE.with(|i| i.hide_results());
        gtk::main();
    }

    // This will be obsolete once the PR supporting Builder::connect_signals is shipped
    fn connect_signals_from_builder(self: &UI) {
        // Global
        let main_window: Window = self.get_object_instance("mainWindow");
        main_window.connect_delete_event(UI::ui_quit);

        let info_bar: InfoBar = self.get_object_instance("mainInfoBar");
        info_bar.connect_response(UI::ui_clear_errors);

        // Generate text
        let entry_secret_text: Entry = self.get_object_instance("entrySecretText");
        entry_secret_text.connect_changed(UI::ui_validate_text_entry);

        let spinner_total_pieces_text: SpinButton = self.get_object_instance("spinnerTotalPiecesText");
        spinner_total_pieces_text.connect_value_changed(UI::ui_validate_text_spinner);

        let spinner_required_pieces_text: SpinButton = self.get_object_instance("spinnerRequiredPiecesText");
        spinner_required_pieces_text.connect_value_changed(UI::ui_validate_text_spinner);

        let button_generate_text: Button = self.get_object_instance("buttonGenerateText");
        button_generate_text.connect_clicked(UI::ui_generate_text);

        // Generate file

        let button_choose_secret_file: FileChooserButton = self.get_object_instance("buttonChooseSecretFile");
        button_choose_secret_file.connect_file_set(UI::ui_validate_file_chooser);

        let spinner_total_pieces_file: SpinButton = self.get_object_instance("spinnerTotalPiecesFile");
        spinner_total_pieces_file.connect_value_changed(UI::ui_validate_file_spinner);

        let spinner_required_pieces_file: SpinButton = self.get_object_instance("spinnerRequiredPiecesFile");
        spinner_required_pieces_file.connect_value_changed(UI::ui_validate_file_spinner);

        let button_generate_file: Button = self.get_object_instance("buttonGenerateFile");
        button_generate_file.connect_clicked(UI::ui_generate_file);

        let button_open_result_file: Button = self.get_object_instance("buttonOpenResultFile");
        button_open_result_file.connect_clicked(UI::ui_open_file);

        // Reconstruct text

        let entry_reconstruct_text_prime_modulator: Entry = self.get_object_instance("entryReconstructTextPrimeModulator");
        entry_reconstruct_text_prime_modulator.connect_changed(UI::ui_validate_reconstruct_text_entry);

        let spinner_reconstruct_text_pieces: SpinButton = self.get_object_instance("spinnerReconstructTextPieces");
        spinner_reconstruct_text_pieces.connect_value_changed(UI::ui_validate_reconstruct_text_spinner);

        let grid_reconstruct_text: Grid = self.get_object_instance("gridReconstructTextPieces");
        let mut row = 0;

        loop {
            let child = grid_reconstruct_text.get_child_at(row, 0);
            if child.is_none() {
                break;
            }

            let entry: Entry = child.unwrap().downcast().unwrap();
            entry.connect_changed(UI::ui_validate_reconstruct_text_entry);
            let entry: Entry = grid_reconstruct_text.get_child_at(row, 1).unwrap().downcast().unwrap();
            entry.connect_changed(UI::ui_validate_reconstruct_text_entry);
            row += 1;
        }

        let button_reconstruct_text: Button = self.get_object_instance("buttonReconstructText");
        button_reconstruct_text.connect_clicked(UI::ui_generate_file);

        // Reconstruct file

        let button_reconstruct_file_choose_pieces: Button = self.get_object_instance("buttonReconstructFileChoosePieces");
        button_reconstruct_file_choose_pieces.connect_clicked(UI::ui_choose_pieces_reconstruct_file);

        let button_reconstruct_file: Button = self.get_object_instance("buttonReconstructFile");
        button_reconstruct_file.connect_clicked(UI::ui_reconstruct_file);

        let button_reconstruct_file_open_result: Button = self.get_object_instance("buttonReconstructFileOpenResult");
        button_reconstruct_file_open_result.connect_clicked(UI::ui_open_reconstruct_file);
    }

    fn hide_results(self: &UI) {
        // Hide result UI until we actually do something
        let results_containers = [
            "frameResultsText",
            "boxReconstructTextSecret",
            "frameResultsFile",
            "frameReconstructFileResults",
            "mainInfoBar",
        ];

        for name in &results_containers {
            let widget: Widget = self.builder.get_object(name).unwrap();
            widget.hide();
        }
    }

    fn ui_quit(_window: &Window, _event: &Event) -> Inhibit {
        gtk::main_quit();
        return Inhibit(false);
    }

    fn ui_clear_errors(infobar: &InfoBar, _response: ResponseType) {
        infobar.hide();
    }

    fn display_error(message: &str) {
        UI::get_object::<Label>("labelError").set_text(message);
        UI::get_object::<InfoBar>("mainInfoBar").show_all();
    }

    fn clear_grid(grid: &Grid) {
        loop {
            let child = grid.get_child_at(0, 0);
            if child.is_none() {
                break;
            }
            grid.remove_row(0);
        }
    }

    fn get_object_instance<T>(self: &UI, name: &str) -> T
        where T: gtk::IsA<gtk::Object> {
        let thing: T = self.builder.get_object(name).unwrap();
        return thing;
    }

    fn get_object<T>(name: &str) -> T
        where T: gtk::IsA<gtk::Object> {
        return INSTANCE.with(|instance| return instance.get_object_instance(name));
    }

    // Generate text

    fn ui_validate_text() {
        let secret = UI::get_object::<Entry>("entrySecretText").get_text().unwrap();
        let total_pieces = UI::get_object::<SpinButton>("spinnerTotalPiecesText").get_value();
        let required_pieces = UI::get_object::<SpinButton>("spinnerRequiredPiecesText").get_value();
        UI::get_object::<Button>("buttonGenerateText").set_sensitive(!secret.is_empty() && total_pieces >= required_pieces);
        UI::get_object::<Frame>("frameResultsText").hide();
    }

    fn ui_validate_text_entry(_entry: &Entry) {
        UI::ui_validate_text();
    }

    fn ui_validate_text_spinner(_spinner: &SpinButton) {
        UI::ui_validate_text();
    }

    fn get_selectable_label(text: &str, alignment: f64) -> Label {
        let label = Label::new(Some(text));
        label.set_selectable(true);
        label.set_xalign(alignment as f32);
        return label;
    }

    fn encode_base64(input: &Vec<u8>) -> String {
        base64::encode_config(input, base64::URL_SAFE)
    }

    fn ui_generate_text(_button: &Button) {
        let secret = UI::get_object::<Entry>("entrySecretText").get_text().unwrap();
        let total_pieces = UI::get_object::<SpinButton>("spinnerTotalPiecesText").get_value() as u32;
        let required_pieces = UI::get_object::<SpinButton>("spinnerRequiredPiecesText").get_value() as u32;
        let progress_bar: ProgressBar = UI::get_object("progressText");
        let total_progress = secret.len();
        let generate_button: Button = UI::get_object("buttonGenerateText");
        let prime = 5717;

        UI::ui_clear_errors(&UI::get_object("mainInfoBar"), ResponseType::Close);
        generate_button.set_sensitive(false);

        let result = sss::generate_string(&secret, total_pieces, required_pieces, prime, |progress| progress_bar.set_fraction(progress));
        if result.is_err() {
            UI::display_error(format!("Error generating shards for {}: {}", secret, result.unwrap_err()).as_str());
            return;
        }

        let pieces = result.unwrap();
        UI::get_object::<Label>("labelPrimeText").set_text(format!("{}", prime).as_str());

        // Build result grid
        let grid: Grid = UI::get_object("gridResultText");
        UI::clear_grid(&grid);
        for index in 1..pieces.len() {
            grid.insert_row(index as i32);
            grid.attach(&UI::get_selectable_label(format!("{}", index).as_str(), 1.0), 0, index as i32 - 1, 1, 1);
            grid.attach(&UI::get_selectable_label(UI::encode_base64(&pieces[index - 1]).as_str(), 0.25), 1, index as i32 - 1, 1, 1);
        }

        progress_bar.set_fraction(1.0);
        UI::get_object::<Frame>("frameResultsText").show_all();
        generate_button.set_sensitive(true);
    }

    // Generate file

    fn ui_validate_file() {
        println!("TODO: validate file");
    }

    fn ui_validate_file_chooser(_chooser: &FileChooserButton) {
        UI::ui_validate_file();
    }

    fn ui_validate_file_spinner(_spinner: &SpinButton) {
        UI::ui_validate_file();
    }

    fn ui_generate_file(_button: &Button) {
        println!("TODO: generate file");
    }

    fn ui_open_file(_button: &Button) {
        println!("TODO: open result directory");
    }

    fn ui_validate_reconstruct_text() {
        println!("TODO: validate reconstruct text")
    }

    fn ui_validate_reconstruct_text_entry(_entry: &Entry) {
        UI::ui_validate_reconstruct_text();
    }

    fn ui_validate_reconstruct_text_spinner(_spinner: &SpinButton) {
        UI::ui_validate_reconstruct_text();
    }

    fn ui_validate_reconstruct_file() {
        println!("TODO: validate reconstruct file")
    }

    fn ui_choose_pieces_reconstruct_file(_button: &Button) {
        println!("TODO: show reconstruct file pieces chooser dialog");
    }

    fn ui_reconstruct_file(_button: &Button) {
        println!("TODO: reconstruct file");
    }

    fn ui_open_reconstruct_file(_button: &Button) {
        println!("TODO: open reconstructed file");
    }
}
