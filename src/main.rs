use std::env;
use std::error::Error;

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
    pub path: PathBuf,
    error: Box<dyn Error>,
}

impl std::fmt::Display for ReadFontNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.description)?;
        Ok(())
    }
}

impl ReadFontNameError {
    pub fn new(path: PathBuf, error: Box<dyn Error>) -> ReadFontNameError {
        ReadFontNameError {
            description: format!("error while reading {}", path.display()),
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
    let f = File::open(&alias_dir)
        .map_err(|e| ReadFontNameError::new(alias_dir.clone(), Box::new(e)))?;
    let mut br = BufReader::new(f);
    let mut contents = String::new();
    br.read_to_string(&mut contents)
        .map_err(|e| ReadFontNameError::new(alias_dir.clone(), Box::new(e)))?;

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

fn write_to_language_selector(xml: String) -> Result<(), Box<dyn Error>> {
    let path: PathBuf = [
        "/",
        "etc",
        "fonts",
        "conf.d",
        "69-language-selector-ja-jp.conf",
    ]
    .iter()
    .collect();
    // println!("writing at {}", path.display());

    let f = File::create(path)?;
    let mut bw = BufWriter::new(f);
    write!(bw, "{}", xml)?;

    Ok(())
}

fn main() {
    let args: Vec<_> = env::args().collect();
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
    if let Err(e) = write_to_language_selector(xml) {
        eprintln!("[EE] error while writing to language selector: {}", e);
    }
    println!("successfully set fonts.");
}
