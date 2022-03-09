use noted::{vec_of_strings, note::Note};


macro_rules! test_note {
        ($name:ident, $($a:expr, $e:expr),+) => {
            #[test]
            fn $name() {
                $({
                    let note = Note::from($a);
                    assert_eq!($e, note);
                })*
            }
        };
    }

test_note!(
    from_empty_vec_should_be_default,
    Vec::new(),
    Note::default()
);

test_note!(
    from_one_argument_should_create_note_without_tags,
    ["Sample note"].to_vec(),
    Note {
        content: "Sample note".to_string(),
        ..Default::default()
    }
);

test_note!(
    from_two_arguments_should_create_note_with_tag,
    ["Sample note", "tag1"].to_vec(),
    Note {
        content: "Sample note".to_string(),
        tags: vec_of_strings!("tag1")
    }
);

test_note!(
    from_multiple_arguments_should_create_note_with_multiple_tags,
    ["Sample note", "tag1", "tag2", "tag3", "tag4"].to_vec(),
    Note {
        content: "Sample note".to_string(),
        tags: vec_of_strings!("tag1", "tag2", "tag3", "tag4")
    }
);
