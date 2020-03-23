# dj-library-gain-calculator

Analyses all tracks in a Traktor DJ library to have constant loudness.

This application computes the loudness of each track of your Traktor DJ library,
and normalize audio so each track of your DJ set are at roughly the same perceptual level.

Normalization is performed using the [EBU R128 loudness standard](https://tech.ebu.ch/docs/r/r128.pdf) under the hood.

## Usage

```bash
dj-library-gain-calculator --help
$ dj-library-gain-calculator 0.1.0
Calculates gains in a Traktor DJ library to have a constant loudness.

USAGE:
    dj-library-gain-calculator [FLAGS] [OPTIONS] --input <input>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -w, --write      Updates the Traktor library in place.

OPTIONS:
    -i, --input <input>      The input Traktor library file to use.
    -o, --output <output>    The output Traktor library file to write or - for stdout.
```

Example to update your Traktor library in place:

```bash
dj-library-gain-calculator --input ~/Documents/Native\ Instruments/Traktor\ 3.3.0/collection.nml --write
```

This analysis only works if
[Autogain](https://support.native-instruments.com/hc/en-us/articles/209551129-How-to-Set-the-Channel-Gain-and-Autogain-in-TRAKTOR-PRO-2)
is enabled in Traktor.

## Development

Use [cargo](https://doc.rust-lang.org/stable/cargo/) commands to build and run the application.
