use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use clap::Args;
use dialoguer::FuzzySelect;

use crate::{
    atcoder::{self, client::AtCoderClient, models::TaskInfo},
    cli::{
        self, Outcome,
        styles::{self, ERROR_PREFIX, dialog_theme::DialogTheme, spinner::Spinner},
        template::expand_template,
        utils,
    },
    config::models::Config,
    error::AppError,
    languages::{LANGUAGES, Language},
};

/// Set up a directory for a contest with one subdirectory per problem.
#[derive(Debug, Args)]
pub struct SetupArgs {
    /// Contest ID or contest URL (e.g. "abc350" or https://atcoder.jp/contests/abc350)
    pub contest_id: String,

    /// Parent directory of the contest directory
    /// [default: `[setup].parent`, or the current directory]
    #[arg(short, long, value_name = "DIR")]
    pub directory: Option<PathBuf>,

    /// Also download problem statements as Markdown into each problem directory
    #[arg(long)]
    pub md: bool,
}

/// Effective `[setup]` parameters resolved from a config and a language preset.
struct SetupPlan {
    dir: String,
    files: Vec<String>,
    templates: BTreeMap<String, String>,
}

/// Config to apply plus the preset it came from, when known.
struct ResolvedSetup {
    config: Config,
    lang: Option<&'static Language>,
    /// `false` when the config was synthesized from an interactive prompt.
    from_disk: bool,
}

#[derive(Default)]
struct Stats {
    dirs_created: usize,
    files_created: usize,
    skipped: usize,
}

pub fn run(args: &SetupArgs) -> Result<Outcome, AppError> {
    let Some(contest_id) = atcoder::utils::get_contest_id(&args.contest_id) else {
        return Err(AppError::InvalidContest(args.contest_id.clone()));
    };

    let Some(resolved) = resolve_config()? else {
        println!("Canceled.");
        return Ok(Outcome::Success);
    };

    if !resolved.from_disk {
        info_ln!(
            "No atcoder.toml found; source templates only. Run 'acc init' to create a config."
        );
    }

    let plan = build_plan(&resolved.config, resolved.lang);
    if plan.files.is_empty() && resolved.lang.is_none() {
        warn_ln!("Could not determine the language; creating directories only.");
        warn_ln!("Add a [setup] section to atcoder.toml to create source files.");
    }

    // Precedence: --directory flag > [setup].parent > current directory.
    let base_dir = match &args.directory {
        Some(dir) => dir.clone(),
        None => resolved
            .config
            .setup
            .as_ref()
            .and_then(|s| s.parent.as_deref())
            .map(expand_tilde)
            .unwrap_or_default(),
    };
    let contest_dir = base_dir.join(&contest_id);

    let cookie = utils::session_cookie()?;
    let client = AtCoderClient::new(cookie);

    let tasks = fetch_task_list(&client, &contest_id)?;
    println!();

    let mut stats = Stats::default();
    for task in &tasks {
        prepare_task(&contest_dir, task, &plan, &mut stats)?;
    }
    println!();

    if args.md && !download_statements(&client, &contest_dir, &plan, &tasks, &mut stats)? {
        failed_ln!("Setup finished with errors: no problem statements could be downloaded");
        return Ok(Outcome::Failed);
    }

    success_ln!(
        "Setup complete under {}",
        styles::success(&contest_dir.display().to_string())
    );
    info_ln!(
        "{} director(ies) created, {} file(s) created, {} skipped",
        stats.dirs_created,
        stats.files_created,
        stats.skipped
    );

    Ok(Outcome::Success)
}

/// Determines which config (and language preset) the new directories use.
///
/// Returns `Ok(None)` when the user cancels the interactive language prompt.
fn resolve_config() -> Result<Option<ResolvedSetup>, AppError> {
    if let Some(config) = utils::load_config()? {
        let lang = LANGUAGES.iter().find(|l| matches_preset(&config, l));
        return Ok(Some(ResolvedSetup {
            config,
            lang,
            from_disk: true,
        }));
    }

    // No config anywhere: fall back to an interactive selection like acc init.
    cli::ensure_interactive()?;

    let names: Vec<&str> = LANGUAGES.iter().map(|l| l.name).collect();
    let selection = FuzzySelect::with_theme(&DialogTheme::default())
        .with_prompt("Choose your favorite programming language")
        .default(0)
        .items(&names)
        .interact_opt()?;

    let Some(index) = selection else {
        return Ok(None);
    };

    let lang = &LANGUAGES[index];
    let mut config = Config::from(&lang.config);
    config.setup = Some(lang.default_setup());

    Ok(Some(ResolvedSetup {
        config,
        lang: Some(lang),
        from_disk: false,
    }))
}

/// Compares a user config against a language preset, ignoring any `[setup]`
/// section so that layout customizations do not hide the preset match.
fn matches_preset(config: &Config, lang: &Language) -> bool {
    let preset = Config::from(&lang.config);

    config.compiler == preset.compiler
        && config.compiler_flags == preset.compiler_flags
        && config.run == preset.run
        && config.run_flags == preset.run_flags
}

