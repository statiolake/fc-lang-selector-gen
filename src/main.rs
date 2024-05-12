use std::env;
use std::error::Error;
use std::fs;

use std::fs::File;
use std::path::PathBuf;

use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::process;

fn get_alias_dir() -> PathBuf {
    env::current_exe()
        .expect("Failed to get current executable's path.")
        .with_file_name("aliases")
}

#[derive(Debug)]
struct ReadFontNameError {
    description: String,
    path: PathBuf,
    error: Box<dyn Error>,
}

impl std::fmt::Display for ReadFontNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "error while reading {}", self.path.display())?;
        Ok(())
    }
}

impl ReadFontNameError {
    pub fn new<S>(description: S, path: PathBuf, error: Box<dyn Error>) -> ReadFontNameError
    where
        S: Into<String>,
    {
        ReadFontNameError {
            description: description.into(),
            path,
            error,
        }
    }
}

impl Error for ReadFontNameError {
    fn description(&self) -> &str {
        &self.description
    }

    fn cause(&self) -> Option<&dyn Error> {
        Some(&*self.error)
    }
}

fn read_font_name(
    mut alias_dir: PathBuf,
    kind: &str,
    alias: &str,
) -> Result<String, ReadFontNameError> {
    alias_dir.push(kind);
    alias_dir.push(alias);

    // println!("[reading {}", alias_dir.display());
    let f = File::open(&alias_dir).map_err(|e| {
        ReadFontNameError::new("failed to open file", alias_dir.clone(), Box::new(e))
    })?;
    let mut br = BufReader::new(f);
    let mut contents = String::new();
    br.read_to_string(&mut contents).map_err(|e| {
        ReadFontNameError::new("failed to read contents", alias_dir.clone(), Box::new(e))
    })?;

    Ok(contents.trim().to_string())
}

fn generate_xml<Sans, Serif, Monospace>(sans: Sans, serif: Serif, monospace: Monospace) -> String
where
    Sans: AsRef<str>,
    Serif: AsRef<str>,
    Monospace: AsRef<str>,
{
    format!(
        r#"<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "fonts.dtd">
<fontconfig>
    <match target="pattern">
        <test qual="any" name="family">
            <string>serif</string>
        </test>
        <edit name="family" mode="prepend" binding="strong">
            <string>{}</string>
        </edit>
    </match>
    <match target="pattern">
        <test qual="any" name="family">
            <string>sans-serif</string>
        </test>
        <edit name="family" mode="prepend" binding="strong">
            <string>{}</string>
        </edit>
    </match>
    <match target="pattern">
        <test qual="any" name="family">
            <string>monospace</string>
        </test>
        <edit name="family" mode="prepend" binding="strong">
            <string>{}</string>
        </edit>
    </match>
</fontconfig>
"#,
        serif.as_ref(),
        sans.as_ref(),
        monospace.as_ref()
    )
}

fn write_to_language_selector(is_system_wide: bool, xml: String) -> Result<(), Box<dyn Error>> {
    let path: PathBuf = if is_system_wide {
        PathBuf::from("/etc/fonts")
    } else if let Ok(config) = env::var("XDG_CONFIG_HOME") {
        PathBuf::from(&config).join("fontconfig")
    } else if let Ok(home) = env::var("HOME") {
        PathBuf::from(&home).join(".config").join("fontconfig")
    } else {
        PathBuf::from("/etc/fonts")
    }
    .join("conf.d")
    .join("69-language-selector-ja-jp.conf");
    println!("config file: {}", path.display());
    fs::create_dir_all(path.parent().unwrap())?;

    let f = File::create(path)?;
    let mut bw = BufWriter::new(f);
    write!(bw, "{}", xml)?;

    Ok(())
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let is_system_wide = args.contains(&"--sys".to_string());
    args.retain(|arg| arg != "--sys");

    if args.len() != 4 {
        eprintln!(
            "[EE] The number of arguments is incollect: expected 4 but {} supplied.",
            args.len()
        );
        process::exit(1);
    }
    let alias_dir = get_alias_dir();

    macro_rules! unwrap_font_name {
        ($kind:expr, $res:expr) => {
            match $res {
                Ok(name) => {
                    println!("{:>10} font: {}", $kind, name);
                    name
                }
                Err(e) => {
                    eprintln!("[EE] encounted error while reading {} font: {}", $kind, e);
                    eprintln!("[EE] ... which was caused by {:?}.", e.source().unwrap());
                    process::exit(1);
                }
            }
        };
    }

    let sans_font_name =
        unwrap_font_name!("sans", read_font_name(alias_dir.clone(), "sans", &args[1]));
    let serif_font_name = unwrap_font_name!(
        "serif",
        read_font_name(alias_dir.clone(), "serif", &args[2])
    );
    let mono_font_name = unwrap_font_name!(
        "monospace",
        read_font_name(alias_dir, "monospace", &args[3])
    );

    let xml = generate_xml(sans_font_name, serif_font_name, mono_font_name);
    if let Err(e) = write_to_language_selector(is_system_wide, xml) {
        eprintln!("[EE] error while writing to language selector: {}", e);
    }
    println!("successfully set fonts.");
}
