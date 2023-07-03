# `mergediff`

Mergediff is a CLI utility to compare two presorted CSV files and output the differences. It pairs well with the output of `sort`, if the files are not presorted. It compares to the tool `diff` but does not load the files into memory, allowing for comparison of very large files. Incidentally it is also generally faster, see [performance](#performance) for more information.

## Usage

Mergediff will append creates (lines in file1 but not in file2), deletes (lines in file2 but not file1), and updates (lines whose primary key is in both files but where other columns differ) to the output directory. The output files will be named `creates.csv`, `deletes.csv`, and `updates.csv` respectively.

```bash
mergediff <file1> <file2> <output dir>
```

## Performance

Mergediff is benchmarked primarily against `diff`.

Why mergediff is fast, non-exhaustive, in approximate order of importance:

* **Assuming presorted input**: Mergediff can avoid loading the entire file into memory, saving the initial load step and keeping the memory footprint low.
* **Avoiding allocations**: While the rust standard library provides an excellent [`.lines()`](https://doc.rust-lang.org/std/io/trait.BufRead.html#method.lines) iterator, it allocates a new `String` for each line, which is massively slow.
  * Interestingly, Rust has since shipped Generic Associated Types, which I believe would allow for a `.lines()` implementation that returns a `&str` instead of a `String`. While it seems unlikely that the stdlib would break the API of `BufRead` to add this, it could be a good opportunity for a third-party crate.
  * Sidebar: this tool was written to replace a Ruby equivalent; even the idiomatic, overallocating stdlib rust implementation was 6x faster.)
* **Use of `bstr`**: Mergediff uses the [bstr](https://crates.io/crates/bstr) crate extracted from the (paradigm-changingly-good) [`ripgrep`](https://github.com/BurntSushi/ripgrep) tool to search for newlines in the input.
  * We can skip utf8 validation, which isn't as expensive as you'd first expect, but it still unnecessary overhead.
  * `bstr` provides a fast implementation of `find_byte`, which is used to find newlines and primary keys.