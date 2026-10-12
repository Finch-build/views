//! The apps installed on this machine, for the picker to list. Only the Settings UI
//! lists them, so this stays out of the backends. Other systems report none.

use serde::Serialize;

use crate::icons::Icon;

/// One executable of an installed app, under the name the system shows for the app.
#[derive(Serialize)]
pub struct Installed {
    name: String,
    exe: String,
    icon: Option<Icon>,
}

/// The installed apps, in the order of their names, with no executable listed twice.
#[tauri::command]
pub async fn installed_apps() -> Vec<Installed> {
    #[cfg(windows)]
    return windows_apps::all();
    #[cfg(not(windows))]
    Vec::new()
}

#[cfg(windows)]
mod windows_apps {
    use std::cmp::Reverse;
    use std::collections::HashSet;
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};

    use serde::Deserialize;
    use windows::Win32::Foundation::MAX_PATH;
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
        CoUninitialize, IPersistFile, STGM_READ,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
    use windows::core::{HSTRING, Interface};
    use windows_registry::{CURRENT_USER, Key, LOCAL_MACHINE};

    use super::Installed;
    use crate::icons;

    const UNINSTALL: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";
    const UNINSTALL_32: &str = r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall";

    /// What the Start menu launches, then the executables of each app that records
    /// where it is installed.
    pub fn all() -> Vec<Installed> {
        let found =
            shortcuts().into_iter().chain(folders().into_iter().flat_map(|(name, folder)| {
                largest(&folder).into_iter().map(move |exe| (name.clone(), exe))
            }));
        let mut seen = HashSet::new();
        let mut apps: Vec<Installed> = found
            .filter_map(|(name, path)| {
                let exe = path.file_name()?.to_str()?.to_owned();
                let listed = is_executable(&path)
                    && !is_helper(&path)
                    && path.is_file()
                    && seen.insert(exe.to_lowercase());
                listed.then(|| Installed { icon: icons::of(&path.to_string_lossy()), name, exe })
            })
            .collect();
        apps.sort_by_key(|app| (app.name.to_lowercase(), app.exe.to_lowercase()));
        apps
    }

    /// The name and target of every shortcut in the Start menu, the machine's and the user's.
    fn shortcuts() -> Vec<(String, PathBuf)> {
        // SAFETY: balanced by the `CoUninitialize` below whenever it succeeds.
        let initialised = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
        let found = ["ProgramData", "APPDATA"]
            .into_iter()
            .filter_map(env::var_os)
            .flat_map(|root| {
                files(&Path::new(&root).join(r"Microsoft\Windows\Start Menu\Programs"), 4)
            })
            .filter(|path| path.extension().is_some_and(|kind| kind.eq_ignore_ascii_case("lnk")))
            .filter_map(|link| Some((link.file_stem()?.to_str()?.to_owned(), target(&link)?)))
            .collect();
        if initialised {
            // SAFETY: this thread initialised COM above, and every COM object is dropped.
            unsafe { CoUninitialize() };
        }
        found
    }

    /// The file the shortcut at `link` opens.
    fn target(link: &Path) -> Option<PathBuf> {
        let mut path = [0; MAX_PATH as usize];
        // SAFETY: COM is initialised on this thread, and `path` is the buffer the call fills.
        unsafe {
            let shortcut: IShellLinkW =
                CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
            shortcut.cast::<IPersistFile>().ok()?.Load(&HSTRING::from(link), STGM_READ).ok()?;
            shortcut.GetPath(&mut path, std::ptr::null_mut(), 0).ok()?;
        }
        let length = path.iter().position(|&unit| unit == 0)?;
        Some(String::from_utf16_lossy(&path[..length]).into())
    }

    /// The name and folder of each app that records where it is installed: those in the
    /// uninstall list, and the games of the Epic launcher, which are not in it.
    fn folders() -> Vec<(String, PathBuf)> {
        #[derive(Deserialize)]
        #[serde(rename_all = "PascalCase")]
        struct Manifest {
            display_name: String,
            install_location: PathBuf,
        }

        let uninstall =
            [(LOCAL_MACHINE, UNINSTALL), (LOCAL_MACHINE, UNINSTALL_32), (CURRENT_USER, UNINSTALL)]
                .into_iter()
                .filter_map(|(hive, path)| hive.open(path).ok())
                .flat_map(|list| {
                    let apps: Vec<Key> = list
                        .keys()
                        .into_iter()
                        .flatten()
                        .filter_map(|app| list.open(app).ok())
                        .collect();
                    apps
                })
                .filter_map(|app| {
                    let folder = app.get_string("InstallLocation").ok()?;
                    Some((app.get_string("DisplayName").ok()?, folder.trim_matches('"').into()))
                });
        let epic = env::var_os("ProgramData")
            .map(|root| Path::new(&root).join(r"Epic\EpicGamesLauncher\Data\Manifests"))
            .into_iter()
            .flat_map(|manifests| files(&manifests, 0))
            .filter_map(|file| {
                serde_json::from_str::<Manifest>(&fs::read_to_string(file).ok()?).ok()
            })
            .map(|manifest| (manifest.display_name, manifest.install_location));
        uninstall
            .chain(epic)
            // Some launchers write `/`, which the shell does not take for an icon's path.
            .map(|(name, folder): (String, PathBuf)| (name, folder.components().collect()))
            // A drive is not an app's folder, whatever an installer wrote.
            .filter(|(_, folder): &(String, PathBuf)| folder.parent().is_some())
            .collect()
    }

    /// The few largest executables in `folder`, largest first: an app's own executable
    /// dwarfs the tools shipped beside it, and a game's is rarely the one in its root.
    fn largest(folder: &Path) -> Vec<PathBuf> {
        let mut exes: Vec<(u64, PathBuf)> = files(folder, 4)
            .into_iter()
            .filter(|path| is_executable(path) && !is_helper(path))
            .filter_map(|path| Some((fs::metadata(&path).ok()?.len(), path)))
            .collect();
        exes.sort_by_key(|(size, _)| Reverse(*size));
        let mut seen = HashSet::new();
        exes.into_iter()
            .map(|(_, path)| path)
            .filter(|path| seen.insert(path.file_name().map(|name| name.to_ascii_lowercase())))
            .take(3)
            .collect()
    }

    fn is_executable(path: &Path) -> bool {
        path.extension().is_some_and(|kind| kind.eq_ignore_ascii_case("exe"))
    }

    /// Whether `path` is by its name an installer, an uninstaller or a crash reporter:
    /// a program that comes with an app and is never the app.
    fn is_helper(path: &Path) -> bool {
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
        ["unins", "setup", "install", "redist", "crash"].iter().any(|word| name.contains(word))
    }

    /// The files in `folder` and in the folders within it, down to `depth` of them.
    fn files(folder: &Path, depth: usize) -> Vec<PathBuf> {
        let Ok(entries) = fs::read_dir(folder) else {
            return Vec::new();
        };
        entries
            .flatten()
            .flat_map(|entry| match entry.file_type() {
                Ok(kind) if kind.is_file() => vec![entry.path()],
                Ok(kind) if kind.is_dir() && depth > 0 => files(&entry.path(), depth - 1),
                _ => Vec::new(),
            })
            .collect()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn the_largest_executables_of_a_folder_come_first_without_its_helpers() {
            let folder = env::temp_dir().join(format!("views-installed-{}", std::process::id()));
            let deep = folder.join("bin").join("x64");
            fs::create_dir_all(&deep).unwrap();
            for (file, size) in [
                (folder.join("launcher.exe"), 10),
                (folder.join("Setup.exe"), 900),
                (folder.join("readme.txt"), 800),
                (deep.join("Game.EXE"), 500),
                (deep.join("tool.exe"), 20),
                (deep.join("tiny.exe"), 1),
            ] {
                fs::write(file, vec![0; size]).unwrap();
            }
            let names: Vec<_> =
                largest(&folder).iter().map(|path| path.file_name().unwrap().to_owned()).collect();
            fs::remove_dir_all(&folder).unwrap();
            assert_eq!(names, ["Game.EXE", "tool.exe", "launcher.exe"]);
        }

        #[test]
        fn the_start_menu_and_the_uninstall_list_yield_apps_in_name_order() {
            let apps = all();
            assert!(apps.iter().all(|app| app.exe.to_lowercase().ends_with(".exe")));
            assert!(apps.is_sorted_by_key(|app| (app.name.to_lowercase(), app.exe.to_lowercase())));
        }
    }
}
