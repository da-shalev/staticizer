fn main() {
    // The compiler crates live in the sysroot: link against it, and find it again at load time.
    let rustc = std::env::var("RUSTC").unwrap();
    let output = std::process::Command::new(&rustc)
        .args(["--print", "sysroot"])
        .output();
    let lib = format!(
        "{}/lib",
        String::from_utf8(output.unwrap().stdout).unwrap().trim()
    );
    println!("cargo::rustc-link-search=native={lib}");
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() != "windows" {
        println!("cargo::rustc-link-arg=-Wl,-rpath,{lib}");
    }

    // Nightly's `DataLayout::pointer_align` returns `Align`, stable's returns `AbiAlign`.
    println!("cargo::rustc-check-cfg=cfg(nightly)");
    let version = std::process::Command::new(&rustc).arg("--version").output();
    if String::from_utf8(version.unwrap().stdout)
        .unwrap()
        .contains("nightly")
    {
        println!("cargo::rustc-cfg=nightly");
    }
}
