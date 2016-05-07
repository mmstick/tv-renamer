use backend::man;
use backend::common::{self, Arguments};
use backend::traits::{Try, TryAndIgnore};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;
use tvdb;
use chrono::*;

const EP_NO_VAL:     &'static [u8] = b"no value was set for the episode count.";
const EP_NAN:        &'static [u8] = b"no value was set for the episode count.";
const SR_NO_VAL:     &'static [u8] = b"no value was set for the series name.";
const SN_NO_VAL:     &'static [u8] = b"no value was set for the season number.";
const SN_NAN:        &'static [u8] = b"value set for season number is not a number.";
const PD_NO_VAL:     &'static [u8] = b"no value was set for the pad length.";
const PD_NAN:        &'static [u8] = b"value set for pad length is not a number.";
const INVALID_CWD:   &'static [u8] = b"unable to get the current working directory.";
const INVALID_ARG:   &'static [u8] = b"invalid argument was given.";
const TOO_MANY_ARGS: &'static [u8] = b"too many extra arguments given to the program.";
const STDERR_ERROR:  &'static [u8] = b"error writing to stderr: ";
const STDOUT_ERROR:  &'static [u8] = b"error writing to stdout: ";
const LOG_ERROR:     &'static [u8] = b"error writing to log: ";

trait CLI {
    fn execute(&mut self);

    fn rename_episodes(&self, directory: &Path, stderr: &mut io::Stderr, stdout: &mut io::Stdout);
}

impl CLI for Arguments {
    fn execute(&mut self) {
        let stderr = &mut io::stderr();
        let stdout = &mut io::stdout();

        // Append the current time to the log if logging is enabled.
        if self.log_changes {
            let mut log_path = env::home_dir().try(b"unable to get home directory: ", stderr);
            log_path.push("tv-renamer.log");
            let mut log = fs::OpenOptions::new().create(true).append(true).open(&log_path)
                .try(b"unable to open log: ", stderr);
            let local_time = Local::now().to_rfc2822();
            log.write(b"\n").try(LOG_ERROR, stderr);
            log.write_all(local_time.as_bytes()).try(LOG_ERROR, stderr);
            log.write(b"\n").try(LOG_ERROR, stderr);
            log.flush().try(LOG_ERROR, stderr);
        }

        if self.automatic {
            let series = PathBuf::from(&self.directory);
            self.series_name = series.components().last().unwrap().as_os_str().to_str().unwrap().to_string();
            let seasons = match common::get_seasons(&self.directory) {
                Ok(seasons) => seasons,
                Err(err) => panic!("{}", err)
            };
            for season in seasons {
                let mut directory_name = season.file_name().unwrap().to_str().unwrap().to_lowercase();
                match directory_name.as_str() {
                    "season0" | "season 0" | "specials" => self.season_number = 0,
                    _ => {
                        directory_name = directory_name.replace("season", "");
                        directory_name = directory_name.replace(" ", "");
                        if let Ok(season_number) = directory_name.parse::<usize>() {
                            self.season_number = season_number;
                        }
                    }
                }
                self.rename_episodes(&season, stderr, stdout);
            }
        } else {
            self.rename_episodes(Path::new(&self.directory), stderr, stdout);
        }
    }

