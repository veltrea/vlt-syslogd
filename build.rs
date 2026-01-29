fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        println!("cargo:rerun-if-changed=vlt_syslogd_icon.ico");
        let mut res = winres::WindowsResource::new();
        res.set_icon("vlt_syslogd_icon.ico");
        res.compile().unwrap();
    }
}
