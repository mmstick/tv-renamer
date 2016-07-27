use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

use tvdb;
use backend::{self, Arguments, logging};
use backend::tokenizer::{self, TemplateToken};
use backend::traits::{ToFilename, Try, TryAndIgnore};

mod man;
use self::man::MAN_PAGE;

const EP_NO_VAL:     &'static [u8] = b"no value was set for the episode count.";
const EP_NAN:        &'static [u8] = b"no value was set for the episode count.";
const SR_NO_VAL:     &'static [u8] = b"no value was set for the series name.";
const SN_NO_VAL:     &'static [u8] = b"no value was set for the season number.";
const SN_NAN:        &'static [u8] = b"value set for season number is not a number.";
const PD_NO_VAL:     &'static [u8] = b"no value was set for the pad length.";
const PD_NAN:        &'static [u8] = b"value set for pad length is not a number.";
const TMP_NO_VAL:    &'static [u8] = b"no value was set for the template.";
const INVALID_CWD:   &'static [u8] = b"unable to get the current working directory.";
const INVALID_ARG:   &'static [u8] = b"invalid argument was given.";
const TOO_MANY_ARGS: &'static [u8] = b"too many extra arguments given to the program.";
const STDERR_ERROR:  &'static [u8] = b"error writing to stderr: ";
const STDOUT_ERROR:  &'static [u8] = b"error writing to stdout: ";

