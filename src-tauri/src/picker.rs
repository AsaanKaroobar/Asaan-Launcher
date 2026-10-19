// Standalone picker. No resident, catalog, preferences, shortcut or clipboard initialization.
use asaaan_search::picker::{self, Item};
use std::io::{IsTerminal, Read, Write};
use std::sync::{Mutex, atomic::AtomicBool};
use tauri::Manager;
pub const HELP: &str = "Asaaan Launcher --search QUERY: print the first eight ranked applications without a window.
Asaaan Launcher --settings [--page general|applications]: open Settings.
Asaaan Launcher --pick [--prompt TEXT] [--query TEXT] [--index]\nReads stdin, one item per line. Tabs separate title and detail; returns the whole line.\nExit: 0 chosen, 1 cancelled/empty, 64 usage. ASAAAN_PICK_AUTO=1 chooses the first match without a window.";
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Options {
    pub prompt: Option<String>,
    pub query: String,
    pub index: bool,
}
pub fn parse(args: &[String]) -> Result<Option<Options>, String> {
    if !args.iter().any(|a| a == "--pick") {
        if args.iter().any(|a| {
            a == "--index"
                || a.starts_with("--index=")
                || a.starts_with("--pick=")
                || a == "--query"
                || a == "--prompt"
                || a.starts_with("--query=")
                || a.starts_with("--prompt=")
        }) {
            return Err("--prompt, --query and --index require --pick".into());
        }
        return Ok(None);
    }
    let mut options = Options::default();
    let mut seen = std::collections::HashSet::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        let (flag, inline) = arg
            .split_once('=')
            .map_or((arg.as_str(), None), |(k, v)| (k, Some(v)));
        if !seen.insert(flag) {
            return Err(format!("Repeated option: {flag}"));
        }
        match flag {
            "--pick" | "--index" if inline.is_none() => {
                if flag == "--index" {
                    options.index = true;
                }
            }
            "--prompt" | "--query" => {
                let value = inline
                    .map(str::to_owned)
                    .or_else(|| iter.next().filter(|v| !v.starts_with("--")).cloned())
                    .ok_or_else(|| format!("{flag} requires text"))?;
                if flag == "--prompt" {
                    options.prompt = Some(value);
                } else {
                    options.query = value;
                }
            }
            _ => return Err(format!("Unsupported picker option: {arg}")),
        }
    }
    Ok(Some(options))
}
pub struct Picker {
    items: Vec<Item>,
    options: Options,
    finished: Mutex<bool>,
    pub focused: AtomicBool,
    pub profile: Mutex<Option<tempfile::TempDir>>,
}
#[derive(serde::Serialize)]
pub struct View {
    prompt: String,
    query: String,
    rows: Vec<Item>,
    total: usize,
    count: String,
}
impl Picker {
    fn view(&self, query: String) -> View {
        let rows = picker::matches(&self.items, &query);
        let count = picker::count_label(rows.len(), self.items.len(), !query.is_empty());
        View {
            prompt: self
                .options
                .prompt
                .clone()
                .unwrap_or_else(|| "Search…".into()),
            query,
            rows,
            total: self.items.len(),
            count,
        }
    }
    pub fn cancel(&self, app: &tauri::AppHandle) {
        let mut finished = self.finished.lock().unwrap();
        if !*finished {
            *finished = true;
            app.exit(picker::CANCELLED);
        }
    }
}
pub enum Start {
    Normal,
    Picker(Picker),
    Exit(i32),
}
pub fn start() -> Start {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{HELP}");
        return Start::Exit(0);
    }
    let options = match parse(&args) {
        Ok(Some(options)) => options,
        Ok(None) => return Start::Normal,
        Err(e) => {
            eprintln!("{e}\n{HELP}");
            return Start::Exit(picker::USAGE);
        }
    };
    if std::io::stdin().is_terminal() {
        eprintln!("{HELP}");
        return Start::Exit(picker::USAGE);
    }
    let mut bytes = vec![];
    if let Err(e) = std::io::stdin().read_to_end(&mut bytes) {
        eprintln!("Cannot read picker input: {e}");
        return Start::Exit(74);
    }
    let items = picker::items(&picker::decode(&bytes));
    if items.is_empty() {
        return Start::Exit(picker::CANCELLED);
    }
    if std::env::var_os("ASAAAN_PICK_AUTO").is_some() {
        let result = picker::matches(&items, &options.query);
        return Start::Exit(if let Some(item) = result.first() {
            match writeln!(
                std::io::stdout().lock(),
                "{}",
                picker::output(item, options.index)
            ) {
                Ok(()) => picker::CHOSEN,
                Err(e) => {
                    eprintln!("Cannot write picker output: {e}");
                    74
                }
            }
        } else {
            picker::CANCELLED
        });
    }
    let profile = match tempfile::Builder::new().prefix("asaaan-picker-").tempdir() {
        Ok(profile) => profile,
        Err(e) => {
            eprintln!("Cannot prepare picker webview: {e}");
            return Start::Exit(74);
        }
    };
    Start::Picker(Picker {
        items,
        options,
        finished: Mutex::new(false),
        focused: AtomicBool::new(false),
        profile: Mutex::new(Some(profile)),
    })
}
#[tauri::command]
pub async fn picker_view(app: tauri::AppHandle, query: Option<String>) -> Result<View, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.try_state::<Picker>().ok_or("Picker unavailable")?;
        Ok(state.view(query.unwrap_or_else(|| state.options.query.clone())))
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn picker_finish(
    app: tauri::AppHandle,
    index: Option<usize>,
    query: String,
) -> Result<(), String> {
    let state = app.try_state::<Picker>().ok_or("Picker unavailable")?;
    if let Some(index) = index {
        let mut finished = state.finished.lock().unwrap();
        if *finished {
            return Err("Picker already finished".into());
        }
        let item = picker::matches(&state.items, &query)
            .into_iter()
            .find(|i| i.index == index)
            .ok_or("Selection no longer matches")?;
        writeln!(
            std::io::stdout().lock(),
            "{}",
            picker::output(&item, state.options.index)
        )
        .map_err(|e| format!("Cannot write picker output: {e}"))?;
        *finished = true;
        app.exit(picker::CHOSEN);
    } else {
        state.cancel(&app);
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn args(s: &[&str]) -> Vec<String> {
        s.iter().map(|s| s.to_string()).collect()
    }
    #[test]
    fn picker_options_are_isolated_and_strict() {
        assert_eq!(
            parse(&args(&[
                "--pick",
                "--prompt",
                "Pick",
                "--query=nav",
                "--index"
            ]))
            .unwrap(),
            Some(Options {
                prompt: Some("Pick".into()),
                query: "nav".into(),
                index: true
            })
        );
        for a in [
            vec!["--query", "x"],
            vec!["--index"],
            vec!["--index=true"],
            vec!["--pick=true"],
            vec!["--pick", "--panel"],
            vec!["--pick", "--query"],
            vec!["--pick", "--query", "--index"],
            vec!["--pick", "--pick"],
            vec!["--pick", "--index=true"],
        ] {
            assert!(parse(&args(&a)).is_err(), "{a:?}");
        }
        assert_eq!(parse(&args(&["--panel"])).unwrap(), None);
    }
}
