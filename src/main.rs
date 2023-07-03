use std::cmp::Ordering;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Result;
use std::io::Write;

fn mergediff(new: File, old: File, creates: File, updates: File, deletes: File) -> Result<()> {
    let mut creates = BufWriter::new(creates);
    let mut updates = BufWriter::new(updates);
    let mut deletes = BufWriter::new(deletes);

    let mut new = BufReader::new(new).lines().peekable();
    let mut old = BufReader::new(old).lines().peekable();

    while let (Some(new_line), Some(old_line)) = (new.peek(), old.peek()) {
        let new_line = new_line.as_ref().expect("error reading line from newfile");
        let old_line = old_line.as_ref().expect("error reading line from oldfile");

        if new_line == old_line {
            new.next();
            old.next();
            continue;
        }

        let (new_pk, _) = new_line.split_once(',').unwrap_or((new_line, ""));
        let (old_pk, _) = old_line.split_once(',').unwrap_or((old_line, ""));

        match new_pk.cmp(old_pk) {
            Ordering::Equal => {
                writeln!(updates, "{}", new_line)?;
                new.next();
                old.next();
            }
            Ordering::Less => {
                writeln!(creates, "{}", new_line)?;
                new.next();
            }
            Ordering::Greater => {
                writeln!(deletes, "{}", old_line)?;
                old.next();
            }
        }
    }

    for line in old {
        writeln!(deletes, "{}", line.unwrap())?;
    }

    for line in new {
        writeln!(creates, "{}", line.unwrap())?;
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
        std::fs::read_to_string(path).expect("couldn't read file")
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

#[test]
fn test_single_column_merge_diff() {
    std::fs::create_dir_all("tmp/").expect("couldn't mkdir tmp/");

    mergediff(
        File::open("fixtures/single-column/target.csv").unwrap(),
        File::open("fixtures/single-column/original.csv").unwrap(),
        File::create("tmp/creates_2.csv").expect("couldn't open tmp/creates.csv for writing"),
        File::create("tmp/updates_2.csv").expect("couldn't open tmp/updates.csv for writing"),
        File::create("tmp/deletes_2.csv").expect("couldn't open tmp/deletes.csv for writing"),
    )
    .expect("it should succeed.");

    fn contents(path: &str) -> String {
        std::fs::read_to_string(path).expect("couldn't read file")
    }

    assert_eq!(
        contents("tmp/creates_2.csv"),
        contents("fixtures/single-column/creates.csv")
    );
    assert_eq!(
        contents("tmp/updates_2.csv"),
        contents("fixtures/single-column/updates.csv")
    );
    assert_eq!(
        contents("tmp/deletes_2.csv"),
        contents("fixtures/single-column/deletes.csv")
    );
}

fn main() {
    let args: [String; 3] = std::env::args()
        .skip(1) // binary name always 0th arg
        .collect::<Vec<String>>()
        .try_into()
        .expect("requires three arguments as input:\n<newfile> <oldfile> <output dir>");

    let new_file = File::open(&args[0]).expect("could not open first file");
    let old_file = File::open(&args[1]).expect("could not open second file");

    let output_path = std::path::Path::new(&args[2]);

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
