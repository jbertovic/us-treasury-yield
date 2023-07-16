use crate::{error::TreasuryCurveError, utility, MAX_FORWARD_DAYS};
use time::{ext::NumericalDuration, Date};

// implicit discriminator (starts at 0)
const CURVE_LENGTH: usize = 13;
pub const CURVE_LABELS: [&str; CURVE_LENGTH] = [
    "1 Mo", "2 Mo", "3 Mo", "4 Mo", "6 Mo", "1 Yr", "2 Yr", "3 Yr", "5 Yr", "7 Yr", "10 Yr",
    "20 Yr", "30 Yr",
];

/// Captures one curve for a single date
/// order of data matches 'CURVE_LABELS'
#[derive(Clone, Copy, Debug, Default)]
pub struct TreasuryCurve([Option<f64>; 13]);

impl TreasuryCurve {
    pub fn get_label(&self, label: &str) -> Result<Option<f64>, TreasuryCurveError> {
        match search_labels(label) {
            Some(index) => Ok(self.0[index]),
            None => Err(TreasuryCurveError::MissingLabel(label.to_string())),
        }
    }
}

/// stores the treasury curve in csv format as fetched from US Treasury website
pub struct TreasuryCurveCsv(pub String);

/// Hold Treasury Curve history
#[derive(Debug)]
/// curve history stored in reverse with latest at top
pub struct TreasuryCurveHistory {
    curves: Vec<TreasuryCurve>,
    dates: Vec<Date>,
}

impl TryFrom<TreasuryCurveCsv> for TreasuryCurveHistory {
    type Error = TreasuryCurveError;

    /// convert csv file with header
    fn try_from(value: TreasuryCurveCsv) -> Result<Self, Self::Error> {
        // check spacing relative to `CurveLocation`
        // if there is missing members than indicate None
        // any errors return an empty array
        let lines: Vec<&str> = value.0.split('\n').collect();
        // set flags based on headers - flag is 16 bits but only using first 13 to line up with labels
        let headers = lines[0].replace('\"', "");
        let headers: Vec<&str> = headers.split(',').collect();
        let flags = active_flags(&headers)?;
        // load data into vector of `TreasuryCuve`
        let curves: Vec<TreasuryCurve> = lines
            .iter()
            .skip(1)
            .map(|l| load_curve(l, &flags))
            .collect();
        let dates: Vec<Date> = lines.iter().skip(1).map(|l| load_date(l)).collect();
        let (dates, curves) = sort_arrays(dates, curves, false);

        Ok(TreasuryCurveHistory { curves, dates })
    }
}

impl TreasuryCurveHistory {
    /// grab the latest date in curve history
    pub fn latest(&self) -> (Date, TreasuryCurve) {
        (self.dates[0], self.curves[0])
    }

    /// grab the date specified or a date prior if curve does not exist for specified date
    /// allow 5 days after last published curve
    pub fn from_date(
        &self,
        request_date: Date,
    ) -> Result<(Date, TreasuryCurve), TreasuryCurveError> {
        // check that date request matches the year range of the data
        if request_date < *self.dates.last().unwrap()
            || request_date > (*self.dates.first().unwrap() + MAX_FORWARD_DAYS.days())
        {
            Err(TreasuryCurveError::OutsideDateRange(
                request_date.to_string(),
            ))
        } else {
            let index = self.closest_date(request_date);
            Ok((self.dates[index], self.curves[index]))
        }
    }

    // grab exact date or closest working backwards in time
    fn closest_date(&self, request_date: Date) -> usize {
        if request_date >= *self.dates.first().unwrap() {
            0
        } else if request_date <= *self.dates.last().unwrap() {
            self.dates.len() - 1
        } else {
            let mut index = 0;
            let mut found = false;
            while !found {
                index += 1;
                if self.dates[index] <= request_date {
                    found = true;
                }
            }
            index
        }
    }
}

