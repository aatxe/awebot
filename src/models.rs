use std::fmt::{Display, Error, Formatter};

use chrono::{DateTime, NaiveDateTime, Utc};

use schema::{mail, whois};

#[derive(Queryable)]
pub struct Message {
    _key: i32,
    pub target: String,
    pub sender: String,
    pub message: String,
    pub sent: NaiveDateTime,
    pub private: bool,
}

impl Message {
    fn time_ago_str(sent: NaiveDateTime) -> String {
        let sent_utc = DateTime::<Utc>::from_utc(sent, Utc);
        let dur = Utc::now().signed_duration_since(sent_utc);
        if dur.num_weeks() > 1 {
            format!("{} weeks ago", dur.num_weeks())
        } else if dur.num_weeks() == 1 {
            "A week ago".to_owned()
        } else if dur.num_days() > 1 {
            format!("{} days ago", dur.num_days())
        } else if dur.num_days() == 1 {
            "A day ago".to_owned()
        } else if dur.num_hours() > 1 {
            format!("{} hours ago", dur.num_hours())
        } else if dur.num_hours() == 1 {
            "An hour ago".to_owned()
        } else if dur.num_minutes() > 1 {
            format!("{} minutes ago", dur.num_minutes())
        } else if dur.num_minutes() == 1 {
            "A minute ago".to_owned()
        } else {
            "Moments ago".to_owned()
        }
    }

}

impl Display for Message {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let ago = Message::time_ago_str(self.sent);
        write!(
            fmt, "{}: {}, {} said {}{}", self.target, ago, self.sender,
            self.message,
            if self.message.ends_with('.') || self.message.ends_with('!') ||
                self.message.ends_with('?') { "" } else { "." }
        )
    }
}

#[derive(Insertable)]
#[table_name="mail"]
pub struct NewMessage<'a> {
    pub target: &'a str,
    pub sender: &'a str,
    pub message: &'a str,
    pub sent: &'a NaiveDateTime,
    pub private: bool,
}

#[derive(Queryable)]
pub struct WhoisEntry {
    pub nickname: String,
    pub description: String,
}

impl Display for WhoisEntry {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{} is {}", self.nickname, self.description)
    }
}

#[derive(Insertable)]
#[table_name="whois"]
pub struct NewWhoisEntry<'a> {
    pub nickname: &'a str,
    pub description: &'a str,
}