    /// Renames all episodes belonging to a season.
    fn rename_episodes(&self, directory: &Path, stderr: &mut io::Stderr, stdout: &mut io::Stdout) {
        // If TVDB is enabled, create the API and search for the series information.
        let api = tvdb::Tvdb::new("0629B785CE550C8D");
        let series_info = if self.tvdb {
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
        let episodes = match common::get_episodes(directory.to_str().unwrap()) {
            Ok(episodes) => episodes,
            Err(err) => panic!("{}", err)
        };

        let mut current_episode = self.episode_count;
        for file in episodes {
            // If TVDB is enabled, get the episode title from the series information, else return an empty string.
            let title = if self.tvdb {
                let reply = series_info.clone().unwrap();
                match api.episode(&reply[0], self.season_number as u32, current_episode as u32) {
                    Ok(episode) => episode.episode_name,
                    Err(_) => {
                        println!("tv-renamer: episode '{}' does not exist", file.to_string_lossy());
                        current_episode += 1;
                        continue
                    }
                }
            } else {
                String::new()
            };

            // Obtain target destination.
            let new_destination = self.get_destination (
                directory, &file, current_episode, &title
            );

            // Print a message if the file already exists, else rename the file if dry-run is disabled.
            if fs::metadata(&new_destination).is_ok() {
                stdout.write(b"tv-renamer: ").try(STDOUT_ERROR, stderr);
                stdout.write(new_destination.to_string_lossy().as_bytes()).try(STDOUT_ERROR, stderr);
                stdout.write(b" already exists, not renaming.\n").try(STDOUT_ERROR, stderr);
                stdout.flush().try(STDOUT_ERROR, stderr);
            } else if !self.dry_run {
                fs::rename(&file, &new_destination).try(b"unable to rename file: ", stderr);
            }

            // If verbose or dry-run is enabled, print the change that occurred.
            if self.verbose | self.dry_run {
                stdout.write(b"\x1b[1m\x1b[32m").try(b"error writing to stdout: ", stderr);
                stdout.write(common::shorten_path(&file).to_string_lossy().as_bytes()).try(STDOUT_ERROR, stderr);
                stdout.write(b"\x1b[0m ->  \x1b[1m\x1b[32m").try(STDOUT_ERROR, stderr);
                stdout.write(common::shorten_path(&new_destination).to_string_lossy().as_bytes()).try(STDOUT_ERROR, stderr);
                stdout.write(b"\x1b[0m\n").try(STDOUT_ERROR, stderr);
                stdout.flush().try(STDOUT_ERROR, stderr);
            }

            // Write the changes to the log if logging is enabled.
            if self.log_changes {
                let mut log_path = env::home_dir().try(b"unable to get home directory: ", stderr);
                log_path.push("tv-renamer.log");
                let mut log = fs::OpenOptions::new().append(true).open(&log_path).try(b"unable to open log: ", stderr);
                log.write(common::shorten_path(&file).to_string_lossy().as_bytes()).try(LOG_ERROR, stderr);
                log.write(b" -> ").try(LOG_ERROR, stderr);
                log.write(common::shorten_path(&new_destination).to_string_lossy().as_bytes()).try(LOG_ERROR, stderr);
                log.write(b"\n").try(LOG_ERROR, stderr);
                log.flush().try(LOG_ERROR, stderr);
            }

            current_episode += 1;
        }
    }
}

pub fn launch() {
    let mut arguments = parse_arguments();
    arguments.execute();
}

fn parse_arguments() -> Arguments {
    let stdout = io::stdout();
    let stdout = &mut stdout.lock();
    let stderr = &mut io::stderr();

    // Initialize the default variables of the program.
    let mut program = Arguments {
        automatic:     false,
        dry_run:       false,
        log_changes:   false,
        no_name:       false,
        tvdb:          false,
        verbose:       false,
        directory:     String::new(),
        series_name:   String::new(),
        season_number: 1,
        episode_count: 1,
        pad_length:    2,
    };

    // Create a peekable iterator so that we can see the value of some options.
    // We will also need to ignore values that have already been read.
    let mut iterator = env::args().skip(1).peekable();
    let mut ignore = false;

    // Advance the iterator in a loop until all values have been read.
    while let Some(argument) = iterator.next() {
        if ignore { ignore = false; continue }
        if argument.starts_with('-') {
            match argument.as_str() {
                "-a" | "--automatic" => program.automatic = true,
                "-h" | "--help" => {
                    let _ = stdout.write(man::MAN_PAGE.as_bytes());
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
                "--no-name" => program.no_name = true,
                "-s" | "--season-number" => {
                    program.season_number = iterator.peek().try(SN_NO_VAL, stderr).parse::<usize>()
                        .try_and_ignore(SN_NAN, stderr);
                    ignore = true;
                },
                "-t" | "--tvdb" => program.tvdb = true,
                "-p" | "--pad-length" => program.pad_length = iterator.peek().try(PD_NO_VAL, stderr)
                    .parse::<usize>().try_and_ignore(PD_NAN, stderr),
                "-v" | "--verbose" => program.verbose = true,
                _ => abort_with_message(stderr, INVALID_ARG)
            }
        } else if program.directory.is_empty() {
            program.directory = argument;
        } else {
            abort_with_message(stderr, TOO_MANY_ARGS)
        }
    }

    // Set to current working directory if no directory argument is given.
    if program.directory.is_empty() {
        program.directory = String::from(env::current_dir().try_and_ignore(INVALID_CWD, stderr).to_str().unwrap());
    }

    // If no series name was given, ask for the name.
    if program.series_name.is_empty() && !program.no_name && !program.automatic {
        stdout.write(b"Please enter the name of the series: ").try(STDOUT_ERROR, stderr);
        stdout.flush().try(STDOUT_ERROR, stderr);
        io::stdin().read_line(&mut program.series_name).try(b"unable to read from stdin: ", stderr);
        program.series_name.pop().unwrap();
    }

    program
}

#[inline]
/// Print an error message and abort with an exit status of 1.
fn abort_with_message(stderr: &mut io::Stderr, message: &[u8]) {
    stderr.write(b"tv-renamer: ").try(b"error writing to stderr: ", stderr);
    stderr.write(message).try(b"error writing to stderr: ", stderr);
    stderr.flush().try(b"error writing to stderr: ", stderr);
    process::exit(1);
}