// determine of 13 labels which ones are active and exist
fn active_flags(headers: &[&str]) -> Result<u16, TreasuryCurveError> {
    let mut flags = 0;
    // ignore first label as "DATE"
    for h in headers.iter().skip(1) {
        match search_labels(h) {
            Some(index) => flags |= 1 << index,
            None => return Err(TreasuryCurveError::MissingLabel(h.to_string())),
        }
    }
    Ok(flags)
}

fn search_labels(label: &str) -> Option<usize> {
    CURVE_LABELS.iter().position(|l| (*l).eq(label))
}

// TODO: Look for missing data where a column is missing half way through the year!
// load raw data into curve depending on which bits are active in flags
fn load_curve(data: &str, flags: &u16) -> TreasuryCurve {
    dbg!(data, flags);
    let mut data: Vec<Option<f64>> = data
        .split(',')
        .skip(1)
        .map(|d| Some(d.parse::<f64>().unwrap()))
        .collect();
    if u16::count_ones(*flags) != 13 {
        // search for zero bits in flag and shift data vector over
        for i in 0..CURVE_LENGTH {
            if (flags >> i) & 1 == 0 {
                data.insert(i, None);
            }
        }
    }
    // TODO: Remove Panic and introduce result return
    TreasuryCurve(
        data.as_slice()
            .try_into()
            .expect("data conversion for row doesn't equal CURVE_LENGTH"),
    )
}

fn load_date(data: &str) -> Date {
    let fd = utility::date_format_desc();
    //let fd = format_description::parse("[month]/[day]/[year]").unwrap();
    let string_date = data.split(',').next().unwrap();
    Date::parse(string_date, &fd).unwrap()
}

