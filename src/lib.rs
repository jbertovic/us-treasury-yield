//! US Treseary Yield Curve
//! 
//! library fetches from https://home.treasury.gov/policy-issues/financing-the-government/interest-rate-statistics/legacy-interest-rate-xml-and-xsd-files
//! 
//! 
pub mod treasury_curve;
pub mod request;
pub mod error;

use error::TreasuryCurveError;
use time::{OffsetDateTime, Date};
use treasury_curve::{TreasuryCurve, TreasuryCurveCsv};
use request::fetch_csv_year;
use treasury_curve::TreasuryCurveHistory;

const MIN_YEAR_AVAIL: i32 = 1990;

pub fn fetch_latest() -> Result<(Date, TreasuryCurve), TreasuryCurveError> {
    TreasuryCurveHistory::try_from(TreasuryCurveCsv(fetch_csv_year(current_year())?))?.latest()
}

fn current_year() -> i32 {
    OffsetDateTime::now_utc().year()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_latest_treasury_curve() {
        let latest = fetch_latest();
        assert!(latest.is_ok());
    }

    #[test]
    fn fetch_date_treasury_curve() {
        todo!()
    }

    #[test]
    fn fetch_date_treasury_curve_date_does_not_exist() {
        todo!()
    }

    #[test]
    fn fetch_date_treasury_curve_data_point() {
        todo!()
    }

}
