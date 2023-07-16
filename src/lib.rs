//! US Treseary Yield Curve
//!
//! library fetches from <https://home.treasury.gov/resource-center/data-chart-center/interest-rates/daily-treasury-rates.csv/>
//!
//!
//! you can access API by:
//! 1) one time fetch with [`fetch_latest`] or [`fetch_date`]
//! 2) grab a year of data with [`fetch_year`] and then use pub functions on [`TreasuryCurveHistory`]
//!
//! TODO: Timeout on fetching data -> timeout to retry twice and then throw error
pub mod error;
mod request;
pub mod treasury_curve;
mod utility;

use error::TreasuryCurveError;
use request::fetch_csv_year;
use time::Date;
use treasury_curve::TreasuryCurveHistory;
use treasury_curve::{TreasuryCurve, TreasuryCurveCsv};
use utility::current_year;

const MIN_YEAR_AVAIL: i32 = 1990;
const MAX_FORWARD_DAYS: i64 = 5;

/// fetch the latest date of the Tresury Curve
pub fn fetch_latest() -> Result<(Date, TreasuryCurve), TreasuryCurveError> {
    Ok(fetch_year(current_year())?.latest())
}

/// fetch a specific date of the Tresury curve
/// Defaults to the last known data point on weekend and holidays
pub fn fetch_date(request_date: Date) -> Result<(Date, TreasuryCurve), TreasuryCurveError> {
    fetch_year(request_date.year())?.from_date(request_date)
}

/// fetch an entire year of Treasury curves
pub fn fetch_year(requst_year: i32) -> Result<TreasuryCurveHistory, TreasuryCurveError> {
    TreasuryCurveHistory::try_from(TreasuryCurveCsv(fetch_csv_year(requst_year)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::ext::NumericalDuration;

    #[test]
    fn fetch_latest_treasury_curve() {
        let latest = fetch_latest();
        assert!(latest.is_ok());
    }

    #[test]
    fn fetch_date_treasury_curve() {
        // data exists on this day
        let exist_date = Date::from_calendar_date(2023, time::Month::July, 5).unwrap();
        assert_eq!(fetch_date(exist_date).unwrap().0, exist_date);
        // data does not exist on this day Jul 2 is a weeekend -> use the day prior which is June 30
        let nonexist_date = Date::from_calendar_date(2023, time::Month::July, 2).unwrap();
        let nonexist_date_check = Date::from_calendar_date(2023, time::Month::June, 30).unwrap();
        assert_eq!(fetch_date(nonexist_date).unwrap().0, nonexist_date_check);
    }

    #[test]
    fn fetch_date_treasury_curve_date_does_not_exist() {
        let date_check = time::OffsetDateTime::now_utc().date();
        let one_year_forward = date_check + 365.days();
        let max_days_forward = date_check + (MAX_FORWARD_DAYS + 1).days();
        assert_eq!(
            fetch_date(one_year_forward).unwrap_err(),
            TreasuryCurveError::InvalidYear(one_year_forward.year())
        );
        assert_eq!(
            fetch_date(max_days_forward).unwrap_err(),
            TreasuryCurveError::OutsideDateRange(max_days_forward.to_string())
        );
    }

    #[test]
    fn fetch_date_treasury_curve_for_various_test_dates() {
        let fd = utility::date_format_desc();
        // pick 6 dates in history and check each date against 2 points in the curve
        // (1)use old dates, (2) weekend, (3) holiday, (4) recent date, (5) end of year, (6) beginning of year
        //        let fetch_dates = ["04/22/1999", "03/19/2011", "07/04/2020", "06/01/2023", "12/31/2022", "01/02/2023"];
        let fetch_dates = ["12/31/2022"];
        let fetch_dates: Vec<Date> = fetch_dates
            .iter()
            .map(|d| Date::parse(d, &fd).unwrap())
            .collect();
        //        let fetch_labels = ["1 Yr", "1 Mo", "6 Mo", "20 Yr", "2 Mo", "5 Yr"];
        let fetch_labels = ["2 Mo"];
        //        let test_dates = ["04/22/1999", "03/18/2011", "07/02/2020", "06/01/2023", "12/30/2022", "12/30/2022"];
        let test_dates = ["12/30/2022"];
        let test_dates: Vec<Date> = test_dates
            .iter()
            .map(|d| Date::parse(d, &fd).unwrap())
            .collect();
        let test_results = [4.73, 0.06, 0.16, 3.98, 4.41, 3.99];

        let mut date_results: Vec<Date> = vec![];
        let mut curve_results: Vec<f64> = vec![];

        for (i, d) in fetch_dates.iter().enumerate() {
            println!("Working on : {d}");
            match fetch_date(*d) {
                Ok((date, curve)) => {
                    date_results.push(date);
                    curve_results.push(curve.get_label(fetch_labels[i]).unwrap().unwrap());
                }
                Err(e) => {
                    println!("Error on date: {d}");
                    println!("Err({e:?}");
                }
            }
        }

        assert_eq!(date_results.as_slice(), test_dates);
        assert_eq!(curve_results.as_slice(), test_results);
    }
}
