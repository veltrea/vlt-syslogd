fn main() {
    // winres は Windows ホストでビルドするときだけ依存に含まれる
    // （Cargo.toml の [target.'cfg(windows)'.build-dependencies] を参照）。
    // macOS / Linux ホストでは何もしない。
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../icons/vlt-syslogd.ico");
        res.set("ProductName", "vlt-syslogd-srv");
        res.set("FileDescription", "vlt-syslogd Server Engine (Windows Service)");
        res.set("LegalCopyright", "Copyright (c) 2026 veltrea");
        res.compile().unwrap();
    }
}
