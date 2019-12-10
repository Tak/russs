extern crate gtk;
extern crate gdk;
extern crate gio;
extern crate base64;

use crate::sss;

use gtk::prelude::*;
use gio::prelude::*;

use gtk::*;
use gio::*;
use gdk::Event;
use std::cell::RefCell;

// All UI action is on the main thread in any case
thread_local!(static INSTANCE: UI = UI::new());

pub struct UI {
    builder: Builder,
    file_result_path: RefCell<String>,
    reconstructed_file_result_path: RefCell<String>,
}

impl UI {
    pub fn new() -> UI {
        return UI {
            builder: Builder::new_from_string(include_str!("ui.glade")),
            file_result_path: RefCell::new(String::from("")),
            reconstructed_file_result_path: RefCell::new(String::from("")),
        };
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
        button_reconstruct_text.connect_clicked(UI::ui_reconstruct_text);

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

    fn clear_errors() {
        UI::ui_clear_errors(&UI::get_object("mainInfoBar"), ResponseType::Close);
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
            match grid.get_child_at(0, 0) {
                None => return,
                Some(_) => grid.remove_row(0),
            }
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
        let total_pieces = UI::get_object::<SpinButton>("spinnerTotalPiecesText").get_value() as i32;
        let required_pieces = UI::get_object::<SpinButton>("spinnerRequiredPiecesText").get_value() as i32;
        let progress_bar: ProgressBar = UI::get_object("progressText");
        let total_progress = secret.len() as f64;
        let generate_button: Button = UI::get_object("buttonGenerateText");
        let prime = 5717;

        UI::ui_clear_errors(&UI::get_object("mainInfoBar"), ResponseType::Close);
        generate_button.set_sensitive(false);

        let pieces = sss::generate_string(&secret.as_str(),
                                          total_pieces,
                                          required_pieces,
                                          prime,
                                          |progress| progress_bar.set_fraction(progress / total_progress));
        // Build result grid
        let grid: Grid = UI::get_object("gridResultText");
        UI::clear_grid(&grid);
        for index in 0..pieces.len() as i32 {
            grid.insert_row(index);
            grid.attach(&UI::get_selectable_label(format!("{}", pieces[index as usize].0).as_str(), 1.0), 0, index, 1, 1);
            grid.attach(&UI::get_selectable_label(UI::encode_base64(&pieces[index as usize].1).as_str(), 0.25), 1, index, 1, 1);
        }

        progress_bar.set_fraction(1.0);
        UI::get_object::<Label>("labelPrimeText").set_text(format!("{}", prime).as_str());
        UI::get_object::<Frame>("frameResultsText").show_all();
        generate_button.set_sensitive(true);
    }

    // Generate file

    fn ui_validate_file() {
        let valid = UI::get_object::<FileChooserButton>("buttonChooseSecretFile").get_file().is_some() &&
            UI::get_object::<SpinButton>("spinnerTotalPiecesFile").get_value() >= UI::get_object::<SpinButton>("spinnerRequiredPiecesFile").get_value();
        UI::get_object::<Button>("buttonGenerateFile").set_sensitive(valid);
    }

    fn ui_validate_file_chooser(_chooser: &FileChooserButton) {
        UI::ui_validate_file();
    }

    fn ui_validate_file_spinner(_spinner: &SpinButton) {
        UI::ui_validate_file();
    }

    fn ui_generate_file(_button: &Button) {
        let prime = 7919;
        let total_pieces = UI::get_object::<SpinButton>("spinnerTotalPiecesFile").get_value() as i32;
        let required_pieces = UI::get_object::<SpinButton>("spinnerRequiredPiecesFile").get_value() as i32;
        let progress_bar: ProgressBar = UI::get_object("progressFile");
        let generate_button: Button = UI::get_object("buttonGenerateFile");

        let secret_file = UI::get_object::<FileChooserButton>("buttonChooseSecretFile").get_file().unwrap();
        let secret_file_path = secret_file.get_path().unwrap().into_os_string().into_string().unwrap();

        let total_progress: f64;
        match secret_file.query_info::<Cancellable>("", FileQueryInfoFlags::NONE, None) {
            Err(_) => {
                UI::display_error(format!("Error reading {}", secret_file_path.as_str()).as_str());
                return;
            },
            Ok(info) => total_progress = info.get_size() as f64,
        }

        let parent: String;
        match secret_file.get_parent() {
            None => {
                UI::display_error(format!("Error getting parent for {}", secret_file_path).as_str());
                return;
            },
            Some(file) => parent = file.get_uri().to_string(),
        }

        UI::clear_errors();
        generate_button.set_sensitive(false);

        match sss::generate_file(secret_file_path.as_str(),
                                        total_pieces,
                                        required_pieces,
                                        prime,
                                        |progress| progress_bar.set_fraction(progress / total_progress)) {
            Err(message) => UI::display_error(format!("Error generating shards for {}: {}", secret_file_path, message).as_str()),
            Ok(()) => {
                INSTANCE.with(|instance| instance.file_result_path.replace(parent));
                UI::get_object::<Frame>("frameResultsFile").show_all();
            },
        }

        progress_bar.set_fraction(1.0);
        generate_button.set_sensitive(true);
    }

    fn ui_open_file(_button: &Button) {
        let _ = INSTANCE.with(|instance| {
            gio::AppInfo::launch_default_for_uri::<AppLaunchContext>(&instance.file_result_path.borrow().as_str(), None)
        });
    }

    // Reconstruct text

    fn ui_validate_reconstruct_text() {
        let mut valid = UI::get_object::<Entry>("entryReconstructTextPrimeModulator").get_text().unwrap().as_str().parse::<i32>().is_ok();

        if valid {
            let grid: Grid = UI::get_object("gridReconstructTextPieces");
            let pieces_count = UI::get_object::<SpinButton>("spinnerReconstructTextPieces").get_value() as i32;

            valid = (0..pieces_count).all(|index| {
                grid.get_child_at(0, index).unwrap().downcast::<Entry>().unwrap().get_text().unwrap().as_str().parse::<i32>().is_ok() &&
                base64::decode_config(grid.get_child_at(1, index).unwrap().downcast::<Entry>().unwrap().get_text().unwrap().as_str(), base64::URL_SAFE).is_ok()
            });
        }

        UI::get_object::<Button>("buttonReconstructText").set_sensitive(valid);
    }

    fn ui_validate_reconstruct_text_entry(_entry: &Entry) {
        UI::ui_validate_reconstruct_text();
    }

    fn get_custom_entry(placeholder_text: &str, input_purpose: InputPurpose) -> Entry {
        let entry = Entry::new();
        entry.set_placeholder_text(Some(placeholder_text));
        entry.set_input_purpose(input_purpose);
        entry.connect_changed(UI::ui_validate_reconstruct_text_entry);
        return entry;
    }

    fn ui_validate_reconstruct_text_spinner(_spinner: &SpinButton) {
        let grid: Grid = UI::get_object("gridReconstructTextPieces");
        let pieces_count = UI::get_object::<SpinButton>("spinnerReconstructTextPieces").get_value() as i32;

        UI::clear_grid(&grid);

        for row in 0..pieces_count {
            grid.insert_row(row);
            grid.attach(&UI::get_custom_entry("Index", InputPurpose::Digits), 0, row, 1, 1);
            grid.attach(&UI::get_custom_entry("Secret shard", InputPurpose::FreeForm), 1, row, 1, 1);
        }
        grid.show_all();
        UI::ui_validate_reconstruct_text();
    }

    fn ui_reconstruct_text(_button: &Button) {
        let prime =  UI::get_object::<Entry>("entryReconstructTextPrimeModulator").get_text().unwrap().as_str().parse::<i32>().unwrap();
        let progress_bar: ProgressBar = UI::get_object("progressReconstructText");
        let generate_button: Button = UI::get_object("buttonReconstructText");
        let grid: Grid = UI::get_object("gridReconstructTextPieces");
        let pieces_count = UI::get_object::<SpinButton>("spinnerReconstructTextPieces").get_value() as i32;

        UI::clear_errors();
        generate_button.set_sensitive(false);

        let pieces: Vec<(i32, Vec<u8>)> = (0..pieces_count).map(|index| {
            (grid.get_child_at(0, index).unwrap().downcast::<Entry>().unwrap().get_text().unwrap().as_str().parse::<i32>().unwrap(),
            base64::decode_config(grid.get_child_at(1, index).unwrap().downcast::<Entry>().unwrap().get_text().unwrap().as_str(), base64::URL_SAFE).unwrap())
        }).collect();
        let total_progress = pieces[0].1.len() as f64;

        match sss::interpolate_string(&pieces, prime, |progress| progress_bar.set_fraction(progress / total_progress)) {
            Ok(secret) => {
                UI::get_object::<Label>("labelReconstructTextSecret").set_text(base64::encode_config(&secret, base64::URL_SAFE).as_str());
                UI::get_object::<Box>("boxReconstructTextSecret").show_all();
            },
            Err(message) => UI::display_error(format!("Error reconstructing text: {}", message).as_str()),
        }

        progress_bar.set_fraction(1.0);
        generate_button.set_sensitive(true);
    }

    // Reconstruct file

    fn ui_validate_reconstruct_file() {
        let pieces = UI::get_object::<FileChooserDialog>("chooserReconstructFileChoosePieces").get_files();
        let valid = pieces.len() > 1;

        if valid {
            UI::get_object::<Button>("buttonReconstructFileChoosePieces").set_label(format!("({} files)", pieces.len()).as_str());
        }
        UI::get_object::<Button>("buttonReconstructFile").set_sensitive(valid);
    }

    fn ui_choose_pieces_reconstruct_file(_button: &Button) {
        let dialog: FileChooserDialog = UI::get_object("chooserReconstructFileChoosePieces");
        dialog.run();
        dialog.hide();
        UI::ui_validate_reconstruct_file();
    }

    fn ui_reconstruct_file(_button: &Button) {
        let reconstruct_button: Button = UI::get_object("buttonReconstructFile");
        let progress_bar: ProgressBar = UI::get_object("progressReconstructFile");

        UI::clear_errors();
        reconstruct_button.set_sensitive(false);

        let piece_files = UI::get_object::<FileChooserDialog>("chooserReconstructFileChoosePieces").get_files();
        let pieces: Vec<String> = piece_files.iter().map(|file| {
            file.get_path().unwrap().into_os_string().into_string().unwrap()
        }).collect();
        let destination = piece_files[0].get_parent().unwrap().get_path().unwrap().into_os_string().into_string().unwrap();
        let total_progress = piece_files[0].query_info::<Cancellable>("", FileQueryInfoFlags::NONE, None).unwrap().get_size() as f64;

        match sss::interpolate_file(pieces, destination.as_str(), Some(|progress| progress_bar.set_fraction(progress / total_progress))) {
            Err(message) => UI::display_error(format!("Error reconstructing file: {}", message).as_str()),
            Ok(output_file) => {
                INSTANCE.with(|instance| instance.reconstructed_file_result_path.replace(output_file));
                UI::get_object::<Frame>("frameReconstructFileResults").show_all();
            },
        }

        progress_bar.set_fraction(1.0);
        reconstruct_button.set_sensitive(true);
    }

    fn ui_open_reconstruct_file(_button: &Button) {
        let _ = INSTANCE.with(|instance| {
            gio::AppInfo::launch_default_for_uri::<AppLaunchContext>(format!("file://{}", &instance.reconstructed_file_result_path.borrow().as_str()).as_str(), None)
        });
    }
}
