# dj-library-gain-calculator

Analyses all tracks in a Traktor DJ library to have constant loudness.

This application computes the loudness of each track of your Traktor DJ library,
and normalize audio so each track of your DJ set are at roughly the same perceptual level.

Normalization is performed using the [TECH 3341 loudness metering](https://tech.ebu.ch/docs/tech/tech3341.pdf)
under the hood.

## Usage

```bash
$ dj-library-gain-calculator --help
dj-library-gain-calculator 0.1.0
Analyses all tracks in a Traktor DJ library to have constant loudness.

USAGE:
    dj-library-gain-calculator [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help       Prints this message or the help of the given subcommand(s)
    library    Analyses all tracks in a Traktor DJ library to have constant loudness.
    track      Analyse a track or set of tracks and output loudness and peak info.
```

### Analyze a Traktor DJ library

```bash
$ dj-library-gain-calculator collection --help
dj-library-gain-calculator-collection
Analyses all tracks in a Traktor DJ collection to have constant loudness.

USAGE:
    dj-library-gain-calculator collection [FLAGS] [OPTIONS] --input <input>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -w, --write      Updates the Traktor collection in place.

OPTIONS:
    -i, --input <input>      The input Traktor collection file to use.
    -o, --output <output>    The output Traktor collection file to write or - for stdout.
    -t, --target <target>    Target loudness in dB LUFS (negative value). [default: -14.0]
```

#### Example to update your Traktor library in place:

```bash
dj-library-gain-calculator library --input ~/Documents/Native\ Instruments/Traktor\ 3.3.0/collection.nml --write
```

This analysis only works if
[Autogain](https://support.native-instruments.com/hc/en-us/articles/209551129-How-to-Set-the-Channel-Gain-and-Autogain-in-TRAKTOR-PRO-2)
is enabled in Traktor.

###  Analyze a track

```bash
$ dj-library-gain-calculator scanner --help
dj-library-gain-calculator-scanner
Analyses a track or set of tracks and output loudness and peak info.

USAGE:
    dj-library-gain-calculator scanner <input>...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <input>...    One or more files to analyse.
```

## Development

Use [cargo](https://doc.rust-lang.org/stable/cargo/) commands to build and run the application.
