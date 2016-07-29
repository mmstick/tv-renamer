pub const MAN_PAGE: &'static str = r#"NAME:
    tv-renamer - rename TV series and movies

SYNOPSIS:
    tv-renamer DIRECTORY [-d | --dry-run]
                         [-n | --series-name "NAME OF SERIES"]
                         [-s | --season-number NUMBER]
                         [-t | --template "TEMPLATE"]
                         [-p | --pad-length NUMBER]
                         [-e | --episode-start NUMBER]

DESCRIPTION:
    Renames all videos in a directory according to their season and episode.

    If the given DIRECTORY contains season directories, it will automatically rename episodes in each season.

    If no DIRECTORY is given, the default path will be the current working directory.

    It is recommended to use the dry-run option first before committing any changes.

    If a target file already exists, the command will ask if it is okay to overwrite the file.

    Please ensure that all of the files in the directory are video files that you want renamed.

OPTIONS:
    -d, --dry-run:
        Runs through all of the files and prints what would happen without doing anything.

    -n, --series-name:
        Sets the name of the series to be renamed. [not optional]

    -s, --season-number:
        Sets the season number to use when renaming a file. [default: 1]

    -t, --template:
        Sets the template that will define the naming scheme.
        [default: "${Series} - ${Season}x${Episode} - ${TVDB_Title}"]

    -e, --episode-start:
        Sets the episode number to start counting from. [default: 1]

    -p, --pad-length:
        Sets the number of digits to pad the episode count for. [default: 2]

    -v, --verbose:
        Print the changes that are occurring.

EXAMPLE:
    When executed inside of a directory with the name of the TV Series
        > one.mkv two.mkv three.mkv
        > tv-renamer
        > "TV Series - 1x01 - Episode Title.mkv"
        > "TV Series - 1x02 - Episode Title.mkv"
        > "TV Series - 1x03 - Episode Title.mkv"

    You can define your own naming scheme with --template:
        > one.mkv two.mkv three.mkv
        > tv-renamer -t "${Series} S${Season}E${Episode} - ${TVDB_Title}"
        > "TV Series S1E01 - Episode Title.mkv"
        > "TV Series S1E02 - Episode Title.mkv"
        > "TV Series S1E03 - Episode Title.mkv"

    The season name can also be automatically inferred:
        > "TV Series/Season1"
        > "TV Series/Season2"
        > tv-renamer "TV Series"
        > "TV Series/Season1/TV Series - 1x01 - Episode Title.mkv"
        > "TV Series/Season2/TV Series - 2x01 - Episode Title.mkv"

AUTHOR:
    Written by Michael Aaron Murphy.
"#;
