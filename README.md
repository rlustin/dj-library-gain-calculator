# dj-library-gain-calculator

Calculates gains in a Traktor DJ library to have a constant loudness.

This application is able to calculate the loudness of each track of your Traktor DJ
library, to be able to normalize audio so each piece of your DJ set sounds roughly
the same volume to the human ear.

It uses the [EBU R128 loudness standard](https://tech.ebu.ch/docs/r/r128.pdf) under the wood.

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

To benefit from this analysis, you should use the
[Autogain](https://support.native-instruments.com/hc/en-us/articles/209551129-How-to-Set-the-Channel-Gain-and-Autogain-in-TRAKTOR-PRO-2)
feature of Traktor.

## Development

You can use [cargo](https://doc.rust-lang.org/stable/cargo/) commands to build and run
the application.
