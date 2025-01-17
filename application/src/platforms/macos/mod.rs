use std::path::{Path,
                PathBuf};

pub fn start_service() {
    unimplemented!("WhiteBeam: Service control is not currently supported on MacOS");
}

pub fn stop_service() {
    unimplemented!("WhiteBeam: Service control is not currently supported on MacOS");
}

pub fn get_data_file_path(data_file: &str) -> PathBuf {
    #[cfg(feature = "whitelist_test")]
    let data_path: String = format!("{}/target/release/examples/", env!("PWD"));
    #[cfg(not(feature = "whitelist_test"))]
    let data_path: String = String::from("/Applications/WhiteBeam/data/");
    let data_file_path = data_path + data_file;
    PathBuf::from(data_file_path)
}

pub fn get_syslog_path() -> PathBuf {
    unimplemented!("WhiteBeam: Syslog is not currently supported on MacOS");
}

pub fn path_open_secure(file_path: &Path) {
    unimplemented!("WhiteBeam: Secure file opening is not currently supported on MacOS");
}

pub fn is_superuser() -> bool {
    unimplemented!("WhiteBeam: Checking user capabilities is not currently supported on MacOS");
}
