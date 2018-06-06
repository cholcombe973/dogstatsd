extern crate indexmap;

use self::indexmap::IndexMap;
use std::string::ToString;
use std::time::{SystemTime, UNIX_EPOCH};

fn format_tags(tags: &IndexMap<String, Option<String>>) -> String {
    let mut tuples: Vec<String> = Vec::new();
    for (k, v) in tags {
        match v {
            Some(val) => {
                tuples.push(format!("{}:{}", k, val));
            }
            None => {
                tuples.push(k.clone());
            }
        }
    }

    tuples.join(",")
}

pub trait IntoStatsd {
    fn serialize(&self) -> String;
}

#[derive(Debug)]
pub enum Priority {
    Normal,
    Low,
}

impl ToString for Priority {
    fn to_string(&self) -> String {
        match self {
            Priority::Normal => "normal".into(),
            Priority::Low => "low".into(),
        }
    }
}

#[derive(Debug)]
pub enum AlertType {
    Error,
    Info,
    Warning,
    Success,
}

impl ToString for AlertType {
    fn to_string(&self) -> String {
        match self {
            AlertType::Error => "error".into(),
            AlertType::Info => "info".into(),
            AlertType::Warning => "warning".into(),
            AlertType::Success => "success".into(),
        }
    }
}

#[derive(Debug)]
pub enum MetricType {
    Counter,
    Gauge,
    Timer,
    Histogram,
    Set,
}

impl ToString for MetricType {
    fn to_string(&self) -> String {
        match self {
            MetricType::Counter => "c".into(),
            MetricType::Gauge => "g".into(),
            MetricType::Timer => "ms".into(),
            MetricType::Histogram => "h".into(),
            MetricType::Set => "s".into(),
        }
    }
}

#[derive(Debug)]
pub enum Status {
    Ok,
    Warning,
    Critical,
    Unknown,
}

impl ToString for Status {
    fn to_string(&self) -> String {
        match self {
            Status::Ok => "0".into(),
            Status::Critical => "2".into(),
            Status::Warning => "1".into(),
            Status::Unknown => "3".into(),
        }
    }
}

#[derive(Debug)]
pub struct Sample(f64);

impl Sample {
    pub fn new(s: f64) -> Result<Self, String> {
        if s < 0.0 || s > 1.0 {
            Err("Sample values must be between 0.0 and 1.0".into())
        } else {
            Ok(Sample(s))
        }
    }
}

#[test]
fn test_metric() {
    let mut tags: IndexMap<String, Option<String>> = IndexMap::new();
    tags.insert("foo".into(), None);
    tags.insert("bar".into(), Some("baz".into()));
    let m = Metric {
        metric: "gluster".into(),
        name: "heal_count".into(),
        value: 0.0,
        m_type: MetricType::Gauge,
        sample_rate: None,
        tags: Some(tags),
    };
    println!("{}", m.serialize());
    assert_eq!("gluster.heal_count:0|g|#foo,bar:baz", m.serialize());
}

#[derive(Debug)]
pub struct Metric {
    /// A string with no colons, bars, or @ characters
    pub metric: String,
    pub name: String,
    pub value: f64,
    /// Metric type
    pub m_type: MetricType,
    ///sample rate (optional) — a float between 0 and 1, inclusive.
    ///Only works with counter, histogram, and timer metrics.
    ///Default is 1 (i.e. sample 100% of the time).
    pub sample_rate: Option<Sample>,
    pub tags: Option<IndexMap<String, Option<String>>>,
}

impl IntoStatsd for Metric {
    fn serialize(&self) -> String {
        let mut buff = String::new();
        buff.push_str(&format!(
            "{}.{}:{}|{}",
            self.metric,
            self.name,
            self.value,
            self.m_type.to_string()
        ));
        if let Some(ref s) = self.sample_rate {
            buff.push_str(&format!("|@{}", s.0));
        }
        if let Some(ref tags) = self.tags {
            buff.push_str(&format!("|#{}", format_tags(tags)));
        }

        buff
    }
}

