use crate::specification::{LineValue, LineValueIndicator};

use super::parse_from_str;

fn line_value(value: &str, indicator: LineValueIndicator) -> LineValue {
    LineValue {
        line_value: value.into(),
        indicator,
    }
}

#[test]
fn test_simple_file() {
    let file = "Some 
multiline
test comment
*** file1	2026-06-18 14:05:12.936105103 +0200
--- file2	2026-06-18 23:36:10.102603136 +0200
***************
*** 1,7 ****
- Please delete me
- delete me too
  Why do the lines above want to be deleted?
! I don't know
  Okay nevermind, here is a funny joke:
  Why do scientists never trust atoms?
    They make up everything!
--- 1,5 ----
  Why do the lines above want to be deleted?
! Which lines?
  Okay nevermind, here is a funny joke:
  Why do scientists never trust atoms?
    They make up everything!
***************
*** 9,11 ****
--- 7,12 ----
  I can tell more funny dad jokes!
  Why does the function have a bad day?
    It had 5 arguments
+ Okay okay, I'm sorry.
+ These jokes were very bad
+ Hopefully this example diff works
*** other file	2026-06-19 04:02:26.103103072 +0200
--- other/file	2026-06-19 08:29:42.921015162 +0200
***************
*** 1,2 ****
--- 1,3 ----
  This is some other file
  With two lines
+ And even a third line!
";

    let parsed = parse_from_str(file);
    match parsed {
        Ok(parsed) => {
            assert_eq!(parsed.comment, "Some \nmultiline\ntest comment");
            assert_eq!(parsed.diffs.len(), 2);

            let file1 = parsed.diffs.first().expect("Expected a FileDiff");
            assert_eq!(file1.from_header.file_path, "file1");
            assert_eq!(
                file1.from_header.modification_time.to_string(),
                "2026-06-18 14:05:12.936105103 +0200"
            );
            assert_eq!(file1.to_header.file_path, "file2");
            assert_eq!(file1.to_header.modification_time.to_string(), "2026-06-18 23:36:10.102603136 +0200");
            assert_eq!(file1.diffs.len(), 2);

            let diff1 = file1.diffs.first().expect("Expected a LocalDiff");
            assert_eq!(diff1.from_file_hunk_header.start_line, Some(1));
            assert_eq!(diff1.from_file_hunk_header.end_line, 7);
            assert_eq!(diff1.to_file_hunk_header.start_line, Some(1));
            assert_eq!(diff1.to_file_hunk_header.end_line, 5);
            assert_eq!(
                diff1.from_file_lines,
                vec![
                    line_value("Please delete me", LineValueIndicator::Deleted),
                    line_value("delete me too", LineValueIndicator::Deleted),
                    line_value("Why do the lines above want to be deleted?", LineValueIndicator::Unchanged),
                    line_value("I don't know", LineValueIndicator::Changed),
                    line_value("Okay nevermind, here is a funny joke:", LineValueIndicator::Unchanged),
                    line_value("Why do scientists never trust atoms?", LineValueIndicator::Unchanged),
                    line_value("  They make up everything!", LineValueIndicator::Unchanged),
                ]
            );
            assert_eq!(
                diff1.to_file_lines,
                vec![
                    line_value("Why do the lines above want to be deleted?", LineValueIndicator::Unchanged),
                    line_value("Which lines?", LineValueIndicator::Changed),
                    line_value("Okay nevermind, here is a funny joke:", LineValueIndicator::Unchanged),
                    line_value("Why do scientists never trust atoms?", LineValueIndicator::Unchanged),
                    line_value("  They make up everything!", LineValueIndicator::Unchanged),
                ]
            );

            let diff2 = file1.diffs.get(1).expect("Expected a LocalDiff");
            assert_eq!(diff2.from_file_hunk_header.start_line, Some(9));
            assert_eq!(diff2.from_file_hunk_header.end_line, 11);
            assert_eq!(diff2.to_file_hunk_header.start_line, Some(7));
            assert_eq!(diff2.to_file_hunk_header.end_line, 12);
            assert_eq!(diff2.from_file_lines, vec![]);
            assert_eq!(
                diff2.to_file_lines,
                vec![
                    line_value("I can tell more funny dad jokes!", LineValueIndicator::Unchanged),
                    line_value("Why does the function have a bad day?", LineValueIndicator::Unchanged),
                    line_value("  It had 5 arguments", LineValueIndicator::Unchanged),
                    line_value("Okay okay, I'm sorry.", LineValueIndicator::Inserted),
                    line_value("These jokes were very bad", LineValueIndicator::Inserted),
                    line_value("Hopefully this example diff works", LineValueIndicator::Inserted),
                ]
            );

            let file2 = parsed.diffs.get(1).expect("Expected a FileDiff");
            assert_eq!(file2.from_header.file_path, "other file");
            assert_eq!(
                file2.from_header.modification_time.to_string(),
                "2026-06-19 04:02:26.103103072 +0200"
            );
            assert_eq!(file2.to_header.file_path, "other/file");
            assert_eq!(file2.to_header.modification_time.to_string(), "2026-06-19 08:29:42.921015162 +0200");
            assert_eq!(file2.diffs.len(), 1);

            let diff1 = file2.diffs.first().expect("Expected a LocalDiff");
            assert_eq!(diff1.from_file_hunk_header.start_line, Some(1));
            assert_eq!(diff1.from_file_hunk_header.end_line, 2);
            assert_eq!(diff1.to_file_hunk_header.start_line, Some(1));
            assert_eq!(diff1.to_file_hunk_header.end_line, 3);
            assert_eq!(diff1.from_file_lines, vec![]);
            assert_eq!(
                diff1.to_file_lines,
                vec![
                    line_value("This is some other file", LineValueIndicator::Unchanged),
                    line_value("With two lines", LineValueIndicator::Unchanged),
                    line_value("And even a third line!", LineValueIndicator::Inserted),
                ]
            );
        },
        Err(e) => panic!("Expected Ok(ContextDiffFile {{ ... }}), got Err({e:?})"),
    }
}
