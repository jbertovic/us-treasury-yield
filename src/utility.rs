use time::OffsetDateTime;

pub(crate) fn current_year() -> i32 {
    OffsetDateTime::now_utc().year()
}
