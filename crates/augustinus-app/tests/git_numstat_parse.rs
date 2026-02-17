use augustinus_app::LocDelta;

#[test]
fn parses_numstat_and_ignores_binary_entries() {
    let output = "\
10\t2\tsrc/main.rs\n\
-\t-\timage.png\n\
3\t0\tsrc/lib.rs\n\
";
    let delta = LocDelta::parse_git_numstat(output);
    assert_eq!(delta.added, 13);
    assert_eq!(delta.removed, 2);
}

