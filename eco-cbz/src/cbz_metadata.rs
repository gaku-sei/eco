#![cfg(feature = "metadata")]

use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Primary {
    #[serde(rename = "YES")]
    Yes,
    #[serde(rename = "NO")]
    No,
}

impl From<bool> for Primary {
    fn from(primary: bool) -> Self {
        if primary {
            Self::Yes
        } else {
            Self::No
        }
    }
}

impl From<Primary> for bool {
    fn from(primary: Primary) -> Self {
        match primary {
            Primary::Yes => true,
            Primary::No => false,
        }
    }
}

impl FromStr for Primary {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "y" | "yes" | "Yes" | "YES" => Ok(Self::Yes),
            "n" | "no" | "No" | "NO" => Ok(Self::No),
            _ => Err(Error::MetadataValue(format!("invalid primary value: {s}"))),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credit {
    pub person: Option<String>,
    pub role: Option<String>,
    pub primary: Option<Primary>,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
pub enum Month {
    Jan = 1,
    Feb = 2,
    Mar = 3,
    Apr = 4,
    May = 5,
    Jun = 6,
    Jul = 7,
    Aug = 8,
    Sep = 9,
    Oct = 10,
    Nov = 11,
    Dec = 12,
}

impl Month {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Jan => "January",
            Self::Feb => "February",
            Self::Mar => "March",
            Self::Apr => "April",
            Self::May => "May",
            Self::Jun => "June",
            Self::Jul => "July",
            Self::Aug => "August",
            Self::Sep => "September",
            Self::Oct => "October",
            Self::Nov => "November",
            Self::Dec => "December",
        }
    }
}

impl FromStr for Month {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "jan" | "Jan" | "January" => Ok(Self::Jan),
            "feb" | "Feb" | "February" => Ok(Self::Feb),
            "mar" | "Mar" | "March" => Ok(Self::Mar),
            "apr" | "Apr" | "April" => Ok(Self::Apr),
            "may" | "May" => Ok(Self::May),
            "jun" | "Jun" | "June" => Ok(Self::Jun),
            "jul" | "Jul" | "July" => Ok(Self::Jul),
            "aug" | "Aug" | "August" => Ok(Self::Aug),
            "sep" | "Sep" | "September" => Ok(Self::Sep),
            "oct" | "Oct" | "October" => Ok(Self::Oct),
            "nov" | "Nov" | "November" => Ok(Self::Nov),
            "dev" | "Dev" | "December" => Ok(Self::Dec),
            _ => Err(Error::MetadataValue(format!("invalid month: {s}"))),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComicBookInfoV1 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(rename = "publicationMonth", skip_serializing_if = "Option::is_none")]
    pub publication_month: Option<Month>,
    #[serde(rename = "publicationYear", skip_serializing_if = "Option::is_none")]
    pub publication_year: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<u16>,
    #[serde(rename = "numberOfIssues", skip_serializing_if = "Option::is_none")]
    pub number_of_issues: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<u16>,
    #[serde(rename = "numberOfVolumes", skip_serializing_if = "Option::is_none")]
    pub number_of_volumes: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credits: Option<Vec<Credit>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

impl ComicBookInfoV1 {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_series(mut self, series: impl Into<String>) -> Self {
        self.series = Some(series.into());
        self
    }

    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    #[must_use]
    pub fn with_publisher(mut self, publisher: impl Into<String>) -> Self {
        self.publisher = Some(publisher.into());
        self
    }

    #[must_use]
    pub fn with_publication_month(mut self, publication_month: Month) -> Self {
        self.publication_month = Some(publication_month);
        self
    }

    #[must_use]
    pub fn with_publication_year(mut self, publication_year: u16) -> Self {
        self.publication_year = Some(publication_year);
        self
    }

    #[must_use]
    pub fn with_issue(mut self, issue: u16) -> Self {
        self.issue = Some(issue);
        self
    }

    #[must_use]
    pub fn with_number_of_issues(mut self, number_of_issues: u16) -> Self {
        self.number_of_issues = Some(number_of_issues);
        self
    }

    #[must_use]
    pub fn with_volume(mut self, volume: u16) -> Self {
        self.volume = Some(volume);
        self
    }

    #[must_use]
    pub fn with_number_of_volumes(mut self, number_of_volumes: u16) -> Self {
        self.number_of_volumes = Some(number_of_volumes);
        self
    }

    #[must_use]
    pub fn with_rating(mut self, rating: u8) -> Self {
        self.rating = Some(rating);
        self
    }

    #[must_use]
    pub fn with_genre(mut self, genre: impl Into<String>) -> Self {
        self.genre = Some(genre.into());
        self
    }

    #[must_use]
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    #[must_use]
    pub fn with_country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }

    #[must_use]
    pub fn with_comments(mut self, comments: impl Into<String>) -> Self {
        self.comments = Some(comments.into());
        self
    }

    #[must_use]
    pub fn with_credits(mut self, credits: impl Into<Vec<Credit>>) -> Self {
        self.credits = Some(credits.into());
        self
    }

    #[must_use]
    pub fn with_tags(mut self, tags: impl Into<Vec<String>>) -> Self {
        self.tags = Some(tags.into());
        self
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnofficialMetadata {
    #[serde(rename = "appID", skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
    #[serde(rename = "lastModified", skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<DateTime<Utc>>,
    #[serde(rename = "ComicBookInfo/1.0", skip_serializing_if = "Option::is_none")]
    pub info: Option<ComicBookInfoV1>,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extra: Option<HashMap<String, Value>>,
}

impl UnofficialMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_app_id(mut self, app_id: impl Into<String>) -> Self {
        self.app_id = Some(app_id.into());
        self
    }

    #[must_use]
    pub fn with_last_modified(mut self, last_modified: DateTime<Utc>) -> Self {
        self.last_modified = Some(last_modified);
        self
    }

    #[must_use]
    pub fn with_info(mut self, info: ComicBookInfoV1) -> Self {
        self.info = Some(info);
        self
    }

    #[must_use]
    pub fn with_extra(mut self, extra: HashMap<String, Value>) -> Self {
        self.extra = Some(extra);
        self
    }

    /// ## Errors
    ///
    /// Can fail on json conversion.
    pub fn try_insert_extra(
        mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> Result<Self> {
        if self.extra.is_none() {
            self.extra = Some(HashMap::new());
        }
        self.extra
            .as_mut()
            .unwrap()
            .insert(key.into(), serde_json::to_value(value)?);

        Ok(self)
    }
}