/// Merges `[setup]` values with their defaults derived from the matched preset.
fn build_plan(config: &Config, lang: Option<&'static Language>) -> SetupPlan {
    let setup = config.setup.as_ref();

    let files = match setup.and_then(|s| s.file.clone()) {
        Some(files) => files,
        None => lang
            .map(|l| l.files.iter().map(|(p, _)| (*p).to_string()).collect())
            .unwrap_or_default(),
    };

    SetupPlan {
        dir: setup
            .and_then(|s| s.dir.clone())
            .unwrap_or_else(|| "${TASK_CHAR}".to_string()),
        files,
        templates: setup.and_then(|s| s.template.clone()).unwrap_or_default(),
    }
}

fn fetch_task_list(client: &AtCoderClient, contest_id: &str) -> Result<Vec<TaskInfo>, AppError> {
    let spinner = Spinner::new("Fetching task list...");

    match client.get_task_list(contest_id) {
        Ok(tasks) => {
            spinner.finish(
                &format!("{}", styles::success("200 OK")),
                &format!("{} task(s) found.", tasks.len()),
            );
            Ok(tasks)
        }
        Err(e) => {
            let code = e
                .as_http()
                .and_then(|http| http.status())
                .map_or_else(|| String::from(ERROR_PREFIX), |s| s.to_string());
            spinner.finish(
                &format!("{}", styles::error(&code)),
                "Failed to fetch task list:",
            );
            Err(e.into())
        }
    }
}

/// Creates one problem directory with its source files for `task`.
fn prepare_task(
    contest_dir: &Path,
    task: &TaskInfo,
    plan: &SetupPlan,
    stats: &mut Stats,
) -> Result<(), AppError> {
    let vars = task.to_map();
    let problem_dir = safe_join(contest_dir, &expand_template(&plan.dir, &vars))?;

    if !problem_dir.exists() {
        std::fs::create_dir_all(&problem_dir)?;
        stats.dirs_created += 1;
        info_ln!("Created {}", problem_dir.display());
    }

    for pattern in &plan.files {
        let file_path = safe_join(&problem_dir, &expand_template(pattern, &vars))?;

        if let Some(parent) = file_path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }

        if file_path.exists() {
            warn_ln!("Skip existing {}", file_path.display());
            stats.skipped += 1;
            continue;
        }

        let body = source_body(pattern, plan);
        std::fs::write(&file_path, body)?;
        stats.files_created += 1;
    }

    Ok(())
}

/// Downloads problem statements as `problem.md` into each problem directory.
///
/// Individual failures are reported as warnings; returns `Ok(false)` only
/// when not a single statement could be downloaded.
fn download_statements(
    client: &AtCoderClient,
    contest_dir: &Path,
    plan: &SetupPlan,
    tasks: &[TaskInfo],
    stats: &mut Stats,
) -> Result<bool, AppError> {
    let mut downloaded = 0;

    for task in tasks {
        let vars = task.to_map();
        let problem_dir = safe_join(contest_dir, &expand_template(&plan.dir, &vars))?;
        let md_path = problem_dir.join("problem.md");

        if md_path.exists() {
            warn_ln!("Skip existing {}", md_path.display());
            stats.skipped += 1;
            downloaded += 1; // nothing left to do for this task
            continue;
        }

        info_ln!("Fetching statement for {}...", task.task_id);
        match client.get_page(&task.url) {
            Ok(page) => {
                std::fs::write(&md_path, page.problem_md)?;
                downloaded += 1;
            }
            Err(e) => {
                warn_ln!("Could not download {}: {}", task.task_id, e);
            }
        }
    }

    Ok(downloaded > 0)
}

/// Chooses the template body for a source file.
///
/// Priority: inline `[setup.template]` by extension, then a builtin preset
/// template matched by exact relative path (searched across all languages so
/// that standard names like `main.cpp` get a skeleton even with a fully
/// custom config), otherwise an empty file.
fn source_body(path: &str, plan: &SetupPlan) -> String {
    if let Some(ext) = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        && let Some(body) = plan.templates.get(&ext)
    {
        return body.clone();
    }

    for lang in LANGUAGES {
        if let Some((_, body)) = lang.files.iter().find(|(p, _)| *p == path) {
            return (*body).to_string();
        }
    }

    String::new()
}

/// Joins `expanded` onto `base`, refusing absolute paths or `..` components.
fn safe_join(base: &Path, expanded: &str) -> Result<PathBuf, AppError> {
    let rel = PathBuf::from(expanded);

    let escapes = rel.is_absolute() || rel.components().any(|c| matches!(c, Component::ParentDir));

    if escapes {
        return Err(AppError::InvalidSetupPath(expanded.to_string()));
    }

    Ok(base.join(rel))
}

/// Expands a leading `~` in a config-provided path to the home directory.
fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        return dirs::home_dir().unwrap_or_default();
    }

    if let Some(rest) = path.strip_prefix("~/") {
        return dirs::home_dir().map_or_else(|| PathBuf::from(path), |home| home.join(rest));
    }

    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_tilde_to_home_directory() {
        let home = dirs::home_dir().expect("home directory must exist");

        assert_eq!(expand_tilde("~"), home);
        assert_eq!(
            expand_tilde("~/atcoder/contests"),
            home.join("atcoder/contests")
        );
    }

    #[test]
    fn leaves_other_paths_untouched() {
        assert_eq!(expand_tilde("contests"), PathBuf::from("contests"));
        assert_eq!(
            expand_tilde("/tmp/opencode/x"),
            PathBuf::from("/tmp/opencode/x")
        );
        // ~user 形式は展開しない
        assert_eq!(expand_tilde("~root/x"), PathBuf::from("~root/x"));
    }
}
