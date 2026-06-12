use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::Utc;

const SECTION_ORDER: [&str; 6] = [
    "Added",
    "Changed",
    "Deprecated",
    "Removed",
    "Fixed",
    "Security",
];

#[derive(Debug)]
struct CliError(String);

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for CliError {}

#[derive(Debug)]
struct ReleaseNotesArgs {
    tag: String,
    repo_url: String,
    cwd: PathBuf,
    output: PathBuf,
    release_date: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_args(env::args_os().skip(1))?;
    let notes = build_release_notes(&args.cwd, &args.tag, &args.repo_url, &args.release_date)?;

    if args.output == Path::new("-") {
        print!("{notes}");
        println!();
    } else {
        fs::write(&args.output, format!("{notes}\n"))?;
    }

    Ok(())
}

fn parse_args<I>(mut args: I) -> Result<ReleaseNotesArgs, Box<dyn Error>>
where
    I: Iterator<Item = OsString>,
{
    let mut tag = None;
    let mut repo_url = None;
    let mut cwd = env::current_dir()?;
    let mut output = PathBuf::from("-");
    let mut release_date = today_utc();

    while let Some(arg) = args.next() {
        let key = arg
            .into_string()
            .map_err(|_| CliError("Arguments must be valid UTF-8".into()))?;

        match key.as_str() {
            "--tag" => tag = Some(next_string(&mut args, "--tag")?),
            "--repo-url" => repo_url = Some(next_string(&mut args, "--repo-url")?),
            "--cwd" => cwd = PathBuf::from(next_string(&mut args, "--cwd")?),
            "--output" => output = PathBuf::from(next_string(&mut args, "--output")?),
            "--release-date" => release_date = next_string(&mut args, "--release-date")?,
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            other => return Err(Box::new(CliError(format!("Unknown argument: {other}")))),
        }
    }

    let tag = tag.ok_or_else(|| CliError("Missing required argument: --tag".into()))?;
    let repo_url =
        repo_url.ok_or_else(|| CliError("Missing required argument: --repo-url".into()))?;

    Ok(ReleaseNotesArgs {
        tag,
        repo_url,
        cwd,
        output,
        release_date,
    })
}

fn next_string<I>(args: &mut I, key: &str) -> Result<String, Box<dyn Error>>
where
    I: Iterator<Item = OsString>,
{
    let value = args
        .next()
        .ok_or_else(|| CliError(format!("Missing value for {key}")))?;
    Ok(value
        .into_string()
        .map_err(|_| CliError(format!("Value for {key} must be valid UTF-8")))?)
}

fn print_help() {
    println!(
        "usage: release-notes --tag TAG --repo-url URL [--cwd DIR] [--output FILE] [--release-date YYYY-MM-DD]"
    );
}

fn today_utc() -> String {
    Utc::now().date_naive().format("%F").to_string()
}

fn build_release_notes(
    cwd: &Path,
    tag: &str,
    repo_url: &str,
    release_date: &str,
) -> Result<String, Box<dyn Error>> {
    let previous_tag = resolve_previous_tag(cwd, tag)?;
    let commits = resolve_commit_subjects(cwd, tag, previous_tag.as_deref())?;
    Ok(render_release_notes(
        tag,
        previous_tag.as_deref(),
        release_date,
        repo_url,
        &commits,
    ))
}

fn resolve_previous_tag(cwd: &Path, current_tag: &str) -> Result<Option<String>, Box<dyn Error>> {
    let output = git(cwd, ["tag", "--sort=-creatordate"])?;
    for line in output.lines() {
        let tag = line.trim();
        if !tag.is_empty() && tag != current_tag {
            return Ok(Some(tag.to_string()));
        }
    }
    Ok(None)
}

fn resolve_commit_subjects(
    cwd: &Path,
    tag: &str,
    previous_tag: Option<&str>,
) -> Result<Vec<String>, Box<dyn Error>> {
    let Some(previous_tag) = previous_tag else {
        let output = git(cwd, ["log", "--no-merges", "--format=%s", tag])?;
        return Ok(output
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect());
    };

    let range = format!("{previous_tag}...{tag}");
    let output = git(
        cwd,
        [
            "log",
            "--left-right",
            "--no-merges",
            "--format=%m%x00%ae%x00%aI%x00%s",
            &range,
        ],
    )?;
    deduplicate_rebased_commits(&output)
}

