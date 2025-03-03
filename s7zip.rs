use clap::Parser;
use iocore::{walk_dir, NoopProgressHandler, Path};
use sevenz_rust::SevenZWriter;
use sevenz_rust::SevenZArchiveEntry;
use std::fs::File;
use sunzip::errors::Result;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "s7zip command-line utility")]
pub struct Cli {
    #[arg(short, long, required = true)]
    output_path: Path,

    #[arg(short, long)]
    force: bool,

    #[arg(short, long)]
    quiet: bool,

    #[arg()]
    source_paths: Vec<Path>,
}
impl Cli {
    pub fn println(&self, string: impl std::fmt::Display) {
        if !self.quiet {
            println!("{}", string);
        }
    }
    pub fn eprintln(&self, string: impl std::fmt::Display) {
        if !self.quiet {
            eprintln!("{}", string);
        }
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let output_path = args.output_path.try_canonicalize();
    match output_path.extension() {
        None => {}
        Some(extension) => match extension.as_str() {
            ".7z" => {}
            _ => {
                args.eprintln(format!(
                    "{} has an unrecognized extension {}",
                    &output_path.relative_to_cwd(),
                    extension
                ));
                std::process::exit(1);
            }
        },
    }
    if output_path.is_file() && !args.force {
        eprintln!("{} already exists", &output_path);
        std::process::exit(1);
    } else if output_path.is_dir() {
        eprintln!("{} is a folder", &output_path);
        std::process::exit(1);
    }
    let mut archive = SevenZWriter::<File>::create(&output_path.path())?;
    let mut source_paths = Vec::<Path>::new();
    for paths in args.source_paths.iter().map(|path| {
        if path.is_dir() {
            match walk_dir(path, NoopProgressHandler, None, None) {
                Ok(entries) => entries
                    .iter()
                    .map(|entry| entry.path())
                    .collect::<Vec<Path>>(),
                Err(e) => {
                    args.eprintln(format!("scanning {}: {}", &path, e));
                    std::process::exit(1);
                }
            }
        } else {
            vec![path.clone()]
        }
    }) {
        for path in paths {
            source_paths.push(path.try_canonicalize());
        }
    }

    for path in source_paths {
        let name = path.relative_to_cwd().to_string().replace("../", "");
        args.println(format!("compressing {}", name));
        archive.push_archive_entry(
            SevenZArchiveEntry::from_path(path.path(), name),
            Some(File::open(path.path())?)
        )?;
        // archive.push_source_path_non_solid(&path.relative_to_cwd().path(), |_| true)?;
    }
    archive.finish()?;

    Ok(())
}
