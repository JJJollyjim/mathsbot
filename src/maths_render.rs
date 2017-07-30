use std::process::{Command, Stdio, ExitStatus};
use std::os::unix::process::CommandExt;
use std::string;
use tempdir::TempDir;
use std::io;
use std::io::prelude::*;
use std::path::*;
use std::fs::*;
use libc::*;

#[derive(Debug)]
pub enum MathsRenderError {
    LatexError(ExitStatus, String),
    IOError(io::Error),
    Utf8Error,
    ConvertError,
}

impl From<string::FromUtf8Error> for MathsRenderError {
    fn from(_: string::FromUtf8Error) -> MathsRenderError {
        MathsRenderError::Utf8Error
    }
}

impl From<io::Error> for MathsRenderError {
    fn from(e: io::Error) -> MathsRenderError {
        MathsRenderError::IOError(e)
    }
}

fn maths_to_document(maths: &str) -> String {
    let mut str = "\\documentclass{standalone}
\\usepackage{amsmath}
\\begin{document}
$ \\displaystyle
".to_owned();

    str.push_str(maths);
    str.push_str("$ \\end{document}");
    str
}

fn make_tex_file(tempdir: &Path, maths: &str) -> Result<PathBuf, io::Error> {
    let path = tempdir.join("maths.tex");
    let mut texfile = File::create(path.as_path())?;
    texfile.write_all(maths_to_document(maths).as_bytes())?;

    Ok(path)
}

fn set_rlimit(resource: c_int, hard: rlim_t) {
    let rlim = rlimit {
        rlim_cur: hard,
        rlim_max: hard
    };

    unsafe {
        if setrlimit(resource, &rlim) != 0 {
            println!("setrlimit failed with {}", io::Error::last_os_error());
        }
    }
}

fn mbtob(mb: rlim_t) -> rlim_t {
    mb * 1024 * 1024
}

fn setlimits() -> io::Result<()> {
    set_rlimit(RLIMIT_CORE, 0);
    set_rlimit(RLIMIT_CPU, 4);
    set_rlimit(RLIMIT_DATA, mbtob(200));
    set_rlimit(RLIMIT_FSIZE, mbtob(10));
    set_rlimit(RLIMIT_MSGQUEUE, 1024);
    set_rlimit(RLIMIT_NOFILE, 200);
    set_rlimit(RLIMIT_RTTIME, 1);
    set_rlimit(RLIMIT_STACK, mbtob(10));
    unsafe {
        nice(10); // +ve priorities are lower
        alarm(10); // seconds
    }
    Ok(())
}

pub fn render_maths(fragment: &str) -> Result<Vec<u8>, MathsRenderError> {
    info!("rendering maths: `{}`", fragment);

    let dir = TempDir::new("mathsbot-maths-preview")?;
    let dirpath = dir.path();

    debug!("created tmp directory {:?}", dirpath);

    let tex = make_tex_file(dirpath, fragment)?;

    let pdflatex = Command::new("pdflatex")
        .current_dir(dirpath)
        .arg("-interaction=nonstopmode")
        .arg("-halt-on-error")
        .arg(tex.as_path())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .before_exec(setlimits)
        .output()?;

    if !pdflatex.status.success() {
        return Err(MathsRenderError::LatexError(pdflatex.status, String::from_utf8(pdflatex.stdout)?))
    }

    debug!("LaTeX complete");

    let convert = Command::new("convert")
        .current_dir(dirpath)
        .arg("-flatten")
        .arg("-density").arg("150")
        .arg(tex.with_extension("pdf"))
        .arg("-bordercolor").arg("none")
        .arg("-border").arg("10x10")
        .arg(tex.with_extension("png"))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .before_exec(setlimits)
        .output()?;

    if !convert.status.success() {
        return Err(MathsRenderError::ConvertError)
    }

    debug!("conversion complete");

    let mut image: Vec<u8> = Vec::new();
    File::open(tex.with_extension("png"))?.read_to_end(&mut image)?;
    debug!("image read");
    Ok(image)
}