impl Arguments {
    fn new(arguments: &[String]) -> Arguments {
        let stdout = io::stdout();
        let stdout = &mut stdout.lock();
        let stderr = &mut io::stderr();

        // Initialize the default variables of the program.
        let mut program = Arguments {
            automatic:     false,
            dry_run:       false,
            log_changes:   false,
            verbose:       false,
            directory:     String::new(),
            series_name:   String::new(),
            season_number: 1,
            episode_count: 1,
            pad_length:    2,
            template:      tokenizer::default_template(),
        };

        // Create a peekable iterator so that we can see the value of some options.
        // We will also need to ignore values that have already been read.
        let mut iterator = arguments.iter().peekable();
        let mut ignore = false;

        // Advance the iterator in a loop until all values have been read.
        while let Some(argument) = iterator.next() {
            if ignore { ignore = false; continue }
            if argument.starts_with('-') {
                match argument.as_str() {
                    "-a" | "--automatic" => program.automatic = true,
                    "-h" | "--help" => {
                        let _ = stdout.write(MAN_PAGE.as_bytes());
                        let _ = stdout.flush();
                        process::exit(0);
                    }
                    "-d" | "--dry-run" => program.dry_run = true,
                    "-e" | "--episode-start" => {
                        program.episode_count = iterator.peek().try(EP_NO_VAL, stderr).parse::<usize>()
                            .try_and_ignore(EP_NAN, stderr);
                        ignore = true;
                    },
                    "-l" | "--log-changes" => program.log_changes = true,
                    "-n" | "--series-name" => {
                        program.series_name.push_str(iterator.peek().try(SR_NO_VAL, stderr));
                        ignore = true;
                    },
                    "-s" | "--season-number" => {
                        program.season_number = iterator.peek().try(SN_NO_VAL, stderr).parse::<usize>()
                            .try_and_ignore(SN_NAN, stderr);
                        ignore = true;
                    },
                    "-t" | "--template" => {
                        program.template = tokenizer::tokenize_template(iterator.peek().try(TMP_NO_VAL, stderr).as_str());
                        ignore = true;
                    },
                    "-p" | "--pad-length" => program.pad_length = iterator.peek().try(PD_NO_VAL, stderr)
                        .parse::<usize>().try_and_ignore(PD_NAN, stderr),
                    "-v" | "--verbose" => program.verbose = true,
                    _ => abort_with_message(stderr, INVALID_ARG)
                }
            } else if program.directory.is_empty() {
                program.directory = argument.clone();
            } else {
                println!("{}", argument);
                abort_with_message(stderr, TOO_MANY_ARGS)
            }
        }

        // Set to current working directory if no directory argument is given.
        if program.directory.is_empty() {
            program.directory = String::from(env::current_dir().try_and_ignore(INVALID_CWD, stderr).to_str().unwrap());
        }

        // If no series name was given, ask for the name.
        if program.series_name.is_empty() {
            program.series_name = String::from(Path::new(&program.directory).file_name().unwrap().to_str().unwrap());
        }

        program
    }
    /// Renames all episodes belonging to a season.
    fn rename_episodes_cli(&self, directory: &Path, stderr: &mut io::Stderr, stdout: &mut io::Stdout) {
        // If TVDB is enabled, create the API and search for the series information.
        let api = tvdb::Tvdb::new("0629B785CE550C8D");
        let series_info = if self.template.contains(&TemplateToken::TVDB) {
            match api.search(self.series_name.as_str(), "en") {
                Ok(reply) => Some(reply),
                Err(_) => {
                    stderr.write(b"tv-renamer: unable to get series information.\n").try(STDERR_ERROR, stderr);
                    stderr.flush().try(STDERR_ERROR, stderr);
                    process::exit(1);
                }
            }
        } else {
            None
        };

        // Get a list of episodes
        let episodes = backend::get_episodes(directory.to_str().unwrap()).unwrap_or_else(|err| panic!("{}", err));

        let mut current_episode = self.episode_count;
        for source in episodes {
            // If TVDB is enabled, get the episode title from the series information, else return an empty string.
            let title = if self.template.contains(&TemplateToken::TVDB) {
                let reply = series_info.clone()
                    .unwrap_or_else(|| panic!("tv-renamer: could not clone series: {:?}", source));
                // let episode_name = api.episode(&reply[0], self.season_number as u32, current_episode as u32)
                match api.episode(&reply[0], self.season_number as u32, current_episode as u32) {
                    Ok(episode) => episode.episode_name,
                    Err(_) => {
                        println!("tv-renamer: episode '{}' does not exist", source.to_string_lossy());
                        current_episode += 1;
                        continue
                    }
                }
            } else {
                String::new()
            };

            // Obtain target destination.
            let target = self.get_destination(directory, &source, current_episode, &title);

            // Print a message if the source already exists, else rename the source if dry-run is disabled.
            if fs::metadata(&target).is_ok() {
                stdout.write(b"tv-renamer: ").try(STDOUT_ERROR, stderr);
                stdout.write(target.to_string_lossy().as_bytes()).try(STDOUT_ERROR, stderr);
                stdout.write(b" already exists, not renaming.\n").try(STDOUT_ERROR, stderr);
                stdout.flush().try(STDOUT_ERROR, stderr);
            } else if !self.dry_run {
                fs::rename(&source, &target).try(b"unable to rename source: ", stderr);
            }

            // If verbose or dry-run is enabled, print the change that occurred.
            if self.verbose | self.dry_run {
                stdout.write(b"\x1b[1m\x1b[32m").try(b"error writing to stdout: ", stderr);
                stdout.write(backend::shorten_path(&source).to_string_lossy().as_bytes()).try(STDOUT_ERROR, stderr);
                stdout.write(b"\x1b[0m ->  \x1b[1m\x1b[32m").try(STDOUT_ERROR, stderr);
                stdout.write(backend::shorten_path(&target).to_string_lossy().as_bytes()).try(STDOUT_ERROR, stderr);
                stdout.write(b"\x1b[0m\n").try(STDOUT_ERROR, stderr);
                stdout.flush().try(STDOUT_ERROR, stderr);
            }

            // Write the changes to the log if logging is enabled.
            if self.log_changes { logging::append_change(source.as_path(), target.as_path()); }

            current_episode += 1;
        }
    }
}

pub fn launch(arguments: &[String], stderr: &mut io::Stderr) {
    let stdout = &mut io::stdout();
    let program = &mut Arguments::new(arguments);

    if program.log_changes { logging::append_time(); }

    if program.automatic {
        let series = PathBuf::from(&program.directory);
        program.series_name = series.to_filename();
        let seasons = match backend::get_seasons(&program.directory) {
            Ok(seasons) => seasons,
            Err(err) => panic!("{}", err)
        };
        for season in seasons {
            if let Some(number) = backend::derive_season_number(&season) {
                program.season_number = number;
                program.rename_episodes_cli(&season, stderr, stdout);
            }
        }
    } else {
        program.rename_episodes_cli(Path::new(&program.directory), stderr, stdout);
    }
}

#[inline]
/// Print an error message and abort with an exit status of 1.
fn abort_with_message(stderr: &mut io::Stderr, message: &[u8]) {
    stderr.write(b"tv-renamer: ").try(b"error writing to stderr: ", stderr);
    stderr.write(message).try(b"error writing to stderr: ", stderr);
    stderr.flush().try(b"error writing to stderr: ", stderr);
    process::exit(1);
}