fn deduplicate_rebased_commits(output: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut previous = HashMap::<String, usize>::new();
    let mut current = Vec::new();

    for line in output.lines().filter(|line| !line.is_empty()) {
        let mut fields = line.splitn(4, '\0');
        let side = fields.next();
        let email = fields.next();
        let authored_at = fields.next();
        let subject = fields.next();
        let (Some(side), Some(email), Some(authored_at), Some(subject)) =
            (side, email, authored_at, subject)
        else {
            return Err(Box::new(CliError(
                "Unexpected git log output while generating release notes".into(),
            )));
        };
        // ponytail: assumes rebase preserves author metadata and subject;
        // use Change-Id trailers if that stops holding.
        let identity = format!("{email}\0{authored_at}\0{subject}");
        match side {
            "<" => *previous.entry(identity).or_default() += 1,
            ">" => current.push((identity, subject.to_string())),
            _ => {
                return Err(Box::new(CliError(
                    "Unexpected git history side while generating release notes".into(),
                )))
            }
        }
    }

    Ok(current
        .into_iter()
        .filter_map(|(identity, subject)| match previous.get_mut(&identity) {
            Some(count) if *count > 0 => {
                *count -= 1;
                None
            }
            _ => Some(subject),
        })
        .collect())
}

fn git<const N: usize>(cwd: &Path, args: [&str; N]) -> Result<String, Box<dyn Error>> {
    let output = Command::new("git").args(args).current_dir(cwd).output()?;
    if !output.status.success() {
        return Err(Box::new(CliError(format!(
            "git command failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))));
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn render_release_notes(
    tag: &str,
    previous_tag: Option<&str>,
    release_date: &str,
    repo_url: &str,
    commits: &[String],
) -> String {
    let mut sections: [Vec<String>; 6] = Default::default();

    for subject in commits {
        let section = classify_subject(subject);
        let cleaned = normalize_subject(subject);
        let index = section_index(section);
        if !sections[index].contains(&cleaned) {
            sections[index].push(cleaned);
        }
    }

    let mut lines = vec![
        "# Changelog".to_string(),
        String::new(),
        format!("## [{}] - {}", tag.trim_start_matches('v'), release_date),
        String::new(),
    ];

    for (index, section_name) in SECTION_ORDER.iter().enumerate() {
        let items = &sections[index];
        if items.is_empty() {
            continue;
        }

        lines.push(format!("### {section_name}"));
        lines.push(String::new());
        for item in items {
            lines.push(format!("- {item}"));
        }
        lines.push(String::new());
    }

    if let Some(previous_tag) = previous_tag {
        lines.extend([
            "### Full Changelog".to_string(),
            String::new(),
            "<details>".to_string(),
            "<summary>Show full changelog</summary>".to_string(),
            String::new(),
            format!(
                "[Compare changes]({}/compare/{}...{})",
                repo_url.trim_end_matches('/'),
                previous_tag,
                tag
            ),
            String::new(),
            "</details>".to_string(),
        ]);
    } else {
        lines.pop();
    }

    lines.join("\n")
}

fn section_index(section: &'static str) -> usize {
    match section {
        "Added" => 0,
        "Changed" => 1,
        "Deprecated" => 2,
        "Removed" => 3,
        "Fixed" => 4,
        "Security" => 5,
        _ => 1,
    }
}

fn normalize_subject(subject: &str) -> String {
    let subject = strip_pr_suffix(subject);
    let subject = strip_conventional_prefix(&subject);
    strip_leading_action(&subject)
}

fn strip_pr_suffix(subject: &str) -> String {
    let mut trimmed = subject.trim().to_string();
    if let Some(start) = trimmed.rfind(" (#") {
        if trimmed.ends_with(')')
            && trimmed[start + 3..trimmed.len() - 1]
                .chars()
                .all(|c| c.is_ascii_digit())
        {
            trimmed.truncate(start);
            trimmed = trimmed.trim().to_string();
        }
    }
    trimmed
}

fn strip_conventional_prefix(subject: &str) -> String {
    let Some(colon) = subject.find(':') else {
        return subject.trim().to_string();
    };

    let prefix = &subject[..colon];
    if prefix
        .chars()
        .all(|c| c.is_ascii_alphabetic() || c == '(' || c == ')')
    {
        return subject[colon + 1..].trim().to_string();
    }

    subject.trim().to_string()
}

fn strip_leading_action(subject: &str) -> String {
    for prefix in [
        "add ",
        "added ",
        "fix ",
        "fixed ",
        "update ",
        "updated ",
        "remove ",
        "removed ",
        "delete ",
        "deleted ",
    ] {
        if subject.len() >= prefix.len() && subject[..prefix.len()].eq_ignore_ascii_case(prefix) {
            return subject[prefix.len()..].trim().to_string();
        }
    }
    subject.trim().to_string()
}

fn classify_subject(subject: &str) -> &'static str {
    let lower = subject.to_ascii_lowercase();
    if lower.contains("security") {
        "Security"
    } else if lower.starts_with("feat") || lower.starts_with("add") {
        "Added"
    } else if lower.starts_with("fix") || lower.starts_with("bugfix") {
        "Fixed"
    } else if lower.starts_with("remove") || lower.starts_with("delete") {
        "Removed"
    } else if lower.starts_with("deprecate") {
        "Deprecated"
    } else {
        "Changed"
    }
}

#[cfg(test)]
mod tests {
    use super::{deduplicate_rebased_commits, normalize_subject, render_release_notes};

    #[test]
    fn excludes_commits_replayed_by_rebase() {
        let commits = deduplicate_rebased_commits(concat!(
            "<\0dev@example.com\02026-06-01T10:00:00Z\0feat: login\n",
            ">\0upstream@example.com\02026-07-01T10:00:00Z\0fix: upstream bug\n",
            ">\0dev@example.com\02026-06-01T10:00:00Z\0feat: login\n",
        ))
        .unwrap();

        assert_eq!(commits, ["fix: upstream bug"]);
    }

    #[test]
    fn renders_keep_a_changelog_sections() {
        let notes = render_release_notes(
            "v1.2.3",
            Some("v1.2.2"),
            "2026-06-01",
            "https://github.com/example/repo",
            &[
                "feat: add api login enforcement".to_string(),
                "fix(security): update mio from 0.8.5 to 0.8.11 (#633)".to_string(),
                "docs: update README.md".to_string(),
                "refactor: simplify relay server flow".to_string(),
                "remove stale debug logging".to_string(),
                "fix: 127.0.0.1 is not loopback (#515)".to_string(),
            ],
        );

        let expected = [
            "# Changelog",
            "",
            "## [1.2.3] - 2026-06-01",
            "",
            "### Added",
            "",
            "- api login enforcement",
            "",
            "### Changed",
            "",
            "- README.md",
            "- simplify relay server flow",
            "",
            "### Removed",
            "",
            "- stale debug logging",
            "",
            "### Fixed",
            "",
            "- 127.0.0.1 is not loopback",
            "",
            "### Security",
            "",
            "- mio from 0.8.5 to 0.8.11",
            "",
            "### Full Changelog",
            "",
            "<details>",
            "<summary>Show full changelog</summary>",
            "",
            "[Compare changes](https://github.com/example/repo/compare/v1.2.2...v1.2.3)",
            "",
            "</details>",
        ]
        .join("\n");

        assert_eq!(notes, expected);
    }

    #[test]
    fn strips_conventional_prefixes_and_actions() {
        assert_eq!(
            normalize_subject("fix(security): update mio"),
            "mio"
        );
        assert_eq!(
            normalize_subject("chore: Added kubernetes example file (#623)"),
            "kubernetes example file"
        );
        assert_eq!(
            normalize_subject("remove stale debug logging"),
            "stale debug logging"
        );
    }
}
