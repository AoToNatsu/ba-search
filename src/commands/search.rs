use anyhow::{Context, Result, anyhow};
use bon::builder;
use cmd_lib::{run_cmd, spawn_with_output};
use colored::Colorize;
use rstest::rstest;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

const ERROR_HEADER: &str = "commands::search";

const IGNORED: [&str; 109] = [
    "effect:", // This could *possibly* ignore ordinary sentences :shrug:
    "</span>",
    "volume-",
    "volume icon-",
    "right: ",
    "left: ",
    r#"class="new""#,
    "float: ", // This marks the last thing that *might* be wrongly omitted
    "text-align:",
    "border: 0;",
    "border-radius",
    "border-width",
    "border-spacing",
    "border-top",
    "border-style",
    "border-bottom",
    "box-sizing:",
    "Lua memory usage:",
    r#"class="story-container"#,
    r#"class="length"#,
    r#"class="current"#,
    r#"class="vector"#,
    r#"class="time"#,
    r#"class="divider"#,
    r#"class="timeline"#,
    r#"class="toggle"#,
    "client-nojs",
    "Parsed by mwtask",
    "table class=",
    "toggle-play pause",
    "background-color:",
    "color: var",
    "transition: background-color",
    "siteSub",
    "mw-",
    "g.async",
    "vector-search",
    "stopMobileRedirectToggle",
    "volume-container",
    "volume-button",
    "searchButton",
    "play-container",
    "BGM stops",
    r#"class="progress""#,
    r#"class="story-bgm-container""#,
    r#"class="controls""#,
    r#"class="timeline""#,
    r#"class="audio-player""#,
    "bluearchivewiki",
    "story-background-image-container",
    "height=",
    "input type=",
    "width=",
    "cdx-",
    "vector-menu",
    "padding-left",
    "padding-right",
    "padding-top",
    "padding-bottom",
    "border-left-color",
    "border-right-color",
    "-2px;",
    "ul id=",
    "div id=\"right",
    "div id=\"left",
    "margin-",
    "placeholder=",
    "li id=",
    "window._paq",
    "line-container",
    "story-sensei-option",
    "px solid",
    "box-shadow",
    "display: block",
    "display: inline-block",
    "clear:both",
    "oo-ui",
    "setSiteId",
    "story-profile-picture",
    "<script>",
    "@context",
    "srcset=",
    "twitter:",
    "src=",
    "File:",
    "data-",
    "ExtLoops",
    "Creative Commons Attribution",
    "as BGM",
    "story-image",
    "story-student-name",
    "Episodes with",
    "Episodes_with",
    "Stories with",
    "title=",
    "<title>",
    "ba-momotalk",
    "cookiewarning",
    "our use of cookies",
    "disableCookies",
    "enableCookies",
    "setDocumentTitle",
    "href=",
    "content=",
    "skin-vector-legacy",
    "og:title",
    "stories_with",
    "RLSTATE",
    "CC BY-SA",
];

const SPLIT: &str = ".html:";

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
enum Line {
    Student {
        file_name: Rc<str>,
        name: String,
        text: String,
    },
    Sensei {
        file_name: Rc<str>,
        text: String,
    },
    Info {
        file_name: Rc<str>,
        text: String,
    },
    Description {
        file_name: Rc<str>,
        text: String,
    },
}

