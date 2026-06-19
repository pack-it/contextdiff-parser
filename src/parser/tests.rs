use crate::specification::{LineValue, LineValueIndicator};

use super::parse_from_str;

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
            assert_eq!(file1.hunks.len(), 2);

            let hunk1 = file1.hunks.first().expect("Expected a LocalDiff");
            assert_eq!(hunk1.from_file_header.start_line, Some(1));
            assert_eq!(hunk1.from_file_header.end_line, 7);
            assert_eq!(hunk1.to_file_header.start_line, Some(1));
            assert_eq!(hunk1.to_file_header.end_line, 5);
            assert_eq!(
                hunk1.from_file_lines,
                vec![
                    LineValue::new("Please delete me", LineValueIndicator::Deleted),
                    LineValue::new("delete me too", LineValueIndicator::Deleted),
                    LineValue::new("Why do the lines above want to be deleted?", LineValueIndicator::Unchanged),
                    LineValue::new("I don't know", LineValueIndicator::Changed),
                    LineValue::new("Okay nevermind, here is a funny joke:", LineValueIndicator::Unchanged),
                    LineValue::new("Why do scientists never trust atoms?", LineValueIndicator::Unchanged),
                    LineValue::new("  They make up everything!", LineValueIndicator::Unchanged),
                ]
            );
            assert_eq!(
                hunk1.to_file_lines,
                vec![
                    LineValue::new("Why do the lines above want to be deleted?", LineValueIndicator::Unchanged),
                    LineValue::new("Which lines?", LineValueIndicator::Changed),
                    LineValue::new("Okay nevermind, here is a funny joke:", LineValueIndicator::Unchanged),
                    LineValue::new("Why do scientists never trust atoms?", LineValueIndicator::Unchanged),
                    LineValue::new("  They make up everything!", LineValueIndicator::Unchanged),
                ]
            );

            let hunk2 = file1.hunks.get(1).expect("Expected a LocalDiff");
            assert_eq!(hunk2.from_file_header.start_line, Some(9));
            assert_eq!(hunk2.from_file_header.end_line, 11);
            assert_eq!(hunk2.to_file_header.start_line, Some(7));
            assert_eq!(hunk2.to_file_header.end_line, 12);
            assert_eq!(hunk2.from_file_lines, vec![]);
            assert_eq!(
                hunk2.to_file_lines,
                vec![
                    LineValue::new("I can tell more funny dad jokes!", LineValueIndicator::Unchanged),
                    LineValue::new("Why does the function have a bad day?", LineValueIndicator::Unchanged),
                    LineValue::new("  It had 5 arguments", LineValueIndicator::Unchanged),
                    LineValue::new("Okay okay, I'm sorry.", LineValueIndicator::Inserted),
                    LineValue::new("These jokes were very bad", LineValueIndicator::Inserted),
                    LineValue::new("Hopefully this example diff works", LineValueIndicator::Inserted),
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
            assert_eq!(file2.hunks.len(), 1);

            let hunk1 = file2.hunks.first().expect("Expected a LocalDiff");
            assert_eq!(hunk1.from_file_header.start_line, Some(1));
            assert_eq!(hunk1.from_file_header.end_line, 2);
            assert_eq!(hunk1.to_file_header.start_line, Some(1));
            assert_eq!(hunk1.to_file_header.end_line, 3);
            assert_eq!(hunk1.from_file_lines, vec![]);
            assert_eq!(
                hunk1.to_file_lines,
                vec![
                    LineValue::new("This is some other file", LineValueIndicator::Unchanged),
                    LineValue::new("With two lines", LineValueIndicator::Unchanged),
                    LineValue::new("And even a third line!", LineValueIndicator::Inserted),
                ]
            );
        },
        Err(e) => panic!("Expected Ok(ContextDiffFile {{ ... }}), got Err({e:?})"),
    }
}
