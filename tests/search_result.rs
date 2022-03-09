use noted::{search_result::SearchResult, str};

macro_rules! test_format {
        ($name:ident, $($a:expr, $e:expr),+) => {
            #[test]
            fn $name() {
                $({
                let table = SearchResult::to_table($a);
                assert_eq!($e, table[1]);
                })*
            }
        };
    }

test_format!(
    format_short_filename_no_dots,
    vec![(str!("/temp/long/filename.md"), 1, str!("test"))],
    "/temp/long/filename.md         | 1    | test"
);

test_format!(
    format_long_filename_with_dots,
    vec![(
        str!("/temp/long/and/longer/or/evenlonger/filename.md"),
        1,
        str!("test")
    )],
    "...r/or/evenlonger/filename.md | 1    | test"
);

test_format!(
    format_line_number_four_digits,
    vec![(
        str!("/temp/long/and/longer/or/evenlonger/filename.md"),
        9999,
        str!("test"),
    )],
    "...r/or/evenlonger/filename.md | 9999 | test"
);

test_format!(
    format_line_number_five_digits,
    vec![(
        str!("/temp/long/and/longer/or/evenlonger/filename.md"),
        99999,
        str!("test"),
    )],
    "...r/or/evenlonger/filename.md | 99999 | test"
);

test_format!(
    format_long_line,
    vec![(
        str!("/temp/long/and/longer/or/evenlonger/filename.md"),
        9999,
        str!("Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum.")
    )],
    "...r/or/evenlonger/filename.md | 9999 | Lorem Ipsum is simply dummy text of the pr..."
);