#[test]
fn test_event() {
    let mut tags: IndexMap<String, Option<String>> = IndexMap::new();
    tags.insert("foo".into(), None);
    tags.insert("bar".into(), Some("baz".into()));
    let e = Event {
        title: "Foo".into(),
        text: "Foo happened!".into(),
        timestamp: None,
        hostname: Some("host".into()),
        aggregation_key: None,
        priority: Some(Priority::Normal),
        source_type: Some("logs".into()),
        alert_type: Some(AlertType::Error),
        tags: Some(tags),
    };
    println!("{}", e.serialize());
    assert_eq!(
        "_e{3,13}:Foo|Foo happened!|h:host|p:normal|t:error|#foo,bar:baz",
        e.serialize()
    );
}

#[derive(Debug)]
pub struct Event {
    pub title: String,
    pub text: String,
    pub timestamp: Option<SystemTime>,
    pub hostname: Option<String>,
    pub aggregation_key: Option<String>,
    pub priority: Option<Priority>,
    pub source_type: Option<String>,
    pub alert_type: Option<AlertType>,
    pub tags: Option<IndexMap<String, Option<String>>>,
}

impl IntoStatsd for Event {
    fn serialize(&self) -> String {
        let mut buff = String::new();
        buff.push_str(&format!(
            "_e{{{},{}}}:{}|{}",
            self.title.len(),
            self.text.len(),
            self.title,
            self.text
        ));
        if let Some(ts) = self.timestamp {
            buff.push_str(&format!(
                "|d:{}",
                ts.duration_since(UNIX_EPOCH).unwrap().as_secs() * 1000
            ));
        }
        if let Some(ref h) = self.hostname {
            buff.push_str(&format!("|h:{}", h));
        }
        if let Some(ref p) = self.priority {
            buff.push_str(&format!("|p:{}", p.to_string()));
        }
        if let Some(ref a) = self.alert_type {
            buff.push_str(&format!("|t:{}", a.to_string()));
        }
        if let Some(ref tags) = self.tags {
            buff.push_str(&format!("|#{}", format_tags(tags)));
        }

        buff
    }
}

#[test]
fn test_service() {
    let mut tags: IndexMap<String, Option<String>> = IndexMap::new();
    tags.insert("foo".into(), None);
    tags.insert("bar".into(), Some("baz".into()));
    let s = ServiceCheck {
        name: "GlusterD".into(),
        status: Status::Ok,
        timestamp: None,
        hostname: Some("host".into()),
        tags: Some(tags),
        service_message: Some("volume ok".into()),
    };
    println!("{}", s.serialize());
    assert_eq!(
        "_sc|GlusterD|0|h:host|#foo,bar:baz|m:volume ok",
        s.serialize()
    );
}

#[derive(Debug)]
pub struct ServiceCheck {
    pub name: String,
    pub status: Status,
    pub timestamp: Option<SystemTime>,
    pub hostname: Option<String>,
    pub tags: Option<IndexMap<String, Option<String>>>,
    pub service_message: Option<String>,
}

impl IntoStatsd for ServiceCheck {
    fn serialize(&self) -> String {
        let mut buff = String::new();
        buff.push_str(&format!("_sc|{}|{}", self.name, self.status.to_string()));
        if let Some(ts) = self.timestamp {
            buff.push_str(&format!(
                "|d:{}",
                ts.duration_since(UNIX_EPOCH).unwrap().as_secs() * 1000
            ));
        }
        if let Some(ref h) = self.hostname {
            buff.push_str(&format!("|h:{}", h));
        }
        if let Some(ref tags) = self.tags {
            buff.push_str(&format!("|#{}", format_tags(tags)));
        }
        if let Some(ref msg) = self.service_message {
            buff.push_str(&format!("|m:{}", msg));
        }

        buff
    }
}
