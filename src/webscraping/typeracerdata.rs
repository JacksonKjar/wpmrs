use std::{error::Error, str::FromStr};

use lazy_static::lazy_static;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TextRow {
    pub rank: usize,
    pub id: String,
    pub text: String,
    pub length: usize,
    pub races: usize,
    pub difficulty: f64,
    pub top_score: f64,
    pub top_100: f64,
    pub average: f64,
    pub date_active: String,
}

lazy_static! {
    static ref RE_ROW: Regex = Regex::new(&[
        r"<tr>",
        r"<td.*?>(\d+)\.</td>",                 // rank
        r"<td.*?>(#\d+)</td>",                  // id
        r"<td.*?><a.*?>(.*?)</a></td>",         // text
        r"<td.*?>([,\d]+)</td>",                // length
        r"<td.*?>([,\d]+)</td>",                // races
        r"<td.*?>([,\.\d]+)</td>",              // difficulty
        r"<td.*?><a.*?>([,\.\d]+)</a>.*?</td>", // top_score
        r"<td.*?>([,\.\d]+)</td>",              // top_100
        r"<td.*?>([,\.\d]+)</td>",              // average
        r"<td.*?>(.*?)</td>",                   // date_active
        r"</tr>"
    ].join(r"\s*")).unwrap();
}

impl TextRow {
    pub fn parse_row(raw: &str) -> Self {
        let res = RE_ROW.captures(raw).unwrap();
        Self::from_match(res).unwrap()
    }
    pub fn parse_table(raw: &str) -> impl Iterator<Item = Self> + '_ {
        RE_ROW
            .captures_iter(raw)
            .map(|c| Self::from_match(c).unwrap())
    }
    fn from_match(cap: Captures) -> Result<Self, Box<dyn Error>> {
        let caps: Option<Vec<_>> = cap.iter().skip(1).map(|c| c.map(|d| d.as_str())).collect();
        let caps = caps.ok_or("Match not found for all groups :(")?;
        if caps.len() != 10 {
            return Err(format!("Expected 10 columns. Found {}", caps.len()).into());
        }

        Ok(Self {
            rank: caps[0].parse_html_str()?,
            id: caps[1].parse_html_str()?,
            text: caps[2].parse_html_str()?,
            length: caps[3].parse_html_str()?,
            races: caps[4].parse_html_str()?,
            difficulty: caps[5].parse_html_str()?,
            top_score: caps[6].parse_html_str()?,
            top_100: caps[7].parse_html_str()?,
            average: caps[8].parse_html_str()?,
            date_active: caps[9].parse_html_str()?,
        })
    }
}

trait FromHtmlStr: FromStr {
    fn from_html_str(html: &str) -> Result<Self, Self::Err>;
}

impl FromHtmlStr for usize {
    fn from_html_str(html: &str) -> Result<Self, Self::Err> {
        Self::from_str(&html.replace(',', ""))
    }
}

impl FromHtmlStr for f64 {
    fn from_html_str(html: &str) -> Result<Self, Self::Err> {
        Self::from_str(&html.replace(',', ""))
    }
}

impl FromHtmlStr for String {
    fn from_html_str(html: &str) -> Result<Self, Self::Err> {
        Self::from_str(&html.replace("&quot;", "\""))
    }
}

trait ParseHtmlStr {
    fn parse_html_str<T: FromHtmlStr>(&self) -> Result<T, T::Err>;
}

impl<K: Deref<Target = str>> ParseHtmlStr for K {
    fn parse_html_str<T: FromHtmlStr>(&self) -> Result<T, T::Err> {
        T::from_html_str(self)
    }
}
