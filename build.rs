fn main() {
    // Give the Windows executable its icon and version information
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/scaleri.ico");
        res.set("ProductName", "Scaleri");
        res.set("FileDescription", "Scaleri Minecraft launcher");
        if let Err(err) = res.compile() {
            println!("cargo:warning=could not embed the Windows icon: {err}");
        }
    }
}
