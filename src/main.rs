use std::cmp::Ordering;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::Result;
use std::io::Write;

use bstr::ByteSlice;
use fastcmp::Compare;

fn mergediff(new: File, old: File, creates: File, updates: File, deletes: File) -> Result<()> {
    let mut creates = BufWriter::new(creates);
    let mut updates = BufWriter::new(updates);
    let mut deletes = BufWriter::new(deletes);

    // SAFETY: mmapping a file gives you no protection from the file changing out from under you.
    //         Theoretically this possibility is always present with files, however not having a
    //         buffer amplifies the chance.
    let new_file = unsafe { memmap::Mmap::map(&new)? };
    let old_file = unsafe { memmap::Mmap::map(&old)? };

    let mut new_iter = new_file.as_bstr().lines_with_terminator().peekable();
    let mut old_iter = old_file.as_bstr().lines_with_terminator().peekable();

    while let (Some(new), Some(old)) = (new_iter.peek(), old_iter.peek()) {
        if new.feq(old) {
            new_iter.next();
            old_iter.next();
            continue;
        }

        let new_pk = &new[0..new.as_bstr().find_byte(b',').unwrap_or(0)];
        let old_pk = &old[0..old.as_bstr().find_byte(b',').unwrap_or(0)];

        match new_pk.cmp(old_pk) {
            Ordering::Equal => {
                updates.write_all(new)?;
                new_iter.next();
                old_iter.next();
            }
            Ordering::Less => {
                creates.write_all(new)?;

                new_iter.next();
            }
            Ordering::Greater => {
                deletes.write_all(old)?;

                old_iter.next();
            }
        }
    }

    for line in old_iter {
        deletes.write_all(line)?;
    }

    for line in new_iter {
        creates.write_all(line)?;
    }

    // Flushing the buffers b/c buffers will automatically flush when they go out-of-scope,
    // however any errors that happen during flushing will silently fail.  The below code
    // lets us capture those errors.
    creates.flush()?;
    updates.flush()?;
    deletes.flush()?;

    Ok(())
}

#[test]
fn test_mergediff() {
    std::fs::create_dir_all("tmp/").expect("couldn't mkdir tmp/");

    mergediff(
        File::open("fixtures/target.csv").unwrap(),
        File::open("fixtures/original.csv").unwrap(),
        File::create("tmp/creates.csv").expect("couldn't open tmp/creates.csv for writing"),
        File::create("tmp/updates.csv").expect("couldn't open tmp/updates.csv for writing"),
        File::create("tmp/deletes.csv").expect("couldn't open tmp/deletes.csv for writing"),
    )
    .unwrap();

    fn contents(path: &str) -> String {
        std::fs::read_to_string(path).expect(&format!("couldn't read file: {path}"))
    }

    assert_eq!(
        contents("tmp/creates.csv"),
        contents("fixtures/creates.csv")
    );
    assert_eq!(
        contents("tmp/updates.csv"),
        contents("fixtures/updates.csv")
    );
    assert_eq!(
        contents("tmp/deletes.csv"),
        contents("fixtures/deletes.csv")
    );
}

fn main() {
    let paths: [String; 3] = std::env::args()
        .skip(1) // binary name always 0th arg
        .collect::<Vec<String>>()
        .try_into()
        .expect("requires three arguments as input:\n<newfile> <oldfile> <output dir>");

    let new_file = File::open(&paths[0]).expect("could not open first file");
    let old_file = File::open(&paths[1]).expect("could not open second file");

    let output_path = std::path::Path::new(&paths[2]);

    let [creates, updates, deletes] = ["creates.csv", "updates.csv", "deletes.csv"].map(|file| {
        OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(output_path.join(file))
            .unwrap_or_else(|err| panic!("couldn't open {} for writing: {}", file, err))
    });

    if let Err(err) = mergediff(new_file, old_file, creates, updates, deletes) {
        panic!("{}", err);
    }
}
