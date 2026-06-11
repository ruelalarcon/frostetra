use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;

use structopt::StructOpt;

#[derive(StructOpt)]
struct CliOptions {
    /// Enable puffin profiler (requires building with feature `puffin_http`)
    #[structopt(long)]
    #[allow(dead_code)]
    profile: bool,

    /// Path to JSON file containing the bot configuration
    #[structopt(short, long)]
    config: Option<PathBuf>,
}

fn main() {
    let options = CliOptions::from_args();

    #[cfg(feature = "puffin_http")]
    let _puffin_server = match options.profile {
        true => {
            puffin::set_scopes_on(true);
            Some(puffin_http::Server::new(&format!(
                "0.0.0.0:{}",
                puffin_http::DEFAULT_PORT
            )))
        }
        false => None,
    };

    let config = options.config.map_or_else(Default::default, |path| {
        let f = BufReader::new(File::open(path).unwrap());
        Arc::new(serde_json::from_reader(f).unwrap())
    });

    let (sender, incoming) = futures::channel::mpsc::unbounded();
    std::thread::spawn(move || loop {
        let mut line = String::new();
        let bytes_read = std::io::stdin().read_line(&mut line).unwrap();
        if bytes_read == 0 {
            break;
        }
        let msg = serde_json::from_str(&line).unwrap();
        if sender.unbounded_send(msg).is_err() {
            break;
        }
    });

    let outgoing = futures::sink::unfold((), |_, msg| {
        serde_json::to_writer(std::io::stdout(), &msg).unwrap();
        println!();
        async { Ok(()) }
    });

    futures::pin_mut!(outgoing);

    futures::executor::block_on(frostetra::run(incoming, outgoing, config));
}