impl Line {
    fn file_name(&self) -> Rc<str> {
        match self {
            Self::Student {
                file_name,
                name: _,
                text: _,
            } => file_name.clone(),
            Self::Sensei { file_name, .. } => file_name.clone(),
            Self::Info { file_name, .. } => file_name.clone(),
            Self::Description { file_name, .. } => file_name.clone(),
        }
    }
    fn speaker(&self) -> String {
        match self {
            Self::Student {
                file_name: _,
                name,
                text: _,
            } => name.into(),
            Self::Sensei { .. } => "Sensei".into(),
            Self::Info { .. } => "INFO LINE".into(),
            Self::Description { .. } => "DESCRIPTION".into(),
        }
    }
    fn text(&self) -> String {
        match self {
            Self::Student {
                file_name: _,
                name: _,
                text,
            } => text.into(),
            Self::Sensei { file_name: _, text } => text.into(),
            Self::Info { file_name: _, text } => text.into(),
            Self::Description { file_name: _, text } => text.into(),
        }
    }
    /// Outline input with double asterisks
    fn outline(initial: &str, input: &str) -> String {
        // Assumes Self is already sanitized, so don't check for case
        let mut buffer: String = initial.into();
        let buffer_lower = buffer.to_lowercase();
        let input_lower = input.to_lowercase();
        let input_len = input.len();

        let asterisks = "**";

        let match_indices: Vec<usize> = buffer_lower
            .match_indices(&input_lower)
            .map(|(index, _)| index)
            .collect();

        // Loop backwards
        for index in match_indices.into_iter().rev() {
            // Insert suffix
            buffer.insert_str(index + input_len, asterisks);
            // Insert prefix
            buffer.insert_str(index, asterisks);
        }

        buffer
    }
}

fn matched_files(input: &str) -> Result<Vec<PathBuf>> {
    let mut set: HashSet<&str> = HashSet::with_capacity(input.lines().count());
    for line in input.lines() {
        if let Some(split) = line.split_once(SPLIT) {
            set.insert(split.0);
        }
    }
    let mut paths = Vec::with_capacity(set.len());
    for filtered in set {
        let path: PathBuf = {
            // Relative path
            let mut new = filtered.to_owned();
            new.push_str(".html");
            new.into()
        };
        if !path.exists() {
            return Err(anyhow!(ERROR_HEADER).context(format!("Path does not exist: {path:?}")));
        }
        if !path.is_file() {
            return Err(anyhow!(ERROR_HEADER).context(format!("Path is not a file: {path:?}")));
        }
        paths.push(path);
    }
    paths.sort();
    Ok(paths)
}

fn remove_double_spaces(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut last_was_blank = false;

    for c in input.chars() {
        if c == ' ' || c == '\t' || c == '\r' {
            if !last_was_blank {
                result.push(' '); // Convert whatever it was to a single space
                last_was_blank = true;
            }
        } else {
            result.push(c);
            last_was_blank = false;
        }
    }
    result
}

