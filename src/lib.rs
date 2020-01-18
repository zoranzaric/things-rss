use std::path::Path;

mod feed_config {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub(crate) struct FeedConfig {
        pub(crate) feeds: Vec<Feed>,
    }

    #[derive(Deserialize)]
    pub(crate) struct Feed {
        pub(crate) title: String,
        pub(crate) url: String,
    }
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Toml(toml::de::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Error {
        Error::Toml(e)
    }
}

#[derive(Debug)]
pub struct Site {
    pub title: String,
    pub url: String,
}

#[derive(Debug)]
pub struct Article {
    pub title: String,
    pub url: String,
}

pub fn read_config<P: AsRef<Path>>(path: P) -> Result<Vec<Site>, Error> {
    let s = std::fs::read_to_string(path)?;
    let config: feed_config::FeedConfig = toml::de::from_str(&s)?;

    Ok(config
        .feeds
        .iter()
        .map(|f| Site {
            title: f.title.clone(),
            url: f.url.clone(),
        })
        .collect())
}
