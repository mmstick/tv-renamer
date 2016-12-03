use backend::{self, Arguments, ReadDirError, ScanDir, Season, tokenizer, DRY_RUN};

use gdk::enums::key;
use gtk::prelude::*;
use gtk::{
    self, Builder, Button, Entry, FileChooserDialog, ListStore,
    SpinButton, TreeView, TreeViewColumn, Type, Window, WindowType
};
use std::fs;
use std::path::{Path, PathBuf};
use tvdb;

/// Allow drag-and-drop support in the directory entry text field by fixing the URI generated by dropped files.
#[inline]
fn parse_directory(directory: &str) -> String {
    let mut output = String::from(directory);
    if output.starts_with("file://") {
        output = output.replace("file://", "");
        output = output.replace("%20", " ");
        output = output.replace("\n", "");
        output = output.replace("\r", "");
    }
    output
}

pub fn interface() {
    gtk::init().unwrap_or_else(|_| panic!("tv-renamer: failed to initialize GTK."));

    // Open the Glade GTK UI and import key GTK objects from the UI.
    let builder = Builder::new_from_string(include_str!("gtk_interface.glade"));
    let window: Window                  = builder.get_object("main_window").unwrap();
    let preview_button: Button          = builder.get_object("preview_button").unwrap();
    let rename_button: Button           = builder.get_object("rename_button").unwrap();
    let series_name_entry: Entry        = builder.get_object("series_name_entry").unwrap();
    let series_directory_entry: Entry   = builder.get_object("series_directory_entry").unwrap();
    let template_entry: Entry           = builder.get_object("template_entry").unwrap();
    let series_directory_button: Button = builder.get_object("series_directory_button").unwrap();
    let episode_spin_button: SpinButton = builder.get_object("episode_spin_button").unwrap();
    let season_spin_button: SpinButton  = builder.get_object("season_spin_button").unwrap();
    let preview_tree: TreeView          = builder.get_object("preview_tree").unwrap();
    let info_bar: gtk::InfoBar          = builder.get_object("info_bar").unwrap();
    let info_button: Button             = builder.get_object("info_close").unwrap();
    let notification_label: gtk::Label  = builder.get_object("notification_label").unwrap();

    // TreeView's List Store
    // Link these up to the preview_tree and then start renaming
    let preview_list = ListStore::new(&[Type::String, Type::String]);

    // A simple macro for adding a column to the preview tree.
    macro_rules! add_column {
        ($preview_tree:ident, $title:expr, $id:expr) => {{
            let column   = TreeViewColumn::new();
            let renderer = gtk::CellRendererText::new();
            column.set_title($title);
            column.set_resizable(true);
            column.pack_start(&renderer, true);
            column.add_attribute(&renderer, "text", $id);
            preview_tree.append_column(&column);
        }}
    }

    // Create and append the Before column to the preview tree
    add_column!(preview_tree, "Before", 0);
    add_column!(preview_tree, "After", 1);

    // Connect the preview_list to the preview tree
    preview_tree.set_model(Some(&preview_list));
    preview_tree.set_headers_visible(true);

    // A simple macro that is shared among all widgets that trigger the action to either
    // update the preview or rename the TV series.
    macro_rules! rename_action {
        ($widget:ident, $dry_run:ident, $dialog:ident) => {{
            let $widget             = $widget.clone();
            let season_spin_button  = season_spin_button.clone();
            let episode_spin_button = episode_spin_button.clone();
            let series_entry        = series_name_entry.clone();
            let directory_entry     = series_directory_entry.clone();
            let preview_list        = preview_list.clone();
            let info_bar            = info_bar.clone();
            let notification_label  = notification_label.clone();
            let template_entry      = template_entry.clone();
            $widget.connect_clicked(move |_| {
                if $dialog {
                    // Open file chooser dialog to modify series_directory_entry.
                    let dialog = FileChooserDialog::new (
                        Some("Choose Directory"),
                        Some(&Window::new(WindowType::Popup)),
                        gtk::FileChooserAction::SelectFolder,
                    );
                    dialog.add_button("Cancel", gtk::ResponseType::Cancel.into());
                    dialog.add_button("Select", gtk::ResponseType::Ok.into());

                    if dialog.run() == gtk::ResponseType::Ok.into() {
                        dialog.get_filename().map(|path| path.to_str().map(|text| directory_entry.set_text(text)));
                    }
                    dialog.destroy();
                }
                if let Some(directory) = directory_entry.get_text() {
                    let mut program = &mut Arguments {
                        flags:          if $dry_run { DRY_RUN } else { 0 },
                        base_directory: parse_directory(&directory),
                        series_name:    series_entry.get_text().unwrap_or_default(),
                        season_index:   season_spin_button.get_value_as_int() as u8,
                        episode_index:  episode_spin_button.get_value_as_int() as u16,
                        pad_length:     2,
                        template:       tokenizer::tokenize_template(template_entry.get_text().unwrap().as_str())
                    };

                    if program.series_name.is_empty() {
                        program.series_name = String::from(Path::new(&program.base_directory)
                            .file_name().unwrap().to_str().unwrap());
                        series_entry.set_text(program.series_name.as_str());
                    }

                    if !program.base_directory.is_empty() {
                        rename_series(&program, &preview_list, &info_bar, &notification_label);
                    }
                }
            });
        }}
    }

    // All of the widgets that implement the rename action.
    rename_action!(preview_button, true, false);
    rename_action!(series_directory_button, true, true);
    rename_action!(rename_button, false, false);

    { // Hide the Info Bar when the Info Bar is closed
        let info_bar = info_bar.clone();
        info_button.connect_clicked(move |_| {
            info_bar.hide();
        });
    }

    window.show_all();
    info_bar.hide();


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

/// Attempt to rename all of the seasons within a given series
fn rename_series(args: &Arguments, preview_list: &ListStore, info_bar: &gtk::InfoBar, notification_label: &gtk::Label) {
    preview_list.clear();
    match backend::scan_directory(&args.base_directory, args.season_index) {
        Ok(ScanDir::Episodes(season)) => {
            match rename_season(&season, args.episode_index, args, preview_list) {
                Ok(_) => {
                    if args.flags & DRY_RUN != 0 { return }
                    info_bar.set_message_type(gtk::MessageType::Info);
                    notification_label.set_text("Rename Success");
                },
                Err(why) => match_rename_error(info_bar, notification_label, why, args)
            }
        },
        Ok(ScanDir::Seasons(seasons))  => {
            for season in seasons {
                match rename_season(&season, 1, args, preview_list) {
                    Ok(_) => {
                        if args.flags & DRY_RUN != 0 { return }
                        info_bar.set_message_type(gtk::MessageType::Info);
                        notification_label.set_text("Rename Success");
                    },
                    Err(why) => {
                        match_rename_error(info_bar, notification_label, why, args);
                        break
                    }
                }
            }
        },
        Err(why) => {
            info_bar.set_message_type(gtk::MessageType::Error);
            notification_label.set_text(match why {
                ReadDirError::UnableToReadDir => "Unable to read directory",
                ReadDirError::InvalidDirEntry => "Directory entry is invalid",
                ReadDirError::MimeFileErr     => "Unable to open /etc/mime.types",
                ReadDirError::MimeStringErr   => "Unable to read /etc/mime.types to string"
            });
        }
    }
    info_bar.show();
}

/// If a rename error occurs, write the message to the `InfoBar`.
fn match_rename_error(info_bar: &gtk::InfoBar, notification_label: &gtk::Label, why: RenameErr, args: &Arguments) {
    info_bar.set_message_type(gtk::MessageType::Error);
    let message = match why {
        RenameErr::RenameFailed(source, target) => format!("Could not rename {:?} to {:?}", source, target),
        RenameErr::TargetExists(path)           => format!("{:?} already exists", path),
        RenameErr::EpisodeDoesNotExist(episode) => format!("Episode {} could not be found on TheTVDB", episode),
        RenameErr::SeriesLookupFailed           => format!("{} could not be found on TheTVDB", &args.series_name)
    };
    notification_label.set_text(message.as_str());
}

enum RenameErr {
    TargetExists(PathBuf),
    RenameFailed(PathBuf, PathBuf),
    EpisodeDoesNotExist(u16),
    SeriesLookupFailed
}

/// Renames a given season and updates the preview for each episode renamed.
/// If executed with `arguments.dry_run` set to true, the preview will be updated but the files will not be renamed.
fn rename_season(season: &Season, episode_no: u16, arguments: &Arguments, preview_list: &ListStore)
    -> Result<(), RenameErr>
{
    let mut episode_no = episode_no;

    // TVDB
    let api = tvdb::Tvdb::new("0629B785CE550C8D");
    let series_id = api.search(&arguments.series_name, "en").map_err(|_| RenameErr::SeriesLookupFailed)?[0].seriesid;

    for source in &season.episodes {
        let target = backend::collect_target(source, season.season_no, episode_no, arguments, &api, series_id)
            .map_err(|_| RenameErr::EpisodeDoesNotExist(episode_no))?;
        if target.exists() { return Err(RenameErr::TargetExists(source.clone())); }
        update_preview(preview_list, source, &target);
        if arguments.flags & DRY_RUN == 0 {
            fs::rename(&source, &target).map_err(|_| RenameErr::RenameFailed(source.clone(), target))?;
        }
        episode_no += 1;
    }
    Ok(()) // Rename success
}

#[inline]
/// Appends an episode to the preview list
fn update_preview(preview_list: &ListStore, source: &Path, target: &Path) {
    let src = source.components().last().unwrap().as_os_str().to_str().unwrap().to_string();
    let trg = target.components().last().unwrap().as_os_str().to_str().unwrap().to_string();
    preview_list.insert_with_values(None, &[0, 1], &[&src, &trg]);
}
