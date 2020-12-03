mod star_info;

use log::info;
use reqwest::header::COOKIE;
use std::path::Path;
use std::str::FromStr;
use std::{env, fmt, fs};

pub struct DateInfo {
    day: String,
    year: String,
}

impl DateInfo {
    pub fn new(day: &str, year: &str) -> Self {
        DateInfo { day: day.to_string(), year: year.to_string() }
    }
}

pub type Result<T> = std::result::Result<T, FetchError>;

#[derive(Debug, Clone)]
pub enum FetchError {
    Cause(String),
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FetchError::Cause(cause) => {
                write!(f, "Error fetching Advent of Code input: {}", cause)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AocInput {
    day: String,
    year: String,
    pub input: String,
}

const INPUT_DIR: &str = "inputs";

impl AocInput {
    pub fn new(day: &str, year: &str, input: &str) -> AocInput {
        AocInput {
            day: day.to_string(),
            year: year.to_string(),
            input: input.to_string(),
        }
    }

    pub fn save_to_file(&self) -> Result<()> {
        info!(
            "Saving input for day {}-{} to {}",
            self.day, self.year, INPUT_DIR
        );

        if !Path::new(INPUT_DIR).exists() && fs::create_dir(INPUT_DIR).is_err() {
            return Err(FetchError::Cause(
                "Unable to create input directory".to_string(),
            ));
        }

        let input_filename = self.get_input_filename(INPUT_DIR);

        if fs::write(input_filename, &self.input).is_err() {
            return Err(FetchError::Cause("Unable to write the file".to_string()));
        }

        Ok(())
    }

    pub fn split<T: FromStr>(&self) -> Vec<T> {
        self.input
            .split_ascii_whitespace()
            .map(|x| x.parse().ok().unwrap())
            .collect()
    }

    pub fn split_by<T: FromStr>(&self, delim: &str) -> Vec<T> {
        self.input
            .split(delim)
            .map(|x| x.parse().ok().unwrap())
            .collect()
    }

    fn get_input_filename(&self, input_dir: &str) -> String {
        format!("{}/{}-{}.txt", input_dir, self.year, self.day)
    }
}

impl ToString for AocInput {
    fn to_string(&self) -> String {
        self.input.clone()
    }
}

pub fn fetch_input(day: &str, year: &str, session: &str) -> Result<AocInput> {
    info!("Fetching input for day {}-{}", day, year);

    let url = format!("https://adventofcode.com/{}/day/{}/input", year, day);
    let input = reqwest::blocking::Client::new()
        .get(&url)
        .header(COOKIE, format!("session={}", session))
        .send();

    if input.is_err() {
        return Err(FetchError::Cause("Error sending GET request".to_string()));
    }

    let input = input.unwrap().text();

    if input.is_err() {
        return Err(FetchError::Cause(
            "Error converting input to string".to_string(),
        ));
    }

    let input = input.unwrap();

    if input.contains("Service Unavailable") {
        return Err(FetchError::Cause("Advent of Code is dead".to_string()));
    }

    if input.contains("Please don't repeatedly request") || input.contains("Not Found") {
        return Err(FetchError::Cause(format!(
            "Puzzle for day {} is not live yet",
            day
        )));
    }

    if input.contains("log in") {
        return Err(FetchError::Cause("Session cookie is invalid".to_string()));
    }

    if input.contains("Internal Server Error") {
        return Err(FetchError::Cause(
            "Internal Server Error, invalid session cookie perhaps?".to_string(),
        ));
    }

    Ok(AocInput::new(day, year, &input))
}

// todo
// pub enum FetchMode
// {
//     Caching,
//     Overriding,
// }

// pub fn fetch_and_save_input(day: &str, year: &str, mode: FetchMode) -> Result<String> {
pub fn load_or_fetch_input(day: &str, year: &str) -> Result<AocInput> {
    let input = AocInput::new(day, year, "");
    let input_path = input.get_input_filename(INPUT_DIR);

    if Path::new(&input_path).exists() {
        info!("Loading input for day {}-{} from {}", day, year, input_path);

        let raw_input = fs::read_to_string(input_path);

        if raw_input.is_err() {
            return Err(FetchError::Cause("Unable to read the file".to_string()));
        }

        Ok(AocInput::new(day, year, &raw_input.unwrap()))
    } else {
        let session_cookie = get_session_cookie();
        let input = fetch_input(day, year, &session_cookie)?;
        input.save_to_file()?;

        Ok(input)
    }
}

fn get_session_cookie() -> String {
    env::var("AOC_SESSION").expect("Expected a token in the environment")
}

#[cfg(test)]
mod fetch_tests {
    use crate::{fetch_input, get_session_cookie};
    use std::fs;

    #[test]
    fn invalid_cookie_test() {
        let input = fetch_input("1", "2018", "invalid");
        assert!(input.is_err());
    }

    #[test]
    fn valid_cookie_test() {
        let session_cookie = get_session_cookie();
        let input = fetch_input("1", "2018", &session_cookie);

        let expected = fs::read_to_string("test/2018-1-input.txt").expect("Error reading the file");

        assert!(input.is_ok());
        assert_eq!(expected, input.unwrap().to_string());
    }
}

#[cfg(test)]
mod save_tests {
    use crate::{fetch_input, get_session_cookie, load_or_fetch_input};
    use std::fs;
    use std::path::Path;

    #[test]
    fn save_input_test() {
        let session_cookie = get_session_cookie();
        let input = fetch_input("4", "2016", &session_cookie).unwrap();

        assert!(input.save_to_file().is_ok());
        assert!(Path::new("inputs").exists());
        assert!(Path::new("inputs/2016-4.txt").exists());
    }

    #[test]
    fn fetch_and_save_test() {
        let input = load_or_fetch_input("2", "2017");
        let expected = fs::read_to_string("test/2017-2-input.txt").expect("Error reading the file");

        assert!(input.is_ok());
        assert!(Path::new("inputs/2017-2.txt").exists());
        assert_eq!(expected, input.unwrap().input);
    }

    #[test]
    fn load_from_disk_test() {
        let input = load_or_fetch_input("5", "2017");
        let filename = "inputs/2017-5.txt";

        assert!(input.is_ok());
        assert!(Path::new(filename).exists());

        fs::write(filename, "data").expect("Unable to write file");

        let input = load_or_fetch_input("5", "2017");
        assert!(input.is_ok());
        assert!(Path::new(filename).exists());

        assert_eq!("data", input.unwrap().input);
    }
}

#[cfg(test)]
mod split_tests {
    use crate::AocInput;

    #[test]
    fn split_as_int_vec() {
        let input = AocInput::new("1", "2020", "1 2 3 4 5");

        assert_eq!(vec![1, 2, 3, 4, 5], input.split());
    }

    #[test]
    fn split_as_string_vec() {
        let input = AocInput::new("1", "2020", "1 2 3 4 5");

        assert_eq!(vec!["1", "2", "3", "4", "5"], input.split::<String>());
    }

    #[test]
    fn split_with_delimiter() {
        let input = AocInput::new("1", "2020", "1,2,3,4,5");

        assert_eq!(vec![1, 2, 3, 4, 5], input.split_by(","));
    }
}
