use crate::{DateInfo, FetchError};
use crate::Result;
use log::debug;
use reqwest::header::COOKIE;

#[derive(Debug, PartialEq)]
pub enum Stars
{
    Zero,
    One,
    Two,
}

pub fn get_stars(task: DateInfo, session: &str) -> Result<Stars> {
    debug!("Fetching star info for day {}-{}", task.day, task.year);

    let url = format!("https://adventofcode.com/{}/day/{}", task.year, task.day);
    let input = reqwest::blocking::Client::new()
        .get(&url)
        .header(COOKIE, format!("session={}", session))
        .send();

    if input.is_err() {
        return Err(FetchError::Cause("Error sending GET request".to_string()));
    }

    let input = input.unwrap().text().unwrap();

    if input.contains("Please don't repeatedly request") || input.contains("Not Found") {
        return Err(FetchError::Cause(format!(
            "Puzzle for day {} is not live yet",
            task.day
        )));
    }

    if input.contains("The first half of this puzzle is complete!") {
        return Ok(Stars::One);
    }

    if input.contains("Both parts of this puzzle are complete!") {
        return Ok(Stars::Two);
    }

    Ok(Stars::Zero)
}

#[cfg(test)]
mod get_stars_tests
{
    use crate::{get_session_cookie, DateInfo};
    use crate::star_info::{get_stars, Stars};

    #[test]
    fn zero_stars_test() {
        let session_cookie = get_session_cookie();
        let result = get_stars(DateInfo::new("24", "2018"), &session_cookie);

        assert!(result.is_ok());
        assert_eq!(Stars::Zero, result.unwrap());
    }

    #[test]
    fn one_star_test() {
        let session_cookie = get_session_cookie();
        let result = get_stars(DateInfo::new("22", "2018"), &session_cookie);

        assert!(result.is_ok());
        assert_eq!(Stars::One, result.unwrap());
    }

    #[test]
    fn two_stars_test() {
        let session_cookie = get_session_cookie();
        let result = get_stars(DateInfo::new("1", "2018"), &session_cookie);

        assert!(result.is_ok());
        assert_eq!(Stars::Two, result.unwrap());
    }

    #[test]
    fn dead_test() {
        let session_cookie = get_session_cookie();
        let result = get_stars(DateInfo::new("30", "2018"), &session_cookie);

        assert!(result.is_err());
    }
}
