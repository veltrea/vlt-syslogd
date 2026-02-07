fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../icons/vlt-syslogd.ico");
        res.set("ProductName", "vlt-syslog-srv");
        res.set("FileDescription", "vlt-syslogd Server Engine (Windows Service)");
        res.set("LegalCopyright", "Copyright (c) 2026 veltrea");
        res.compile().unwrap();
    }
}
