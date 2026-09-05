use crate::{parser::parse_from_str, translator};

fn assert_parse_translate(context_diff: &str, unified_diff: &str) {
    match parse_from_str(context_diff) {
        Ok(parsed) => {
            let translated = translator::translate_to_unified_diff(parsed);

            assert_eq!(translated, unified_diff);
        },
        Err(e) => panic!("Expected Ok(ContextDiffFile {{ ... }}), got Err({e:?})"),
    }
}

#[test]
fn test_no_comment() {
    let context_diff = "*** file1	2026-06-18 14:05:12.936105103 +0200
--- file2	2026-06-18 23:36:10.102603136 +0200
***************
*** 1,2 ****
--- 1,3 ----
  This is some file
  With two lines
+ And even a third line!
";

    let unified_diff = "--- file1	2026-06-18 14:05:12.936105103 +0200
+++ file2	2026-06-18 23:36:10.102603136 +0200
@@ -1,2 +1,3 @@
 This is some file
 With two lines
+And even a third line!
";

    assert_parse_translate(context_diff, unified_diff);
}

#[test]
fn test_new_file() {
    let context_diff = "*** /dev/null	2026-06-18 14:05:12.936105103 +0200
--- file	2026-06-18 23:36:10.102603136 +0200
***************
*** 0 ****
--- 1,3 ----
+ This is a new file
+ With two lines
+ And even a third line!
";

    let unified_diff = "--- /dev/null	2026-06-18 14:05:12.936105103 +0200
+++ file	2026-06-18 23:36:10.102603136 +0200
@@ -0,0 +1,3 @@
+This is a new file
+With two lines
+And even a third line!
";

    assert_parse_translate(context_diff, unified_diff);
}

#[test]
fn test_delete_file() {
    let context_diff = "*** file	2026-06-18 14:05:12.936105103 +0200
--- /dev/null	2026-06-18 23:36:10.102603136 +0200
***************
*** 1,3 ****
- This file will be deleted
- And the parser will parse the
- diff of the deletion correctly
--- 0 ----
";

    let unified_diff = "--- file	2026-06-18 14:05:12.936105103 +0200
+++ /dev/null	2026-06-18 23:36:10.102603136 +0200
@@ -1,3 +0,0 @@
-This file will be deleted
-And the parser will parse the
-diff of the deletion correctly
";

    assert_parse_translate(context_diff, unified_diff);
}
