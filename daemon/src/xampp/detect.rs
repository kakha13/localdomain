use localdomain_shared::protocol::DetectXamppResult;

/// Auto-detect XAMPP installation path by checking platform defaults.
pub fn detect_xampp() -> DetectXamppResult {
    let candidates = get_platform_candidates();

    for path in candidates {
        if verify_xampp_path(&path) {
            return DetectXamppResult {
                found: true,
                path: Some(path),
            };
        }
    }

    DetectXamppResult {
        found: false,
        path: None,
    }
}

/// Verify that a given path is a valid XAMPP installation by checking for the httpd binary.
pub fn verify_xampp_path(path: &str) -> bool {
    let httpd = get_httpd_binary(path);
    std::path::Path::new(&httpd).exists()
}

#[cfg(target_os = "macos")]
fn get_platform_candidates() -> Vec<String> {
    vec!["/Applications/XAMPP/xamppfiles".to_string()]
}

#[cfg(target_os = "linux")]
fn get_platform_candidates() -> Vec<String> {
    vec!["/opt/lampp".to_string()]
}

#[cfg(target_os = "windows")]
fn get_platform_candidates() -> Vec<String> {
    vec![
        "C:\\xampp".to_string(),
        "D:\\xampp".to_string(),
    ]
}

#[cfg(target_os = "macos")]
pub fn get_httpd_binary(xampp_path: &str) -> String {
    format!("{}/bin/httpd", xampp_path)
}

#[cfg(target_os = "linux")]
pub fn get_httpd_binary(xampp_path: &str) -> String {
    format!("{}/bin/httpd", xampp_path)
}

#[cfg(target_os = "windows")]
pub fn get_httpd_binary(xampp_path: &str) -> String {
    format!("{}\\apache\\bin\\httpd.exe", xampp_path)
}
