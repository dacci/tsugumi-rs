# Simple EPUB builder

## Usage

```console
$ tsugumi --help
Simple EPUB builder

Usage: tsugumi <COMMAND>

Commands:
  new    Create a new book
  build  Build the current book
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```

```console
$ tsugumi new --help
Create a new book

Usage: tsugumi new [OPTIONS] [FILES]...

Arguments:
  [FILES]...  Create pages from files and set the first page as the cover page

Options:
  -t, --title <TITLE>     Set the main title of the book
  -a, --author <AUTHOR>   Set the author of the book
  -i, --identifier <URN>  Set the identifier of the book
  -h, --help              Print help information
```

```console
$ tsugumi build --help
Build the current book

Usage: tsugumi build [OPTIONS]

Options:
  -o, --output <PATH>  Output EPub file in PATH
  -h, --help           Print help information
```
