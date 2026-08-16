use {
    std::{
        env,
        io,
    },
    winres::WindowsResource,
};

fn main() -> io::Result<()> {
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let mut res = WindowsResource::new();
        // This path can be absolute, or relative to your crate root.
        res.set_icon("./icon/icon.ico");

        // Cross-compiling to Windows from a non-Windows host (e.g. via the
        // x86_64-pc-windows-gnu target): winres defaults to plain "windres"/"ar" on unix
        // hosts, but mingw-w64 only installs the triple-prefixed binaries.
        if cfg!(unix) {
            res.set_windres_path("x86_64-w64-mingw32-windres");
            res.set_ar_path("x86_64-w64-mingw32-ar");
        }

        res.compile()?;
    }
    Ok(())
}