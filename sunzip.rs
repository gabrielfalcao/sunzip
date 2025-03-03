use clap::Parser;
use crc::{
    Algorithm, Crc, CRC_32_AIXM, CRC_32_AUTOSAR, CRC_32_BASE91_D, CRC_32_BZIP2, CRC_32_CD_ROM_EDC,
    CRC_32_CKSUM, CRC_32_ISCSI, CRC_32_ISO_HDLC, CRC_32_JAMCRC, CRC_32_MEF, CRC_32_MPEG_2,
    CRC_32_XFER, CRC_64_ECMA_182, CRC_64_GO_ISO, CRC_64_MS, CRC_64_REDIS, CRC_64_WE, CRC_64_XZ,
};
use iocore::Path;
use sevenz_rust::Password;
use sevenz_rust::SevenZArchiveEntry;
use sevenz_rust::SevenZReader;
use sunzip::errors::Result;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "sunzip command-line utility")]
pub struct Cli {
    #[arg()]
    path: Path,

    #[arg(short, long, env = "SUNZIP_PASSWORD")]
    password: Option<String>,

    #[arg(short, long)]
    force: bool,

    #[arg(short, long)]
    quiet: bool,
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

pub fn crc_64_algorithms() -> Vec<String> {
    vec![
        format!("ECMA_182"),
        format!("GO_ISO"),
        format!("MS"),
        format!("REDIS"),
        format!("WE"),
        format!("XZ"),
    ]
}
pub fn crc_32_algorithms() -> Vec<String> {
    vec![
        format!("AIXM"),
        format!("AUTOSAR"),
        format!("BASE91_D"),
        format!("BZIP2"),
        format!("CD_ROM_EDC"),
        format!("CKSUM"),
        format!("ISCSI"),
        format!("ISO_HDLC"),
        format!("JAMCRC"),
        format!("MEF"),
        format!("MPEG_2"),
        format!("XFER"),
    ]
}

pub fn crc_64_algorithm(name: &str) -> &'static Algorithm<u64> {
    match name {
        "ECMA_182" => &CRC_64_ECMA_182,
        "GO_ISO" => &CRC_64_GO_ISO,
        "MS" => &CRC_64_MS,
        "REDIS" => &CRC_64_REDIS,
        "WE" => &CRC_64_WE,
        "XZ" => &CRC_64_XZ,
        unknown => panic!("unknown CRC64 algorithm {}", unknown),
    }
}
pub fn crc_32_algorithm(name: &str) -> &'static Algorithm<u32> {
    match name {
        "AIXM" => &CRC_32_AIXM,
        "AUTOSAR" => &CRC_32_AUTOSAR,
        "BASE91_D" => &CRC_32_BASE91_D,
        "BZIP2" => &CRC_32_BZIP2,
        "CD_ROM_EDC" => &CRC_32_CD_ROM_EDC,
        "CKSUM" => &CRC_32_CKSUM,
        "ISCSI" => &CRC_32_ISCSI,
        "ISO_HDLC" => &CRC_32_ISO_HDLC,
        "JAMCRC" => &CRC_32_JAMCRC,
        "MEF" => &CRC_32_MEF,
        "MPEG_2" => &CRC_32_MPEG_2,
        "XFER" => &CRC_32_XFER,
        unknown => panic!("unknown CRC32 algorithm {}", unknown),
    }
}

pub fn crc_64(data: &[u8]) -> Vec<(u64, String)> {
    let mut result = Vec::<(u64, String)>::new();
    for algorithm_name in crc_64_algorithms() {
        let crc = Crc::<u64>::new(crc_64_algorithm(&algorithm_name));
        let mut digest = crc.digest();
        digest.update(data);
        result.push((digest.finalize(), algorithm_name.to_string()));
    }
    result
}
pub fn crc_32(data: &[u8]) -> Vec<(u32, String)> {
    let mut result = Vec::<(u32, String)>::new();
    for algorithm_name in crc_32_algorithms() {
        let crc = Crc::<u32>::new(crc_32_algorithm(&algorithm_name));
        let mut digest = crc.digest();
        digest.update(data);
        result.push((digest.finalize(), algorithm_name.to_string()));
    }
    result
}
fn main() -> Result<()> {
    let args = Cli::parse();
    let path = args.path.try_canonicalize();
    let password = args
        .password
        .clone()
        .map(|password| Password::from(password.as_str()))
        .or_else(|| Some(Password::empty()))
        .unwrap();
    let mut reader = SevenZReader::open(&path.path(), password)?;
    let cwd = Path::cwd().join(path.without_extension().name());
    reader.for_each_entries(|entry: &SevenZArchiveEntry, reader| {
        let name = entry.name.to_string();
        args.println(format!("decompressing {}", &name));
        let mut bytes = Vec::<u8>::new();
        let bytecount = reader.read_to_end(&mut bytes)?;
        let path = cwd.join(&name);
        if path.is_file() && !args.force {
            let oldbytes = path.read_bytes().expect(&format!("read {}", &path));
            let mut crc64matches = false;
            let mut crc32matches = false;
            let mut matched_crc_algorithm_name: Option<String> = None;
            args.eprintln(format!(
                "{} already exists with CRC {} ({} compressed)",
                &path.relative_to_cwd(),
                hex::encode(&entry.crc.to_ne_bytes()),
                hex::encode(&entry.compressed_crc.to_ne_bytes())
            ));

            for (crc64, algorithm_name) in crc_64(&oldbytes) {
                args.eprintln(format!(
                    "calculating CRC 64 {}: {}",
                    &algorithm_name,
                    hex::encode(&crc64.to_ne_bytes())
                ));
                if crc64 == entry.crc {
                    crc64matches = true;
                    matched_crc_algorithm_name = Some(algorithm_name.to_string())
                }
            }
            for (crc32, algorithm_name) in crc_32(&oldbytes) {
                let crc32 = Into::<u64>::into(crc32);
                args.eprintln(format!(
                    "calculating CRC 32 {}: {}",
                    &algorithm_name,
                    hex::encode(&crc32.to_ne_bytes())
                ));
                if crc32 == entry.crc {
                    crc32matches = true;
                    matched_crc_algorithm_name = Some(algorithm_name.to_string())
                }
            }
            if crc64matches || crc32matches {
                args.eprintln(format!(
                    "{} matches successfully",
                    matched_crc_algorithm_name.expect("algorithm name")
                ));
            } else if !crc32matches {
                args.eprintln(format!("WARNING: no valid CRC32 match found!"));
            } else {
                args.eprintln(format!("WARNING: no valid CRC64 match found!"));
            }
        } else {
            path.write(&bytes).expect(&format!("write {}", &path));
            args.println(format!(
                "wrote {} bytes to {} (crc: {})",
                bytecount,
                &path.relative_to_cwd(),
                hex::encode(&entry.crc.to_ne_bytes())
            ));
        }
        Ok(true)
    })?;
    Ok(())
}
