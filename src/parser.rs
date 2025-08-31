use regex::Regex;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
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

#[derive(Debug)]
struct Clipping {
    book_title: String,
    author: String,
    location: String,
    datetime: String,
    weekday: Weekday,
    content: String,
}

impl Clipping {
    pub fn build(clipping: &str) -> Result<Self, String> {
        let mut clipping_iterator = clipping.lines().filter(|line| !line.trim().is_empty());

        let re = Regex::new(r"^(.+?)\s+\((.+)\)$").unwrap();
        let first_line = clipping_iterator.next().unwrap();

        let mut book_title = String::new();
        let mut author = String::new();

        if let Some(captures) = re.captures(first_line) {
            book_title = captures[1].to_string();
            author = captures[2].to_string();
        } else {
            eprintln!("Failed to parse clipping");
        }

        let second_line = clipping_iterator.next().unwrap();
        let re = Regex::new(r"Location\s+(\d+-\d+)\s+\|\s+Added on\s+(Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sunday),\s+(\d{1,2}\s+(?:January|February|March|April|May|June|July|August|September|October|November|December)\s+\d{4}\s+\d{1,2}:\d{2}:\d{2})").unwrap();
        let mut location = String::new();
        let mut datetime = String::new();
        let mut weekday_str = String::new();

        if let Some(captures) = re.captures(second_line) {
            location = captures[1].to_string();
            weekday_str = captures[2].to_string();
            datetime = captures[3].to_string();
        } else {
            eprintln!("Failed to parse clipping");
        }

        let weekday = weekday_str
            .parse::<Weekday>()
            .map_err(|e| format!("Failed to parse weekday: {}", e))?;

        let content = clipping_iterator.next().unwrap().to_string();

        // 需要返回 Clipping 实例
        Ok(Self {
            book_title,
            author,
            location,
            datetime,
            weekday,
            content,
        })
    }
}

pub fn parse_clippings(contents: &str) {
    let clippings_str: Vec<&str> = contents.split("==========").collect();

    let clippings: Vec<Clipping> = clippings_str
        .iter()
        .filter(|clipping| !clipping.trim().is_empty())
        .map(|clipping| Clipping::build(clipping).unwrap())
        .collect();

    println!("{:?}", clippings);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipping_information_parsing() {
        let clipping = "\
一无所有 ([美] Ursula K. Le Guin 著 (陶雪蕾 译))
- Your Highlight on page 314 | Location 5134-5134 | Added on Tuesday, 26 August 2025 12:57:30

时间没有虚度，痛苦自有其价值。";

        let result = Clipping::build(clipping).unwrap();

        assert_eq!(result.book_title, "一无所有");
        assert_eq!(result.author, "[美] Ursula K. Le Guin 著 (陶雪蕾 译)");
        assert_eq!(result.location, "5134-5134");
        assert_eq!(result.datetime, "26 August 2025 12:57:30");
        assert_eq!(result.weekday, Weekday::Tuesday);
        assert_eq!(result.content, "时间没有虚度，痛苦自有其价值。");
    }
}
