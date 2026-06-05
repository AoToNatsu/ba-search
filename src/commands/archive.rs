use anyhow::{Context, Result, anyhow};
use bon::builder;
use cmd_lib::run_cmd;
use std::fs;
use std::path::{Path, PathBuf};

const ERROR_HEADER: &str = "commands::archive";

#[derive(Debug)]
enum LinkType {
    Volume { volume: Box<str>, chapter: Box<str> },
    Event(Box<str>),
    Relationship(Box<str>),
}

impl LinkType {
    /// Returns the full details for a Volume (Volume, Chapter).
    pub fn as_volume(&self) -> Option<(&str, &str)> {
        if let Self::Volume { volume, chapter } = self {
            Some((volume, chapter))
        } else {
            None
        }
    }
    /// Defaults to "Volume_#" for Volume
    pub fn as_text(&self) -> &str {
        match self {
            Self::Volume { volume, .. } => volume,
            Self::Event(text) | Self::Relationship(text) => text,
        }
    }
}

#[builder]
/// Archive character stories
pub fn main(
    link: String,
    start: Option<usize>,
    end: usize,
    increment: Option<usize>,
    format: Option<String>,
    quiet: bool,
) -> Result<()> {
    let link = link.trim();
    let start = start.unwrap_or(1);
    let increment = increment.unwrap_or(1);
    let format = format.unwrap_or("{}".into());
    let format = format.trim();

    // Verify inputs
    if link.is_empty() {
        return Err(anyhow::anyhow!(ERROR_HEADER).context("Link cannot be empty"));
    }
    if !link.contains(format) {
        let err_message =
            format!("Link does not contain the formatter string\nFormatter string: {format:?}");
        return Err(anyhow::anyhow!(ERROR_HEADER).context(err_message));
    }
    if increment == 0 {
        return Err(anyhow::anyhow!(ERROR_HEADER).context("Increment cannot be 0"));
    }
    if format.is_empty() {
        return Err(anyhow::anyhow!(ERROR_HEADER).context("Format cannot be empty"));
    }

    // Grab link's name; assumes sanitized
    let link_type = match link {
        s if s.contains("Relationship_Story") => {
            let right_side = link.split_once("/wiki/").unwrap().1;
            let character_name = right_side.split_once("/").unwrap().0;
            LinkType::Relationship(character_name.into())
        }
        s if s.contains("Volume_") & s.contains("Chapter_") => {
            let right_side = link.split_once("Main_Story/").unwrap().1;
            let volume_chapter = right_side.split_once("/Episode").unwrap().0;
            let volume_chapter = volume_chapter.split_once('/').unwrap();

            let volume = volume_chapter.0;
            let chapter = volume_chapter.1;

            LinkType::Volume {
                volume: volume.into(),
                chapter: chapter.into(),
            }
        }
        s if s.contains("Story") => {
            let right_side = link.split_once("/wiki/").unwrap().1;
            let event_name = right_side.split_once("/").unwrap().0;
            LinkType::Event(event_name.into())
        }
        _ => return Err(anyhow!(ERROR_HEADER).context("Could not find a valid link type")),
    };

    // Create directories and `cd` into them
    let create_dir = |path: &Path| {
        if path.exists() && path.is_file() {
            Err(anyhow!(ERROR_HEADER).context(format!("Path exists, but as a file: {path:?}")))
        } else if !path.exists() {
            match fs::create_dir(&path) {
                Ok(()) => Ok(()),
                Err(err) => return Err(anyhow!(ERROR_HEADER).context(err)),
            }
        } else {
            Ok(())
        }
    };

    let start_archiving = |path: &Path| {
        for i in (start..=end).step_by(increment) {
            let formatted_link = link.replace(&format, i.to_string().as_str());
            let err_message =
                format!("Failed to run wget for the following link: {formatted_link:?}");

            match run_cmd!(cd $path; wget $formatted_link).context(err_message) {
                Ok(_) => {}
                Err(_) => break,
            }
        }
    };

    if let Some((volume, chapter)) = link_type.as_volume() {
        let volume_path: PathBuf = volume.into();
        let chapter_path = {
            let mut new = volume_path.clone();
            new.push(chapter);
            new
        };

        create_dir(&volume_path)?;
        create_dir(&chapter_path)?;

        start_archiving(&chapter_path);
    } else {
        let text: &str = link_type.as_text();
        let text_path: PathBuf = text.into();

        create_dir(&text_path)?;
        start_archiving(&text_path);
    }

    // Use format if given
    if !quiet {
        println!("Details:\nLink: {link:?}\nRange: {start}-{end}\nIncrement: {increment}");
    }

    Ok(())
}
