mod bravewatch;
mod genwall;
mod hypr;
mod tools;
mod wsdaemon;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(String::as_str) {
        Some("genwall") => genwall::run(&args[1..]),
        Some("ws-daemon") => wsdaemon::run(),
        Some("brave-wall-watch") => bravewatch::run(),
        Some("popup-pos") => tools::popup_pos(&args[1..]),
        Some("kb-name") => tools::kb_name(),
        _ => {
            eprintln!("usage: hypr-helper genwall|ws-daemon|brave-wall-watch|popup-pos|kb-name");
            2
        }
    };
    std::process::exit(code);
}