// Scrape file with for dialogue type; assumes `path` is already sanitized
#[builder]
fn scrape_from_match(
    path: &Path,
    input: &str,
    ignore_case: bool,
    word_regexp: bool,
    student: bool,
    sensei: bool,
    info: bool,
    description: bool,
) -> Result<Option<Vec<Line>>> {
    let file_contents = fs::read_to_string(path)?;
    let file_name: Rc<str> = path.to_string_lossy().into();

    let document = Html::parse_document(&file_contents);

    let td_all = Selector::parse("td").unwrap();
    let td_all = document.select(&td_all);

    let div_student_name = Selector::parse(r#"div[class="story-student-name"]"#).unwrap();
    let div_student_line = Selector::parse(r#"div[class="story-student-line-container"]"#).unwrap();
    let div_sensei_line = Selector::parse(r#"div[class="story-sensei-line"]"#).unwrap();
    let div_sensei_reply = Selector::parse(r#"div[class="story-reply-option"]"#).unwrap();
    let div_info_line = Selector::parse(r#"div[class="story-info-container"]"#).unwrap();
    let meta_description_line = Selector::parse(r#"meta[name="description"]"#).unwrap();

    let input_lower = input.to_lowercase();

    let matched_line = |line: &str| match (ignore_case, word_regexp) {
        (false, false) => line.contains(input),
        (true, false) => line.to_lowercase().contains(&input_lower),
        // Use alphanumeric checks, otherwise punctuation like "hello!" get ignored by "hello" as the input
        (false, true) => line
            .split(|c: char| !c.is_alphanumeric())
            .any(|word| word == input),
        (true, true) => line
            .split(|c: char| !c.is_alphanumeric())
            .any(|word| word.eq_ignore_ascii_case(input)),
    };

    let formatted_line = |input: &str| {
        let formatted = input.replace("\n", "");
        let formatted = remove_double_spaces(formatted.trim());
        formatted
    };

    let mut lines: Vec<Line> = vec![];

    if !description {
        if let Some(line) = document.select(&meta_description_line).next() {
            if let Some(description) = line.value().attr("content") {
                let text = formatted_line(&description);

                lines.push(Line::Description {
                    file_name: file_name.clone(),
                    text,
                });
            };
        }
    }

    for td in td_all {
        if !student {
            // Student line found
            match (
                td.select(&div_student_name).next(),
                td.select(&div_student_line).next(),
            ) {
                (Some(name), Some(line)) => {
                    let text: String = line.text().collect();

                    if !matched_line(&text) {
                        continue;
                    }

                    let text = formatted_line(&text);

                    let name: String = name.inner_html();
                    let name: String = {
                        if let Some(split) = name.split_once(r#"<span"#) {
                            split.0.trim().to_owned()
                        } else {
                            name.trim().to_owned()
                        }
                    };

                    lines.push(Line::Student {
                        file_name: file_name.clone(),
                        name,
                        text,
                    });
                }
                _ => {}
            }
        }

        // Sensei line found
        if !sensei {
            if let Some(line) = td.select(&div_sensei_line).next() {
                let text: String = line.text().collect();

                if !matched_line(&text) {
                    continue;
                }

                let text = formatted_line(&text);

                lines.push(Line::Sensei {
                    file_name: file_name.clone(),
                    text,
                });
            }
            if let Some(line) = td.select(&div_sensei_reply).next() {
                let text: String = line.text().collect();

                if !matched_line(&text) {
                    continue;
                }

                let text = formatted_line(&text);

                lines.push(Line::Sensei {
                    file_name: file_name.clone(),
                    text,
                });
            }
        }

        // Info line found
        if !info {
            if let Some(line) = td.select(&div_info_line).next() {
                let text: String = line.text().collect();

                if !matched_line(&text) {
                    continue;
                }

                let text = formatted_line(&text);

                lines.push(Line::Info {
                    file_name: file_name.clone(),
                    text,
                });
            }
        }
    }

    lines.sort();

    if lines.is_empty() {
        Ok(None)
    } else {
        Ok(Some(lines))
    }
}

/// Assumes sanitized
fn number_text(input: &str) -> Box<str> {
    // Might panic if empty
    if input.is_empty() {
        return "".into();
    }

    let lines: Vec<&str> = input.lines().collect();
    let width = match lines.len() {
        0..=9 => 1,
        10..=99 => 2,
        100..=999 => 3,
        1000..=9999 => 4,
        _ => 5,
    };
    let mut buffer = String::with_capacity(input.len() + lines.len() * (width + 2));

    for (i, line) in lines.into_iter().enumerate() {
        let _ = write!(buffer, "{:0width$}. {line}\n", i + 1);
    }

    buffer.truncate(buffer.len() - 1);

    buffer.into()
}

#[builder]
pub fn main(
    input: String,
    count: bool,
    ignore_case: bool,
    summary: bool,
    word_regexp: bool,
    outline: bool,
    numbered: bool,
    student: bool,
    sensei: bool,
    info: bool,
    description: bool,
) -> Result<()> {
    match (student, sensei, info) {
        (true, true, true) => {
            return Err(anyhow!(ERROR_HEADER).context(
                "Cannot throw away Student, Sensei, & Info lines. Keep at least one line type.",
            ));
        }
        _ => {}
    }

    let input = input.trim();

    // Verify inputs
    if input.is_empty() {
        return Err(anyhow::anyhow!(ERROR_HEADER).context("Format cannot be empty"));
    }

    let output = {
        let err_message = format!("Failed to run the following: rg {input:?}");
        match (ignore_case, word_regexp) {
            (true, true) => spawn_with_output!(rg --trim --sort=path --ignore-case -w $input)?
                .wait_with_output()
                .context(err_message)
                .ok(),
            (true, false) => spawn_with_output!(rg --trim --sort=path --ignore-case $input)?
                .wait_with_output()
                .context(err_message)
                .ok(),
            (false, true) => spawn_with_output!(rg --trim --sort=path -w $input)?
                .wait_with_output()
                .context(err_message)
                .ok(),
            (false, false) => spawn_with_output!(rg --trim --sort=path $input)?
                .wait_with_output()
                .context(err_message)
                .ok(),
        }
    };

    let output = if let Some(output) = output {
        output
    } else {
        println!("No matches for {input:?}");
        return Ok(());
    };

    let output = {
        let mut sanitized = String::with_capacity(output.len());
        let ignored: Vec<&str> = if !summary {
            // By default, ignore summaries
            let mut ignored = IGNORED.to_vec();
            ignored.push("<p>");
            ignored.push("content=");
            ignored
        } else {
            IGNORED.into()
        };
        for line in output.lines() {
            if !ignored.iter().any(|&text| line.contains(text)) {
                sanitized.push_str(line);
                sanitized.push('\n');
            }
        }
        sanitized
    };
    let output = output.trim_end();
    let num_lines = output.lines().count();

    let output = {
        let mut sanitized = String::with_capacity(output.len() + num_lines);
        for line in output.lines() {
            if let Some((initial, end)) = line.split_once(SPLIT) {
                sanitized.push_str(initial);
                sanitized.push_str(SPLIT);
                sanitized.push_str(" ");
                sanitized.push_str(end);
                sanitized.push('\n');
            }
        }
        sanitized
    };

    if count {
        println!("{num_lines}");
        return Ok(());
    }

    // Test paths
    let found_paths = matched_files(&output)?;

    let mut output: String = String::with_capacity(output.len());

    let lines: Vec<Line> = found_paths
        .into_iter()
        .filter_map(|path| {
            scrape_from_match()
                .path(&path)
                .input(&input)
                .ignore_case(ignore_case)
                .word_regexp(word_regexp)
                .student(student)
                .sensei(sensei)
                .info(info)
                .description(description)
                .call()
                .unwrap()
        })
        .flat_map(|lines| lines)
        .collect();

    // Remove duplicates
    let mut set: HashSet<Line> = HashSet::with_capacity(lines.len());
    for line in lines.iter() {
        set.insert(line.clone());
    }

    let lines: Vec<Line> = set.into_iter().map(|e| e).collect();

    // Concatenate lines
    for line in lines {
        let mut buffer = String::new();
        let text = line.text();

        if outline {
            let speaker_outlined = Line::outline(&line.speaker(), &line.speaker()).bold();
            let text_outlined = Line::outline(&text, input);
            let _ = writeln!(
                buffer,
                "{} ({}): {}",
                line.file_name(),
                speaker_outlined.italic(),
                text_outlined
            );
        } else {
            let _ = writeln!(
                buffer,
                "{} ({}): {}",
                line.file_name(),
                line.speaker().italic(),
                text
            );
        }
        output.push_str(&buffer);
    }

    let output_cloned = output.clone();
    let output = if ignore_case {
        spawn_with_output!(echo $output_cloned | rg --ignore-case $input)
            .unwrap()
            .wait_with_output()
            .unwrap()
    } else {
        spawn_with_output!(echo $output_cloned | rg $input)
            .unwrap()
            .wait_with_output()
            .unwrap()
    };
    // Just in case its unsorted
    let output = {
        let mut lines: Vec<&str> = output.lines().collect();

        lines.sort();
        lines.join("\n")
    };

    let output = if !numbered {
        number_text(&output)
    } else {
        output.into()
    };

    // Running rg once again for the highlighting
    // This may crash if too many lines, but it worked for 2,203,820 lines
    if ignore_case {
        run_cmd!(echo $output | rg --ignore-case $input)?;
    } else {
        run_cmd!(echo $output | rg $input)?;
    };

    Ok(())
}

#[rstest]
#[case("", "", "****")]
#[case("", " ", "")]
#[case("apple", "apple", "**apple**")]
#[case(" apple", "apple", " **apple**")]
#[case("apple ", "apple", "**apple** ")]
#[case("apple banana", " ", "apple** **banana")]
#[case("apple banana", "banana", "apple **banana**")]
#[case(
    "apple cherry banana cherry",
    "cherry",
    "apple **cherry** banana **cherry**"
)]
fn line_outline(#[case] text: &str, #[case] input: &str, #[case] expected: &str) -> Result<()> {
    let actual = Line::outline(text, input);

    assert_eq!(actual, expected.to_owned());

    Ok(())
}
