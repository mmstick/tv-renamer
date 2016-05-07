pub const MAN_PAGE: &'static str = r#"
NAME:
    tv-renamer - rename TV series and movies

SYNOPSIS:
    tv-renamer DIRECTORY [-a | --automatic]
                           [-d | --dry-run]
                           [--no-name]
                           [-l | --log-changes]
                           [-n | --series-name "NAME OF SERIES"]
                           [-s | --season-number NUMBER]
                           [-t | --tvdb]
                           [-p | --pad-length NUMBER]
                           [-e | --episode-start NUMBER]

DESCRIPTION:
    Renames all videos in a directory according to their season number and episode count.

    Please ensure that all of the files in the directory are files that you want renamed.

    It is recommended to use the dry-run option first before committing any changes.

    If no DIRECTORY is given, the default path will be the current working directory.

    If a target file already exists, the command will skip the file.

OPTIONS:
    -a, --automatic:
        Automatically infer the season name and number based on the directory structure.

    -d, --dry-run:
        Runs through all of the files and prints what would happen without doing anything.

    -l, --log-changes:
        Log changes made to the disk to a file in your home directory.

    -n, --series-name:
        Sets the name of the series to be renamed. [not optional]

    --no-name:
        Disables writing the name of the series when renaming.

    -s, --season-number:
        Sets the season number to use when renaming a file. [default: 1]

    -t, --tvdb:
        Append the episode title from TVDB to each episode.

    -e, --episode-start:
        Sets the episode number to start counting from. [default: 1]

    -p, --pad-length:
        Sets the number of digits to pad the episode count for. [default: 2]

    -v, --verbose:
        Print the changes that are occurring.

EXAMPLE:
    When executed inside of a directory with the name of the TV Series
        > one.mkv two.mkv three.mkv
        > tv-renamer -n "Name of Season"
        > "TV Series 1x01.mkv" "TV Series 1x02.mkv" "TV Series 1x03.mkv"

    If you do not want to have the series name added to the episodes:
        > one.mkv two.mkv three.mkv
        > tv-renamer --no-name
        > "1x01.mkv" "1x02.mkv" "1x03.mkv

    The season name can also be automatically inferred:
        > "TV Series/Season1"
        > "TV Series/Season2"
        > tv-renamer "TV Series" -a
        > "TV Series/Season1/TV Series 1x01.mkv" ... "TV Series/Season2/TV Series 2x01.mkv" ...

AUTHOR:
    Written by Michael Aaron Murphy.
"#;
