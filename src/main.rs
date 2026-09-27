use std::{
    env::{
        self,
        consts::{DLL_PREFIX, DLL_SUFFIX},
    },
    ffi::{OsStr, OsString},
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::Path,
    process::{Command, exit},
};

/// Compiled by each toolchain the first time Staticizer sees it, so it always matches the rustc
/// that loads it.
const HOOK: &str = include_str!("hook.rs");

/// Runs the compiler Cargo chose (`rustc`, or `clippy-driver rustc` under `cargo clippy`) with
/// the hook loaded and every dependency forced to load, so records in dependencies the code
/// never names are collected too.
fn main() {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    let rustc = (args.iter().take(2))
        .find(|arg| Path::new(arg).file_stem() == Some(OsStr::new("rustc")))
        .expect("staticizer runs as Cargo's `build.rustc-wrapper`");
    let mut command = Command::new(&args[0]);
    let mut rest = args[1..].iter();
    while let Some(arg) = rest.next() {
        command.arg(arg);
        if arg == "--extern"
            && let Some(spec) = rest.next()
        {
            let named = spec.as_encoded_bytes().split(|&byte| byte == b'=').next();
            let mut forced = OsString::from(if named.is_some_and(|name| name.contains(&b':')) {
                "force,"
            } else {
                "force:"
            });
            forced.push(spec);
            command.arg(forced);
        }
    }
    let mut backend = OsString::from("-Zcodegen-backend=");
    backend.push(hook(rustc));
    let status = command
        .args(["-Zunstable-options".into(), backend])
        .status();
    exit(
        status
            .expect("staticizer cannot run the compiler")
            .code()
            .unwrap_or(1),
    );
}

/// Returns the hook built by `rustc`, building it the first time this toolchain is seen.
fn hook(rustc: &OsStr) -> std::path::PathBuf {
    let run = |args: &[&str]| {
        let output = Command::new(rustc)
            .args(args)
            .output()
            .expect("staticizer cannot run rustc");
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    };
    let mut hash = DefaultHasher::new();
    (run(&["-vV"]), HOOK).hash(&mut hash);
    let cache = env::var_os("XDG_CACHE_HOME")
        .map_or_else(|| env::home_dir().unwrap().join(".cache"), Into::into);
    let dir = cache
        .join("staticizer")
        .join(format!("{:016x}", hash.finish()));
    let hook = dir.join(format!("{DLL_PREFIX}staticizer_hook{DLL_SUFFIX}"));
    if !hook.exists() {
        // Parallel compilations may build it at once; each builds in its own directory and the
        // atomic rename keeps whichever finishes last.
        let building = dir.join(std::process::id().to_string());
        fs::create_dir_all(&building).unwrap();
        fs::write(building.join("staticizer_hook.rs"), HOOK).unwrap();
        let built = Command::new(rustc)
            .args([
                "--edition=2024",
                "--crate-type=dylib",
                "-Copt-level=3",
                "--out-dir",
            ])
            .arg(&building)
            .arg(format!("-Lnative={}/lib", run(&["--print", "sysroot"])))
            .arg(building.join("staticizer_hook.rs"))
            .status()
            .is_ok_and(|status| status.success());
        assert!(
            built,
            "staticizer: building the compiler hook failed; the toolchain needs the `rustc-dev` component"
        );
        fs::rename(building.join(hook.file_name().unwrap()), &hook).unwrap();
        fs::remove_dir_all(building).unwrap();
    }
    hook
}
