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

// Clipping type
#[derive(Debug, PartialEq)]
pub enum ClippingType {
    Highlight,
    Note,
    Bookmark,
}

impl fmt::Display for ClippingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for ClippingType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // en
            "Highlight" => Ok(ClippingType::Highlight),
            "Note" => Ok(ClippingType::Note),
            "Bookmark" => Ok(ClippingType::Bookmark),
            // support more languages...
            _ => Err(format!("Invalid clipping type: {}", s)),
        }
    }
}

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
    pub clipping_type: ClippingType,
    pub book_title: String,
    pub author: String,
    pub page: Option<u32>,
    pub location: String,
    pub datetime: String,
    pub weekday: Weekday,
    pub content: Option<String>,
}

impl fmt::Display for Clipping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Book: {}\nAuthor: {}\nLocation: {}\nDate: {} ({})\nPage: {}\nContent: {}",
            self.book_title,
            self.author,
            self.location,
            self.datetime,
            self.weekday,
            self.page.map_or("N/A".to_string(), |p| p.to_string()),
            self.content.as_deref().unwrap_or("N/A")
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

        let clipping_type = Self::parse_type(second_line)?;
        let (page, location) = Self::parse_page_and_location(second_line)?;
        let weekday = Self::parse_weekday(second_line)?;
        let datetime = Self::parse_datetime(second_line)?;

        // Parse content
        let content = if clipping_type == ClippingType::Bookmark {
            None
        } else {
            Some(
                lines
                    .next()
                    .ok_or_else(|| ParseError::MissingField("content".to_string()))?
                    .to_string(),
            )
        };

        Ok(Self {
            clipping_type,
            book_title,
            author,
            page,
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

    fn parse_type(line: &str) -> Result<ClippingType, ParseError> {
        let patterns = vec![
            // en
            r"(Bookmark|Highlight|Note)",
            // support more languages...
        ];

        patterns
            .iter()
            .find_map(|pattern| {
                let re = Regex::new(pattern).unwrap();
                if let Some(caps) = re.captures(line) {
                    if caps.len() == 2 {
                        let clipping_type: ClippingType = caps[1]
                            .parse()
                            .map_err(|error| {
                                ParseError::InvalidFormat(format!(
                                    "Invalid clipping type: {}",
                                    error
                                ))
                            })
                            .ok()?;

                        Some(Ok(clipping_type))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                Err(ParseError::InvalidFormat(format!(
                    "Failed to parse clipping type: {}",
                    line
                )))
            })
    }

    fn parse_page_and_location(line: &str) -> Result<(Option<u32>, String), ParseError> {
        let patterns = vec![
            // en
            r"page (\d+) \| Location (\d+-\d+)",
            r"page (\d+) \| Location (\d+)",
            r"Location (\d+-\d+)",
            r"Location (\d+)",
            // support more languages...
        ];

        patterns
            .iter()
            .find_map(|pattern| {
                let re = Regex::new(pattern).unwrap();
                if let Some(caps) = re.captures(line) {
                    match caps.len() {
                        3 => {
                            // have page
                            let page: u32 = caps[1]
                                .parse()
                                .map_err(|error| {
                                    ParseError::InvalidFormat(format!("Invalid page: {}", error))
                                })
                                .ok()?;
                            let location = caps[2].to_string();
                            Some(Ok((Some(page), location)))
                        }
                        2 => {
                            let location = caps[1].to_string();
                            Some(Ok((None, location)))
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                Err(ParseError::InvalidFormat(format!(
                    "Failed to parse page and location: {}",
                    line
                )))
            })
    }

    fn parse_weekday(line: &str) -> Result<Weekday, ParseError> {
        let patterns = vec![
            // en
            r"Added on (Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sunday)", // support more languages...
        ];

        patterns
            .iter()
            .find_map(|pattern| {
                let re = Regex::new(pattern).unwrap();
                if let Some(caps) = re.captures(line) {
                    if caps.len() == 2 {
                        let weekday: Weekday = caps[1]
                            .parse()
                            .map_err(|error| {
                                ParseError::InvalidFormat(format!("Invalid weekday: {}", error))
                            })
                            .ok()?;
                        Some(Ok(weekday))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                Err(ParseError::InvalidFormat(format!(
                    "Failed to parse weekday: {}",
                    line
                )))
            })
    }

    fn parse_datetime(line: &str) -> Result<String, ParseError> {
        let patterns = vec![
            r"(\d{1,2}\s+(?:January|February|March|April|May|June|July|August|September|October|November|December)\s+\d{4}\s+\d{1,2}:\d{2}:\d{2})",
        ];

        patterns
            .iter()
            .find_map(|pattern| {
                let re = Regex::new(pattern).unwrap();
                if let Some(caps) = re.captures(line) {
                    if caps.len() == 2 {
                        let datetime = caps[1].to_string();
                        Some(Ok(datetime))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                Err(ParseError::InvalidFormat(format!(
                    "Failed to parse datetime: {}",
                    line
                )))
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
    fn test_clipping_parsing_en() {
        // Highlight
        let highlight = "\
Book Title (Author Name)
- Your Highlight on page 123 | Location 1234-1235 | Added on Monday, 26 August 2025 12:57:30

Highlighted text content goes here.";

        let result = Clipping::from_text(highlight).unwrap();

        assert_eq!(result.clipping_type, ClippingType::Highlight);
        assert_eq!(result.book_title, "Book Title");
        assert_eq!(result.author, "Author Name");
        assert_eq!(result.page, Some(123));
        assert_eq!(result.location, "1234-1235");
        assert_eq!(result.datetime, "26 August 2025 12:57:30");
        assert_eq!(result.weekday, Weekday::Monday);
        assert_eq!(
            result.content,
            Some(format!("Highlighted text content goes here."))
        );

        // Bookmark
        let bookmark = "\
Book Title (Author Name)
- Your Bookmark on page 123 | Location 1234 | Added on Monday, 26 August 2025 12:57:30

";
        let result = Clipping::from_text(bookmark).unwrap();

        assert_eq!(result.clipping_type, ClippingType::Bookmark);
        assert_eq!(result.content, None);

        // Note
        let note = "\
Book Title (Author Name)
- Your Note on page 123 | Location 1234 | Added on Monday, 26 August 2025 12:57:30

Your note content goes here.";
        let result = Clipping::from_text(note).unwrap();

        assert_eq!(result.clipping_type, ClippingType::Note);
        assert_eq!(
            result.content,
            Some(format!("Your note content goes here."))
        );
    }

    #[test]
    fn test_missing_content() {
        let clipping = "\
Book (Author)
Location 123 | Added on Monday, 1 January 2025 10:00:00";

        assert!(Clipping::from_text(clipping).is_err());
    }
}
