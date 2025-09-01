use regex::Regex;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

/// Parse errors
#[derive(Debug)]
pub enum ParseError {
    InvalidFormat(String),
    MissingField(String),
    InvalidWeekday(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ParseError::MissingField(field) => write!(f, "Missing field: {}", field),
            ParseError::InvalidWeekday(day) => write!(f, "Invalid weekday: {}", day),
        }
    }
}

impl Error for ParseError {}

/// Days of the week
#[derive(Debug, PartialEq)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl fmt::Display for Weekday {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for Weekday {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Monday" => Ok(Weekday::Monday),
            "Tuesday" => Ok(Weekday::Tuesday),
            "Wednesday" => Ok(Weekday::Wednesday),
            "Thursday" => Ok(Weekday::Thursday),
            "Friday" => Ok(Weekday::Friday),
            "Saturday" => Ok(Weekday::Saturday),
            "Sunday" => Ok(Weekday::Sunday),
            _ => Err(format!("Invalid weekday: {}", s)),
        }
    }
}

/// A single Kindle clipping
#[derive(Debug)]
pub struct Clipping {
    pub book_title: String,
    pub author: String,
    pub location: String,
    pub datetime: String,
    pub weekday: Weekday,
    pub content: String,
}

impl fmt::Display for Clipping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Book: {}\nAuthor: {}\nLocation: {}\nDate: {} ({})\nContent: {}",
            self.book_title, self.author, self.location, self.datetime, self.weekday, self.content
        )
    }
}

impl Clipping {
    /// Parse a single clipping from text
    pub fn from_text(text: &str) -> Result<Self, ParseError> {
        let mut lines = text.lines().filter(|line| !line.trim().is_empty());

        // Parse first line: book title and author
        let first_line = lines
            .next()
            .ok_or_else(|| ParseError::MissingField("book title and author".to_string()))?;

        let (book_title, author) = Self::parse_title_and_author(first_line)?;

        // Parse second line: metadata
        let second_line = lines
            .next()
            .ok_or_else(|| ParseError::MissingField("metadata".to_string()))?;

        let (location, weekday, datetime) = Self::parse_metadata(second_line)?;

        // Parse content
        let content = lines
            .next()
            .ok_or_else(|| ParseError::MissingField("content".to_string()))?
            .to_string();

        Ok(Self {
            book_title,
            author,
            location,
            datetime,
            weekday,
            content,
        })
    }

    fn parse_title_and_author(line: &str) -> Result<(String, String), ParseError> {
        // Match pattern: "Title (Author)"
        let re = Regex::new(r"^(.+?)\s+\((.+)\)$").unwrap();

        re.captures(line)
            .map(|caps| (caps[1].trim().to_string(), caps[2].trim().to_string()))
            .ok_or_else(|| {
                ParseError::InvalidFormat(format!(
                    "Expected 'Title (Author)' format, got: {}",
                    line
                ))
            })
    }

    fn parse_metadata(line: &str) -> Result<(String, Weekday, String), ParseError> {
        // Match location, weekday and datetime
        let re = Regex::new(r"Location\s+(\d+-\d+)\s+\|\s+Added on\s+(Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sunday),\s+(\d{1,2}\s+(?:January|February|March|April|May|June|July|August|September|October|November|December)\s+\d{4}\s+\d{1,2}:\d{2}:\d{2})").unwrap();

        re.captures(line)
            .ok_or_else(|| {
                ParseError::InvalidFormat(format!("Cannot parse metadata line: {}", line))
            })
            .and_then(|caps| {
                let location = caps[1].to_string();
                let weekday = caps[2].parse().map_err(|error| {
                    ParseError::InvalidFormat(format!("Invalid weekday: {}", error))
                })?;
                let datetime = caps[3].to_string();
                Ok((location, weekday, datetime))
            })
    }
}

pub fn parse_clippings(contents: &str) -> Result<Vec<Clipping>, ParseError> {
    const SEPARATOR: &str = "==========";

    contents
        .split(SEPARATOR)
        .filter(|text| !text.trim().is_empty())
        .enumerate()
        .map(|(index, text)| {
            Clipping::from_text(text).map_err(|error| {
                ParseError::InvalidFormat(format!(
                    "Failed to parse clipping #{}: {}",
                    index + 1,
                    error
                ))
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weekday_parsing() {
        assert_eq!("Monday".parse::<Weekday>().unwrap(), Weekday::Monday);
        assert_eq!("Sunday".parse::<Weekday>().unwrap(), Weekday::Sunday);
        assert!("InvalidDay".parse::<Weekday>().is_err());
    }

    #[test]
    fn test_clipping_parsing() {
        let clipping = "\
一无所有 ([美] Ursula K. Le Guin 著 (陶雪蕾 译))
- Your Highlight on page 314 | Location 5134-5134 | Added on Tuesday, 26 August 2025 12:57:30

时间没有虚度，痛苦自有其价值。";

        let result = Clipping::from_text(clipping).unwrap();

        assert_eq!(result.book_title, "一无所有");
        assert_eq!(result.author, "[美] Ursula K. Le Guin 著 (陶雪蕾 译)");
        assert_eq!(result.location, "5134-5134");
        assert_eq!(result.datetime, "26 August 2025 12:57:30");
        assert_eq!(result.weekday, Weekday::Tuesday);
        assert_eq!(result.content, "时间没有虚度，痛苦自有其价值。");
    }

    #[test]
    fn test_missing_content() {
        let clipping = "\
Book (Author)
Location 123 | Added on Monday, 1 January 2025 10:00:00";

        assert!(Clipping::from_text(clipping).is_err());
    }
}
