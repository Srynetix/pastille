// SPDX-License-Identifier: MIT OR Apache-2.0

//! Compiling an Arduino sketch for the Uno, with the machine's own
//! `arduino-cli` (ADR 0024).
//!
//! The compiler is GCC, which ADR 0001 keeps out of the repository, so it is
//! never shipped: the shell runs the one the user installed, as another
//! program, and reads back the Intel HEX the core already knows how to load.
//!
//! Arduino's own avr-gcc and ctags are Intel-only on macOS. On Apple silicon
//! without Rosetta, `arduino-cli` cannot start them; the shell then tries
//! again the way `programs/build.sh` builds the bundled programs: an avr-gcc
//! from the system (Homebrew's), and no ctags, so no prototypes are written
//! for the sketch and a function has to be declared before it is called.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The board `arduino-cli` builds for.
const FQBN: &str = "arduino:avr:uno";

/// What a person needs to do when there is no compiler to run.
pub const NOT_INSTALLED: &str = "Compiling a sketch needs arduino-cli and the Uno's core. \
     Install arduino-cli (https://arduino.github.io/arduino-cli/), then run: \
     arduino-cli core install arduino:avr";

/// What `arduino-cli` says when a tool it starts is built for another processor.
const WRONG_PROCESSOR: &str = "bad CPU type in executable";

/// Folders a desktop app is not given on its `PATH` on macOS, which is
/// where Homebrew puts what a person installs from a terminal.
const ALSO_SEARCHED: &[&str] = &["/opt/homebrew/bin", "/usr/local/bin"];

/// A sketch's name as `arduino-cli` accepts it, which is also its folder's:
/// letters, digits and underscores, starting with a letter or a digit.
pub fn sketch_name(name: &str) -> String {
    let stem = name
        .strip_suffix(".ino")
        .or_else(|| name.strip_suffix(".hex"))
        .unwrap_or(name);
    let cleaned: String = stem
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .skip_while(|c| *c == '_')
        .take(63)
        .collect();
    if cleaned.is_empty() {
        "sketch".to_string()
    } else {
        cleaned
    }
}

/// Where a program called `tool` is, on the `PATH` or where Homebrew puts it.
pub fn find(tool: &str) -> Option<PathBuf> {
    let file = if cfg!(windows) {
        format!("{tool}.exe")
    } else {
        tool.to_string()
    };
    let path = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&path)
        .chain(ALSO_SEARCHED.iter().map(PathBuf::from))
        .map(|folder| folder.join(&file))
        .find(|candidate| candidate.is_file())
}

/// Compiles `sketch` as `name` with the `arduino-cli` at `cli`, in a folder
/// of its own under `work`, and answers the Intel HEX without bootloader.
///
/// An error is what the compiler said, with the work folder taken out of its
/// paths so a line reads `Blink.ino:4:3: error: …`.
pub fn compile(cli: &Path, name: &str, sketch: &str, work: &Path) -> Result<String, String> {
    let name = sketch_name(name);
    let folder = work.join(&name);
    let output = work.join("out");
    std::fs::create_dir_all(&folder).map_err(|error| format!("could not compile: {error}"))?;
    std::fs::write(folder.join(format!("{name}.ino")), sketch)
        .map_err(|error| format!("could not compile: {error}"))?;

    let mut said = run(cli, &folder, &output, &[])?;
    if said
        .as_ref()
        .is_err_and(|text| text.contains(WRONG_PROCESSOR))
        && let Some(gcc) = find("avr-gcc")
    {
        said = run(cli, &folder, &output, &without_arduino_tools(&gcc, work)?)?;
    }
    said.map_err(|text| tidy(&text, work))?;

    std::fs::read_to_string(output.join(format!("{name}.ino.hex")))
        .map_err(|error| format!("arduino-cli said it compiled, but left no program: {error}"))
}

/// One run of `arduino-cli compile`: `Ok(Ok(()))` when it compiled,
/// `Ok(Err(what it said))` when it did not, `Err` when it could not start.
fn run(
    cli: &Path,
    folder: &Path,
    output: &Path,
    properties: &[String],
) -> Result<Result<(), String>, String> {
    let mut command = Command::new(cli);
    command
        .args(["compile", "--no-color", "--fqbn", FQBN, "--output-dir"])
        .arg(output);
    for property in properties {
        command.args(["--build-property", property]);
    }
    let done = command
        .arg(folder)
        .output()
        .map_err(|error| format!("could not start arduino-cli: {error}"))?;
    if done.status.success() {
        return Ok(Ok(()));
    }
    let mut text = String::from_utf8_lossy(&done.stderr).into_owned();
    text.push_str(&String::from_utf8_lossy(&done.stdout));
    Ok(Err(text))
}

