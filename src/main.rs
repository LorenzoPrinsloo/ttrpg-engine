pub mod modules;

pub use modules::{bags, config, encounters, markup, shell};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let (root, loaded) = match config::load() {
        Ok(loaded) => loaded,
        Err(errors) => {
            for error in errors {
                eprintln!("{error}");
            }
            std::process::exit(1);
        }
    };

    /* Only when there is a session to have: a one-shot command should print
       nothing but its answer. */
    if args.is_empty() {
        println!(
            "{} bags and {} tables from {}",
            loaded.bags.len(),
            loaded.tables.len(),
            root.display()
        );
    }

    shell::run(loaded.bags.into_iter().collect(), loaded.tables.into_iter().collect(), &args);
}
