//! Isolated catalog/launch test tool. Never included in native packages.
fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).is_some_and(|s| s == "record") {
        println!("This application stdout must never enter the panel protocol");
        std::fs::write(&args[2], serde_json::to_vec(&args[3..]).unwrap()).unwrap();
        return;
    }
    let store = asaaan_apps::Store::default();
    let reply = store.search("", true).unwrap();
    #[cfg(target_os = "linux")]
    if args.get(1).is_some_and(|s| s == "refresh") {
        let file = std::path::PathBuf::from(&args[2]);
        std::fs::write(
            &file,
            "[Desktop Entry]\nType=Application\nName=New Watched App\nExec=/bin/true\n",
        )
        .unwrap();
        let wait = |wanted: usize| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            loop {
                let context = gio::glib::MainContext::default();
                while context.pending() {
                    context.iteration(false);
                }
                let snapshot = store.search("", true).unwrap();
                if snapshot.total == wanted {
                    break;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "GIO did not observe a desktop-entry change"
                );
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
        };
        wait(reply.total + 1);
        std::fs::remove_file(file).unwrap();
        wait(reply.total);
        println!("refresh passed");
        return;
    }
    if args.get(1).is_some_and(|s| s == "removed") {
        let app = store.get(&args[2]).unwrap();
        std::fs::remove_file(&app.location).unwrap();
        assert!(asaaan_apps::launch(&app).is_err());
        println!("removed rejected");
        return;
    }
    if args.get(1).is_some_and(|s| s == "launch") {
        asaaan_apps::launch(&store.get(&args[2]).unwrap()).unwrap();
        println!("launched");
    } else {
        println!("{}", serde_json::to_string(&reply).unwrap());
    }
}