/// The build properties that swap Arduino's avr-gcc for `gcc`'s folder and
/// its ctags for one that finds nothing, which is a script written in `work`.
fn without_arduino_tools(gcc: &Path, work: &Path) -> Result<Vec<String>, String> {
    let tools = work.join("tools");
    std::fs::create_dir_all(&tools).map_err(|error| format!("could not compile: {error}"))?;
    let ctags = tools.join("ctags");
    std::fs::write(&ctags, "#!/bin/sh\nexit 0\n")
        .map_err(|error| format!("could not compile: {error}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&ctags, std::fs::Permissions::from_mode(0o755))
            .map_err(|error| format!("could not compile: {error}"))?;
    }
    let gcc_folder = gcc.parent().unwrap_or(Path::new("/"));
    Ok(vec![
        format!("compiler.path={}/", gcc_folder.display()),
        format!("tools.ctags.path={}", tools.display()),
    ])
}

/// What the compiler said, without the work folder in its paths, and with
/// the one thing to do when the Uno's core is missing.
fn tidy(text: &str, work: &Path) -> String {
    let mut prefix = work.display().to_string();
    prefix.push(std::path::MAIN_SEPARATOR);
    let text = text.replace(&prefix, "");
    if text.contains("platform not installed") || text.contains("Platform 'arduino:avr' not found")
    {
        return format!("{}\n\n{NOT_INSTALLED}", text.trim());
    }
    text.trim().to_string()
}

/// A folder of its own for one compilation, removed when dropped.
pub struct Work(pub PathBuf);

impl Work {
    pub fn new() -> Result<Work, String> {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_nanos());
        let path =
            std::env::temp_dir().join(format!("pastille-sketch-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&path).map_err(|error| format!("could not compile: {error}"))?;
        Ok(Work(path))
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_becomes_one_arduino_cli_accepts() {
        assert_eq!(sketch_name("Blink"), "Blink");
        assert_eq!(sketch_name("Blink.ino"), "Blink");
        assert_eq!(sketch_name("my blink (2).hex"), "my_blink__2_");
        assert_eq!(sketch_name("_hidden"), "hidden");
        assert_eq!(sketch_name("été"), "t_");
        assert_eq!(sketch_name(""), "sketch");
        assert_eq!(sketch_name("…"), "sketch");
        assert_eq!(sketch_name(&"a".repeat(100)).len(), 63);
    }

    #[test]
    fn the_work_folder_is_taken_out_of_what_the_compiler_said() {
        let work = Path::new("/tmp/pastille-sketch-1-2");
        let said = "/tmp/pastille-sketch-1-2/Blink/Blink.ino:4:3: error: 'x' was not declared\n";
        assert_eq!(
            tidy(said, work),
            "Blink/Blink.ino:4:3: error: 'x' was not declared"
        );
    }

    #[test]
    fn a_missing_core_says_how_to_install_it() {
        let said = tidy(
            "Error during build: Platform 'arduino:avr' not found",
            Path::new("/w"),
        );
        assert!(said.ends_with(NOT_INSTALLED), "{said}");
    }

    #[cfg(unix)]
    fn stand_in(work: &Path, body: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let cli = work.join("arduino-cli");
        std::fs::write(&cli, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&cli, std::fs::Permissions::from_mode(0o755)).unwrap();
        cli
    }

    /// A stand-in `arduino-cli` that checks it is asked for an Uno and the
    /// sketch is where it should be, and writes a program as the real one does.
    #[cfg(unix)]
    #[test]
    fn a_sketch_is_compiled_for_the_uno_and_its_hex_read_back() {
        let work = Work::new().unwrap();
        let cli = stand_in(
            &work.0,
            r#"[ "$1" = compile ] && [ "$4" = arduino:avr:uno ] || exit 2
out="$6"; sketch="$7"; name=$(basename "$sketch")
grep -q 'void loop' "$sketch/$name.ino" || exit 3
mkdir -p "$out" && echo ':00000001FF' > "$out/$name.ino.hex""#,
        );
        let hex = compile(
            &cli,
            "My Blink.ino",
            "void setup() {}\nvoid loop() {}\n",
            &work.0,
        );
        assert_eq!(hex.unwrap(), ":00000001FF\n");
    }

    #[cfg(unix)]
    #[test]
    fn a_sketch_that_does_not_compile_answers_what_the_compiler_said() {
        let work = Work::new().unwrap();
        let folder = work.0.display().to_string();
        let cli = stand_in(
            &work.0,
            &format!(
                "echo '{folder}/Blink/Blink.ino:2:1: error: expected ; before }}' >&2; exit 1"
            ),
        );
        let said = compile(&cli, "Blink", "void loop() { x }", &work.0).unwrap_err();
        assert_eq!(said, "Blink/Blink.ino:2:1: error: expected ; before }");
    }

    /// What happens on Apple silicon without Rosetta: Arduino's tools do not
    /// start, and the second run is pointed at another avr-gcc and no ctags.
    #[cfg(unix)]
    #[test]
    fn arduino_s_intel_only_tools_are_swapped_when_they_cannot_start() {
        if find("avr-gcc").is_none() {
            return;
        }
        let work = Work::new().unwrap();
        let cli = stand_in(
            &work.0,
            r#"case "$*" in
*compiler.path=*tools.ctags.path=*) ;;
*) echo 'fork/exec ctags: bad CPU type in executable' >&2; exit 1 ;;
esac
for last; do :; done; name=$(basename "$last")
mkdir -p "$6" && echo ':00000001FF' > "$6/$name.ino.hex""#,
        );
        assert_eq!(
            compile(&cli, "Blink", "void loop() {}", &work.0).unwrap(),
            ":00000001FF\n"
        );
    }

    /// The real thing, where it is installed: the bundled Blink compiles to
    /// a program, whichever of the two ways this machine needs.
    #[test]
    fn the_bundled_blink_compiles_with_this_machine_s_arduino_cli() {
        let Some(cli) = find("arduino-cli") else {
            return;
        };
        let sketch = std::fs::read_to_string("../programs/Blink.ino").unwrap();
        let work = Work::new().unwrap();
        let hex = compile(&cli, "Blink", &sketch, &work.0).unwrap();
        assert!(
            hex.starts_with(':') && hex.trim_end().ends_with(":00000001FF"),
            "{hex}"
        );
    }
}
