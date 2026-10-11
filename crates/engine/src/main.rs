//! The Engine executable: the platform's backend, handed to the Engine's logic.

#![windows_subsystem = "windows"]

#[cfg(target_os = "macos")]
use views_macos as os;
#[cfg(windows)]
use views_windows as os;

fn main() {
    // There is no host to be had while another Engine runs in this session.
    if let Some(mut host) = os::Host::new() {
        let path = os::config_path();
        views_engine::run(os::Desktop, &mut host, || std::fs::read_to_string(&path));
    }
}
