extern crate gtk;
extern crate gdk;
extern crate gio;

pub mod ui {
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
            let main_window: Window = INSTANCE.with(|i| i.builder.get_object("mainWindow").unwrap());
            main_window.show_all();
            INSTANCE.with(|i| i.hide_results());
            gtk::main();
        }

        // This will be obsolete once the PR supporting Builder::connect_signals is shipped
        fn connect_signals_from_builder(self: &UI) {
            // Global
            let main_window: Window = self.builder.get_object("mainWindow").unwrap();
            main_window.connect_delete_event(UI::ui_quit);

            let info_bar: InfoBar = self.builder.get_object("mainInfoBar").unwrap();
            info_bar.connect_response(UI::ui_clear_errors);

            // Generate text
            let entry_secret_text: Entry = self.builder.get_object("entrySecretText").unwrap();
            entry_secret_text.connect_changed(UI::ui_validate_text_entry);

            let spinner_total_pieces_text: SpinButton = self.builder.get_object("spinnerTotalPiecesText").unwrap();
            spinner_total_pieces_text.connect_value_changed(UI::ui_validate_text_spinner);

            let spinner_required_pieces_text: SpinButton = self.builder.get_object("spinnerRequiredPiecesText").unwrap();
            spinner_required_pieces_text.connect_value_changed(UI::ui_validate_text_spinner);

            let button_generate_text: Button = self.builder.get_object("buttonGenerateText").unwrap();
            button_generate_text.connect_clicked(UI::ui_generate_text);

            // Generate file

            let button_choose_secret_file: FileChooserButton = self.builder.get_object("buttonChooseSecretFile").unwrap();
            button_choose_secret_file.connect_file_set(UI::ui_validate_file_chooser);

            let spinner_total_pieces_file: SpinButton = self.builder.get_object("spinnerTotalPiecesFile").unwrap();
            spinner_total_pieces_file.connect_value_changed(UI::ui_validate_file_spinner);

            let spinner_required_pieces_file: SpinButton = self.builder.get_object("spinnerRequiredPiecesFile").unwrap();
            spinner_required_pieces_file.connect_value_changed(UI::ui_validate_file_spinner);

            let button_generate_file: Button = self.builder.get_object("buttonGenerateFile").unwrap();
            button_generate_file.connect_clicked(UI::ui_generate_file);

            let button_open_result_file: Button = self.builder.get_object("buttonOpenResultFile").unwrap();
            button_open_result_file.connect_clicked(UI::ui_open_file);

            // Reconstruct text

            let entry_reconstruct_text_prime_modulator: Entry = self.builder.get_object("entryReconstructTextPrimeModulator").unwrap();
            entry_reconstruct_text_prime_modulator.connect_changed(UI::ui_validate_reconstruct_text_entry);

            let spinner_reconstruct_text_pieces: SpinButton = self.builder.get_object("spinnerReconstructTextPieces").unwrap();
            spinner_reconstruct_text_pieces.connect_value_changed(UI::ui_validate_reconstruct_text_spinner);

            let grid_reconstruct_text: Grid = self.builder.get_object("gridReconstructTextPieces").unwrap();
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

            let button_reconstruct_text: Button = self.builder.get_object("buttonReconstructText").unwrap();
            button_reconstruct_text.connect_clicked(UI::ui_generate_file);

            // Reconstruct file

            let button_reconstruct_file_choose_pieces: Button = self.builder.get_object("buttonReconstructFileChoosePieces").unwrap();
            button_reconstruct_file_choose_pieces.connect_clicked(UI::ui_choose_pieces_reconstruct_file);

            let button_reconstruct_file: Button = self.builder.get_object("buttonReconstructFile").unwrap();
            button_reconstruct_file.connect_clicked(UI::ui_reconstruct_file);

            let button_reconstruct_file_open_result: Button = self.builder.get_object("buttonReconstructFileOpenResult").unwrap();
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

// Generate text

        fn ui_validate_text() {
            println!("TODO: validate text")
        }

        fn ui_validate_text_entry(_entry: &Entry) {
            UI::ui_validate_text();
        }

        fn ui_validate_text_spinner(_spinner: &SpinButton) {
            UI::ui_validate_text();
        }

        fn ui_generate_text(_button: &Button) {
            println!("TODO: generate text");
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
}
