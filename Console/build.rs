fn main() {
    // winres は Windows ホストでビルドするときだけ実効性がある
    // （macOS / Linux ホストでは何もしない）。
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../icons/vlt-syslogd.ico");
        res.set("ProductName", "vlt-syslogd-console");
        res.set("FileDescription", "vlt-syslogd Console (GUI Frontend)");
        res.set("LegalCopyright", "Copyright (c) 2026 veltrea");
        res.compile().unwrap();
    }
}
