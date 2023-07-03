# `mergediff`

Mergediff is a CLI utility to compare two presorted CSV files and output the differences. It pairs well with the output of `sort`, if the files are not presorted. It compares to the tool `diff` but does not load the files into memory, allowing for comparison of very large files.

## Usage

Mergediff will append creates (lines in file1 but not in file2), deletes (lines in file2 but not file1), and updates (lines whose primary key is in both files but where other columns differ) to the output directory. The output files will be named `creates.csv`, `deletes.csv`, and `updates.csv` respectively.

```bash
mergediff <file1> <file2> <output dir>
```
