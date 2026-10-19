use std::io::Write;
/// Returns before constructing Tauri, GTK, a resident, a browser or any source service.
pub fn start() -> Option<i32> {
    let args = match std::env::args_os()
        .skip(1)
        .map(|s| s.into_string())
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(args) => args,
        Err(_) => {
            eprintln!("Command-line arguments must be UTF-8");
            return Some(64);
        }
    };
    let options = match asaaan_apps::search_command::parse(&args) {
        Ok(Some(options)) => options,
        Ok(None) => return None,
        Err(error) => {
            eprintln!("{error}");
            return Some(64);
        }
    };
    let path = match crate::data::personalization_path() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("{error}");
            return Some(78);
        }
    };
    let snapshot = match asaaan_personalization::Store::open(path) {
        Ok(store) => store.snapshot(),
        Err(error) => {
            eprintln!("Search preferences unavailable: {error}");
            return Some(78);
        }
    };
    let store = asaaan_apps::Store::default();
    let output = match asaaan_apps::search_command::render(
        &store,
        &options.query,
        snapshot.context(&options.query, asaaan_personalization::now()),
    ) {
        Ok(output) => output,
        Err(error) => {
            eprintln!("Application search unavailable: {error}");
            return Some(69);
        }
    };
    let mut stdout = std::io::stdout().lock();
    match stdout
        .write_all(output.as_bytes())
        .and_then(|()| stdout.flush())
    {
        Ok(()) => Some(0),
        Err(error) => {
            eprintln!("Cannot write search output: {error}");
            Some(74)
        }
    }
}
