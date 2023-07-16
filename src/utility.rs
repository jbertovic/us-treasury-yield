use time::{
    format_description::{self, FormatItem},
    OffsetDateTime,
};

pub(crate) fn current_year() -> i32 {
    OffsetDateTime::now_utc().year()
}

pub(crate) fn date_format_desc() -> Vec<FormatItem<'static>> {
    format_description::parse("[month]/[day]/[year]").unwrap()
}
