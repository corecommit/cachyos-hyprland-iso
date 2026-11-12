mod clip;
mod conf;
mod power;
mod ui;
mod volume;
mod wifi;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let which = args.first().map(String::as_str).unwrap_or("");
    if !matches!(which, "wifi" | "volume" | "power" | "clipboard") {
        eprintln!("usage: hypr-ui wifi|volume|power|clipboard");
        std::process::exit(2);
    }
    if gtk::init().is_err() {
        eprintln!("hypr-ui: cannot open the display");
        std::process::exit(1);
    }
    let conf = conf::Conf::load();
    match which {
        "wifi" => wifi::run(&conf),
        "volume" => volume::run(&conf),
        "power" => power::run(&conf),
        _ => clip::run(&conf),
    }
    gtk::main();
}