fn sort_arrays<C, D>(primary: Vec<D>, secondary: Vec<C>, ascending: bool) -> (Vec<D>, Vec<C>)
where
    D: Ord,
{
    // zip vectors, sort, unzip
    let mut zipped: Vec<_> = primary.into_iter().zip(secondary.into_iter()).collect();
    if ascending {
        zipped.sort_by(|a, b| a.0.cmp(&b.0));
    } else {
        zipped.sort_by(|a, b| b.0.cmp(&a.0));
    }
    let (sorted_primary, sorted_secondary): (Vec<_>, Vec<_>) = zipped.into_iter().unzip();

    (sorted_primary, sorted_secondary)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_csv_data() -> &'static str {
        r###""Date,"1 Mo","2 Mo","3 Mo","4 Mo","6 Mo","1 Yr","2 Yr","3 Yr","5 Yr","7 Yr","10 Yr","20 Yr","30 Yr"
07/07/2023,5.32,5.47,5.46,5.52,5.53,5.41,4.94,4.64,4.35,4.23,4.06,4.27,4.05
07/06/2023,5.32,5.47,5.46,5.52,5.54,5.44,4.99,4.68,4.37,4.22,4.05,4.23,4.01
07/05/2023,5.28,5.38,5.44,5.51,5.52,5.40,4.94,4.59,4.25,4.11,3.95,4.17,3.95
07/03/2023,5.27,5.40,5.44,5.52,5.53,5.43,4.94,4.56,4.19,4.03,3.86,4.08,3.87
06/30/2023,5.24,5.39,5.43,5.50,5.47,5.40,4.87,4.49,4.13,3.97,3.81,4.06,3.85
06/29/2023,5.25,5.40,5.46,5.51,5.50,5.41,4.87,4.49,4.14,3.99,3.85,4.11,3.92
06/28/2023,5.17,5.32,5.44,5.49,5.47,5.32,4.71,4.32,3.97,3.83,3.71,4.00,3.81
06/27/2023,5.17,5.31,5.44,5.44,5.46,5.33,4.74,4.38,4.02,3.90,3.77,4.03,3.84
06/26/2023,5.17,5.31,5.50,5.44,5.45,5.27,4.65,4.30,3.96,3.85,3.72,4.01,3.83"###
    }

    #[test]
    fn test_all_13_labels() {
        let headers = vec![
            "Date", "1 Mo", "2 Mo", "3 Mo", "4 Mo", "6 Mo", "1 Yr", "2 Yr", "3 Yr", "5 Yr", "7 Yr",
            "10 Yr", "20 Yr", "30 Yr",
        ];
        let flags = active_flags(&headers).unwrap();
        assert_eq!(flags, 0b1111111111111);
    }

    #[test]
    fn test_missing_labels() {
        let headers = vec![
            "Date", "1 Mo", "2 Mo", "3 Mo", "6 Mo", "1 Yr", "2 Yr", "5 Yr", "7 Yr", "10 Yr",
            "20 Yr", "30 Yr",
        ];
        let flags = active_flags(&headers).unwrap();
        assert_eq!(flags, 0b1111101110111);
    }

    #[test]
    fn test_unrecognized_labels() {
        // added unsupported label "9 Mo"
        let headers = vec![
            "Date", "1 Mo", "2 Mo", "3 Mo", "4 Mo", "9 Mo", "1 Yr", "2 Yr", "3 Yr", "5 Yr", "7 Yr",
            "10 Yr", "20 Yr", "30 Yr",
        ];
        let flags = active_flags(&headers);
        assert!(flags.is_err());
    }

    #[test]
    fn check_parsing_curve_data_into_treasurycurve() {
        let data = "07/07/2023,5.32,5.47,5.46,5.52,5.53,5.41,4.94,4.64,4.35,4.23,4.06,4.27,4.05";
        let flags: u16 = 0b1111111111111;
        let curve = load_curve(data, &flags);
        assert_eq!(curve.get_label("1 Mo").unwrap(), Some(5.32));
        assert_eq!(curve.get_label("30 Yr").unwrap(), Some(4.05));

        // data must be reduced to match number of flags or it will ***PANIC***
        let data = "07/07/2023,5.32,5.47,5.46,5.52,5.53,5.41,4.94,4.64,4.35,4.23";
        let missingflags: u16 = 0b1111111010101;
        let missingcurve = load_curve(data, &missingflags);
        assert_eq!(missingcurve.0[1], None);
        assert_eq!(missingcurve.0[3], None);
        assert_eq!(missingcurve.0[5], None);
    }

    #[test]
    fn check_parsing_curve_data_into_date() {
        let data = "07/10/2023,5.32,5.47,5.46,5.52,5.53,5.41,4.94,4.64,4.35,4.23,4.06,4.27,4.05";
        let date = load_date(data);
        assert_eq!(
            date,
            Date::from_calendar_date(2023, time::Month::July, 10).unwrap()
        );
    }

    #[test]
    fn check_data_on_old_curves() {
        // data from the year 2000
        let csvdata = r###"Date,"3 Mo","6 Mo","1 Yr","2 Yr","3 Yr","5 Yr","7 Yr","10 Yr","20 Yr","30 Yr"
12/29/2000,5.89,5.70,5.32,5.11,5.06,4.99,5.16,5.12,5.59,5.46
12/28/2000,5.87,5.79,5.40,5.18,5.12,5.02,5.21,5.13,5.59,5.44
12/27/2000,5.75,5.68,5.32,5.10,5.04,4.99,5.17,5.11,5.58,5.45
12/26/2000,5.85,5.76,5.31,5.10,5.00,4.92,5.09,5.04,5.54,5.41
12/22/2000,5.27,5.52,5.25,5.10,5.02,4.93,5.07,5.02,5.52,5.40
12/21/2000,5.38,5.64,5.33,5.14,5.04,4.94,5.10,5.03,5.53,5.41
12/20/2000,5.82,5.82,5.46,5.24,5.12,5.00,5.13,5.08,5.55,5.42
12/19/2000,5.93,5.93,5.58,5.35,5.23,5.12,5.22,5.19,5.61,5.47
12/18/2000,5.95,5.94,5.58,5.33,5.21,5.10,5.19,5.17,5.59,5.44
12/15/2000,6.02,5.99,5.65,5.38,5.26,5.15,5.24,5.20,5.59,5.44
12/14/2000,6.06,6.01,5.70,5.43,5.31,5.19,5.28,5.23,5.60,5.45
12/13/2000,6.06,6.03,5.74,5.45,5.34,5.24,5.33,5.29,5.64,5.48
12/12/2000,6.06,6.06,5.79,5.54,5.42,5.33,5.42,5.36,5.70,5.53
12/11/2000,6.08,6.06,5.79,5.52,5.43,5.33,5.42,5.37,5.71,5.54
12/08/2000,6.09,6.04,5.77,5.50,5.41,5.32,5.39,5.35,5.71,5.55"###;
        let tc = TreasuryCurveHistory::try_from(TreasuryCurveCsv(csvdata.to_string())).unwrap();
        let first_curve = tc.curves[0];
        assert_eq!(first_curve.get_label("1 Mo").unwrap(), None);
        assert_eq!(first_curve.get_label("2 Mo").unwrap(), None);
        assert_eq!(first_curve.get_label("3 Mo").unwrap(), Some(5.89));
        assert_eq!(first_curve.get_label("30 Yr").unwrap(), Some(5.46));
    }

    #[test]
    fn check_data_on_new_curves() {
        // data from year 2023
        let csvdata = new_csv_data();
        let tc = TreasuryCurveHistory::try_from(TreasuryCurveCsv(csvdata.to_string())).unwrap();
        let first_curve = tc.curves[0];
        assert_eq!(first_curve.0[0], Some(5.32));
        assert_eq!(first_curve.0[1], Some(5.47));
        assert_eq!(first_curve.0[2], Some(5.46));
        assert_eq!(first_curve.0[12], Some(4.05));
    }

    #[test]
    fn check_two_arrays_are_sorted_by_first_array() {
        let primary = vec!["07/01/2023", "07/10/2023", "06/25/2023", "08/01/2023"];
        let secondary = vec![3, 2, 4, 1];
        let fd = utility::date_format_desc();
        let primary: Vec<Date> = primary
            .into_iter()
            .map(|d| Date::parse(d, &fd).unwrap())
            .collect();
        dbg!(&primary, &secondary);
        let (primary, secondary) = sort_arrays(primary, secondary, false);
        dbg!(&primary, &secondary);
        assert_eq!(secondary, vec![1, 2, 3, 4]);
    }

    #[test]
    fn check_closest_date() {
        let csvdata = new_csv_data();
        let tc = TreasuryCurveHistory::try_from(TreasuryCurveCsv(csvdata.to_string())).unwrap();
        // date is below range
        assert_eq!(
            tc.closest_date(Date::from_calendar_date(2023, time::Month::June, 25).unwrap()),
            8
        );
        // date is exact
        assert_eq!(
            tc.closest_date(Date::from_calendar_date(2023, time::Month::July, 3).unwrap()),
            3
        );
        // date is above range
        assert_eq!(
            tc.closest_date(Date::from_calendar_date(2023, time::Month::July, 2).unwrap()),
            4
        );
        // date doesn't exist grab closest
        assert_eq!(
            tc.closest_date(Date::from_calendar_date(2023, time::Month::July, 10).unwrap()),
            0
        );
    }

    #[test]
    fn check_if_label_does_not_exist_throws_error() {
        let csvdata = new_csv_data();
        let tc = TreasuryCurveHistory::try_from(TreasuryCurveCsv(csvdata.to_string())).unwrap();
        let error_str = "does_not_exist".to_string();
        assert_eq!(
            tc.latest().1.get_label(&error_str).unwrap_err(),
            TreasuryCurveError::MissingLabel(error_str)
        );
    }
}
