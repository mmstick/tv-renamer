use backend::common::{self, Arguments};
use backend::traits::{Try};
use chrono::*;
use gtk;
use gdk::enums::key;
use gtk::prelude::*;
use gtk::{
    Builder, Button, CheckButton, Entry, FileChooserDialog, ListBox, ListBoxRow, ListStore,
    SpinButton, TreeView, TreeViewColumn, Type, Window, WindowType
};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

// TODO:
// - Implement GTK3 notifications for when errors occur, and when renaming has completed successfully
// - Add a tab for checking the rename log.
// - Add support for a configuration file.
// - Reduce source code redundancy with macros and functions.

pub fn launch() {
    gtk::init().unwrap_or_else(|_| panic!("tv-renamer: failed to initialize GTK."));

    // Open the Glade GTK UI and import key GTK objects from the UI.
    let builder = Builder::new_from_string(include_str!("gtk_interface.glade"));
    let window: Window                  = builder.get_object("main_window").unwrap();
    let preview_button: Button          = builder.get_object("preview_button").unwrap();
    let rename_button: Button           = builder.get_object("rename_button").unwrap();
    let input_list: ListBox             = builder.get_object("input_list").unwrap();
    let series_name_entry: Entry        = builder.get_object("series_name_entry").unwrap();
    let series_directory_entry: Entry   = builder.get_object("series_directory_entry").unwrap();
    let series_directory_button: Button = builder.get_object("series_directory_button").unwrap();
    let episode_spin_button: SpinButton = builder.get_object("episode_spin_button").unwrap();
    let season_spin_button: SpinButton  = builder.get_object("season_spin_button").unwrap();
    let preview_tree: TreeView          = builder.get_object("preview_tree").unwrap();

    // Create rows for the input_list
    let automatic_row = ListBoxRow::new();
    automatic_row.set_selectable(false);
    let series_name_row = ListBoxRow::new();
    series_name_row.set_selectable(false);
    let tvdb_row = ListBoxRow::new();
    tvdb_row.set_selectable(false);
    let log_changes_row = ListBoxRow::new();
    log_changes_row.set_selectable(false);

    // Create check boxes for the rows
    let automatic_check = CheckButton::new_with_label("Automatic");
    let no_name_check = CheckButton::new_with_label("No Series Name");
    let tvdb_check = CheckButton::new_with_label("TVDB Titles");
    let log_changes_check = CheckButton::new_with_label("Log Changes");

    // Add the check boxes to the rows
    automatic_row.add(&automatic_check);
    series_name_row.add(&no_name_check);
    tvdb_row.add(&tvdb_check);
    log_changes_row.add(&log_changes_check);

    // Add the rows to the list box
    input_list.insert(&automatic_row, -1);
    input_list.insert(&series_name_row, -1);
    input_list.insert(&tvdb_row, -1);
    input_list.insert(&log_changes_row, -1);

    // TreeView's List Store
    // Link these up to the preview_tree and then start renaming
    let preview_list = ListStore::new(&[
        Type::String, // Before
        Type::String, // After
    ]);

    // Create and append the Before column to the preview tree
    let before_column = TreeViewColumn::new();
    let renderer = gtk::CellRendererText::new();
    before_column.set_title("Before");
    before_column.set_resizable(true);
    before_column.pack_start(&renderer, true);
    before_column.add_attribute(&renderer, "text", 0);
    preview_tree.append_column(&before_column);

    // Create and append the After column to the preview tree
    let after_column = TreeViewColumn::new();
    let renderer = gtk::CellRendererText::new();
    after_column.set_title("After");
    after_column.set_resizable(true);
    after_column.pack_start(&renderer, true);
    after_column.add_attribute(&renderer, "text", 1);
    preview_tree.append_column(&after_column);

    // Connect the preview_list to the preview tree
    preview_tree.set_model(Some(&preview_list));
    preview_tree.set_headers_visible(true);

    { // NOTE: Programs the Choose Directory button with a File Chooser Dialog.
        let directory_entry = series_directory_entry.clone();
        series_directory_button.connect_clicked(move |_| {
            // Open file chooser dialog to modify series_directory_entry.
            let dialog = FileChooserDialog::new (
                Some("Choose Directory"),
                Some(&Window::new(WindowType::Popup)),
                gtk::FileChooserAction::SelectFolder,
            );
            dialog.add_button("Cancel", gtk::ResponseType::Cancel as i32);
            dialog.add_button("OK", gtk::ResponseType::Ok as i32);

            if dialog.run() == gtk::ResponseType::Ok as i32 {
                if let Some(path) = dialog.get_filename() {
                    directory_entry.set_text(path.to_str().unwrap());
                }
            }
            dialog.destroy();
        });
    }

    { // NOTE: Controls what happens when the preview button is pressed
        let button              = preview_button.clone();
        let auto                = automatic_check.clone();
        let no_name             = no_name_check.clone();
        let tvdb                = tvdb_check.clone();
        let log_changes         = log_changes_check.clone();
        let season_spin_button  = season_spin_button.clone();
        let episode_spin_button = episode_spin_button.clone();
        let series_entry        = series_name_entry.clone();
        let directory_entry     = series_directory_entry.clone();
        let preview_list        = preview_list.clone();
        button.connect_clicked(move |_| {
            let mut program = Arguments {
                automatic:     auto.get_active(),
                dry_run:       false,
                log_changes:   log_changes.get_active(),
                no_name:       no_name.get_active(),
                tvdb:          tvdb.get_active(),
                verbose:       false,
                directory:     directory_entry.get_text().unwrap(),
                series_name:   series_entry.get_text().unwrap(),
                season_number: season_spin_button.get_value_as_int() as usize,
                episode_count: episode_spin_button.get_value_as_int() as usize,
                pad_length:    2,
            };

            if program.automatic {
                let series = PathBuf::from(&program.directory);
                program.series_name = series.components().last().unwrap().as_os_str().to_str().unwrap().to_string();
                match common::get_seasons(&program.directory) {
                    Ok(seasons) => {
                        preview_list.clear();
                        for season in seasons {
                            match common::derive_season_number(&season) {
                                Some(number) => program.season_number = number,
                                None         => continue
                            }
                            match common::get_episodes(season.as_os_str().to_str().unwrap()) {
                                Ok(episodes) => {
                                    match program.get_targets(season.as_os_str().to_str().unwrap(), &episodes, program.episode_count) {
                                        Ok(targets) => {
                                            for (source, target) in episodes.iter().zip(targets) {
                                                let source = source.components().last().unwrap().as_os_str().to_str().unwrap().to_string();
                                                let target = target.components().last().unwrap().as_os_str().to_str().unwrap().to_string();
                                                preview_list.insert_with_values(None, &[0, 1], &[&source, &target]);
                                            }
                                        },
                                        Err(err) => {
                                            // Dialog of Error?
                                            println!("{:?}", err);
                                        }
                                    };
                                },
                                Err(err) => {
                                    // Dialog of Error?
                                    println!("{:?}", err);
                                }
                            };
                        }
                    },
                    Err(err) => {
                        // Dialog of Error?
                        println!("{:?}", err);
                    }
                }
            } else {
                match common::get_episodes(&program.directory) {
                    Ok(episodes) => {
                        match program.get_targets(&program.directory, &episodes, program.episode_count) {
                            Ok(targets) => {
                                preview_list.clear();
                                for (source, target) in episodes.iter().zip(targets) {
                                    let source = source.components().last().unwrap().as_os_str().to_str().unwrap().to_string();
                                    let target = target.components().last().unwrap().as_os_str().to_str().unwrap().to_string();
                                    preview_list.insert_with_values(None, &[0, 1], &[&source, &target]);
                                }
                            },
                            Err(err) => {
                                // Dialog of Error?
                                println!("{:?}", err);
                            }
                        };
                    },
                    Err(err) => {
                        // Dialog of Error?
                        println!("{:?}", err);
                    }
                };
            }
        });
    }

    { // NOTE: Controls what happens when rename button is pressed
        let button              = rename_button.clone();
        let auto                = automatic_check.clone();
        let no_name             = no_name_check.clone();
        let tvdb                = tvdb_check.clone();
        let log_changes         = log_changes_check.clone();
        let season_spin_button  = season_spin_button.clone();
        let episode_spin_button = episode_spin_button.clone();
        let series_entry        = series_name_entry.clone();
        let directory_entry     = series_directory_entry.clone();
        let preview_list        = preview_list.clone();
        button.connect_clicked(move |_| {
            let mut program = Arguments {
                automatic:     auto.get_active(),
                dry_run:       false,
                log_changes:   log_changes.get_active(),
                no_name:       no_name.get_active(),
                tvdb:          tvdb.get_active(),
                verbose:       false,
                directory:     directory_entry.get_text().unwrap(),
                series_name:   series_entry.get_text().unwrap(),
                season_number: season_spin_button.get_value_as_int() as usize,
                episode_count: episode_spin_button.get_value_as_int() as usize,
                pad_length:    2,
            };

            if program.automatic {
                let series = PathBuf::from(&program.directory);
                program.series_name = series.components().last().unwrap().as_os_str().to_str().unwrap().to_string();
                match common::get_seasons(&program.directory) {
                    Ok(seasons) => {
                        preview_list.clear();
                        for season in seasons {
                            match common::derive_season_number(&season) {
                                Some(number) => program.season_number = number,
                                None         => continue
                            }
                            match common::get_episodes(season.as_os_str().to_str().unwrap()) {
                                Ok(episodes) => {
                                    match program.get_targets(season.as_os_str().to_str().unwrap(), &episodes, program.episode_count) {
                                        Ok(targets) => {
                                            // Append the current time to the log if logging is enabled.
                                            if program.log_changes {
                                                let mut log_path = ::std::env::home_dir()
                                                    .try(b"unable to get home directory: ", &mut io::stderr());
                                                log_path.push("tv-renamer.log");
                                                let mut log = fs::OpenOptions::new().create(true).append(true).open(&log_path)
                                                    .try(b"unable to open log: ", &mut io::stderr());
                                                let local_time = Local::now().to_rfc2822();
                                                let _ = log.write(b"\n");
                                                let _ = log.write_all(local_time.as_bytes());
                                                let _ = log.write(b"\n");
                                                let _ = log.flush();
                                            }

                                            // Clear the preview and then update it
                                            preview_list.clear();
                                            for (source, target) in episodes.iter().zip(targets) {
                                                let _ = fs::rename(&source, &target);

                                                // Write the changes to the log if logging is enabled.
                                                if program.log_changes {
                                                    let mut log_path = ::std::env::home_dir()
                                                        .try(b"unable to get home directory: ", &mut io::stderr());
                                                    log_path.push("tv-renamer.log");
                                                    let mut log = fs::OpenOptions::new().append(true).open(&log_path)
                                                        .try(b"unable to open log: ", &mut io::stderr());
                                                    let _ = log.write(common::shorten_path(&source).to_string_lossy().as_bytes());
                                                    let _ = log.write(b" -> ");
                                                    let _ = log.write(common::shorten_path(&target).to_string_lossy().as_bytes());
                                                    let _ = log.write(b"\n");
                                                    let _ = log.flush();
                                                }

                                                // Update the preview
                                                let source = source.components().last().unwrap().as_os_str().to_str().unwrap().to_string();
                                                let target = target.components().last().unwrap().as_os_str().to_str().unwrap().to_string();
                                                preview_list.insert_with_values(None, &[0, 1], &[&source, &target]);
                                            }
                                        },
                                        Err(err) => {
                                            // Dialog of Error?
                                            println!("{:?}", err);
                                        }
                                    };
                                },
                                Err(err) => {
                                    // Dialog of Error?
                                    println!("{:?}", err);
                                }
                            };
                        }
                    },
                    Err(err) => {
                        // Dialog of Error?
                        println!("{:?}", err);
                    }
                }
            } else {
                match common::get_episodes(&program.directory) {
                    Ok(episodes) => {
                        match program.get_targets(&program.directory, &episodes, program.episode_count) {
                            Ok(targets) => {
                                // Append the current time to the log if logging is enabled.
                                if program.log_changes {
                                    let mut log_path = ::std::env::home_dir()
                                        .try(b"unable to get home directory: ", &mut io::stderr());
                                    log_path.push("tv-renamer.log");
                                    let mut log = fs::OpenOptions::new().create(true).append(true).open(&log_path)
                                        .try(b"unable to open log: ", &mut io::stderr());
                                    let local_time = Local::now().to_rfc2822();
                                    let _ = log.write(b"\n");
                                    let _ = log.write_all(local_time.as_bytes());
                                    let _ = log.write(b"\n");
                                    let _ = log.flush();
                                }

                                // Clear the preview, rename the files and then update the preview
                                preview_list.clear();
                                for (source, target) in episodes.iter().zip(targets) {
                                    let _ = fs::rename(&source, &target);

                                    // Write the changes to the log if logging is enabled.
                                    if program.log_changes {
                                        let mut log_path = ::std::env::home_dir()
                                            .try(b"unable to get home directory: ", &mut io::stderr());
                                        log_path.push("tv-renamer.log");
                                        let mut log = fs::OpenOptions::new().append(true).open(&log_path)
                                            .try(b"unable to open log: ", &mut io::stderr());
                                        let _ = log.write(common::shorten_path(&source).to_string_lossy().as_bytes());
                                        let _ = log.write(b" -> ");
                                        let _ = log.write(common::shorten_path(&target).to_string_lossy().as_bytes());
                                        let _ = log.write(b"\n");
                                        let _ = log.flush();
                                    }

                                    // Update the preview
                                    let source = source.components().last().unwrap().as_os_str().to_str().unwrap().to_string();
                                    let target = target.components().last().unwrap().as_os_str().to_str().unwrap().to_string();
                                    preview_list.insert_with_values(None, &[0, 1], &[&source, &target]);
                                }
                            },
                            Err(err) => {
                                // Dialog of Error?
                                println!("{:?}", err);
                            }
                        };
                    },
                    Err(err) => {
                        // Dialog of Error?
                        println!("{:?}", err);
                    }
                };
            }
        });
    }

    window.show_all();

    // Quit the program when the program has been exited
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    // Define custom actions on keypress
    window.connect_key_press_event(move |_, key| {
        if let key::Escape = key.get_keyval() { gtk::main_quit() }
        gtk::Inhibit(false)
    });

    gtk::main();

}
