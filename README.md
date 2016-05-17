**Build Status:** [![Build Status](https://travis-ci.org/mmstick/tv-renamer.png?branch=master)](https://travis-ci.org/mmstick/tv-renamer)

# Installation Instructions

With Rust installed, simply execute `cargo install --git https://github.com/mmstick/tv-renamer` to install with just CLI support. To enable the GTK3 front-end which may be called with the `--gui` flag, simply enable the `enable_gtk` feature with `cargo install --git https://github.com/mmstick/tv-renamer --features "enable_gtk"`.

# GTK3 Manual

![GTK3 Screenshot](screenshot-gtk3.png)

The use of this application should be fairly straightforward. However, ensure that the only files located in directories to be renamed are files that you want to be renamed as episodes in the series, and ensure that all of the episodes are in alphabetical order. The application does not derive the episode number from the episode name, but by their alphabetical order in the directory.

**Season Name** is where you will place the name of the series that you are renaming, which can be ignored if you have toggled **Automatic**. **Season Directory** is the base directory of the series if you are using **Automatic**, or the season directory where your episodes are stored. **Template** is where you will define your naming scheme, which by default is set to name files like so: "Season Name 1x01 Episode Title.mkv". **Season Number** and **Episode Number** are only used when **Automatic** is disabled, and it will define where to start counting from. **Log Changes** will simply log changes that have been performed on the disk.

The directory structure for **Automatic** should be as follows:

- Series directory <- directory to point to

  - Specials OR Season 0

    - Episodes...

  - Season 1

    - Episodes...

  - Season 2

    - Episodes...

# CLI Manual

If you need help with the usage of the CLI application, this manual page is also included in the program and is invokable with the -h and --help flags.

![CLI Screenshot](screenshot-cli.png)

## NAME:

**tv-renamer** - rename TV series and movies

## DESCRIPTION:

Renames all videos in a directory according to their season number and episode count. Please ensure that all of the files in the directory are files that you want renamed. It is recommended to use the dry-run option first before committing any changes.

If no DIRECTORY is given, the default path will be the current working directory. If a target file already exists, the command will skip the file.

## OPTIONS:

**-a, --automatic**: Automatically infer the season name and number based on the directory structure.

**-d, --dry-run:** Runs through all of the files and prints what would happen without doing anything.

**-l, --log-changes:** Log changes made to the disk to a file in your home directory.

**-n, --series-name:** Sets the name of the series to be renamed. [not optional]

**-s, --season-number:** Sets the season number to use when renaming a file. [default: 1]

**-t, --template:** Sets the template that will define the naming scheme. [default: "${Series} ${Season}x${Episode} ${TVDB_Title}"]

**-e, --episode-start:** Sets the episode number to start counting from. [default: 1]

**-p, --pad-length:** Sets the number of digits to pad the episode count for. [default: 2]

**-v, --verbose:** Print the changes that are occurring.

## EXAMPLE:

When executed inside of a directory with the name of the TV Series

```
one.mkv two.mkv three.mkv
> tv-renamer -n "series name"
"TV Series 1x01 Episode Title.mkv"
"TV Series 1x02 Episode Title.mkv"
"TV Series 1x03 Episode Title.mkv"
```

You can define your own naming scheme with --template:

```
> one.mkv two.mkv three.mkv
> tv-renamer -t "${Series} S${Season}E${Episode} - ${TVDB_Title}"
> "TV Series S1E01 - Episode Title.mkv" "TV Series S1E02 - Episode Title.mkv" "TV Series S1E03 - Episode Title.mkv"
```

The season name can also be automatically inferred:

```
"$series/Season1" "$series/Season2"
> tv-renamer "$series" -a OR cd $series && tv-renamer -a
"TV Series/Season1/TV Series 1x01.mkv"
...
"TV Series/Season2/TV Series 2x01.mkv"
...
```

Episode titles can also be pulled from the TVDB and added to the filenames.

```
> tv-renamer -a -t "${Series} ${Season}x${Episode} ${TVDB_Title}"
"TV Series/Season1/TV Series 1x01 Episode Title.mkv"
```

## AUTHOR:

Written by Michael Aaron Murphy.
