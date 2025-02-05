use time::error::Parse;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;
use time::PrimitiveDateTime;

const DATE_TIME_FORMAT: &[BorrowedFormatItem] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

pub fn parse_date_time(t: &str) -> Result<PrimitiveDateTime, Parse> {
    PrimitiveDateTime::parse(t, &DATE_TIME_FORMAT)
}
