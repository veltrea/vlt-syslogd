fn main() {
    // winres は Windows ホストでビルドするときだけ意味を持つ。
    // macOS / Linux ホストでは何もしない。
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../icons/vlt-syslogd.ico");
        res.set("ProductName", "vlt-syslogd-portable");
        res.set("FileDescription", "vlt-syslogd Portable (GUI)");
        res.set("LegalCopyright", "Copyright (c) 2026 veltrea");
        res.compile().unwrap();
    }
}
