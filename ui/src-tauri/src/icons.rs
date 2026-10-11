//! The icon of an executable, as pixels for the page to paint. Only the Settings UI
//! shows icons, so this stays out of the backends. Other systems have none.

use serde::Serialize;

/// A square of pixels, four bytes each: red, green, blue, alpha.
#[derive(Serialize)]
pub struct Icon {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[cfg(not(windows))]
pub fn of(_path: &str) -> Option<Icon> {
    None
}

/// The small icon of the file at `path`.
#[cfg(windows)]
pub fn of(path: &str) -> Option<Icon> {
    use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
    use windows::Win32::UI::Shell::{SHFILEINFOW, SHGFI_ICON, SHGFI_SMALLICON, SHGetFileInfoW};
    use windows::Win32::UI::WindowsAndMessaging::DestroyIcon;
    use windows::core::HSTRING;

    let mut info = SHFILEINFOW::default();
    // SAFETY: `info` is the size passed with it, and `path` outlives the call.
    let found = unsafe {
        SHGetFileInfoW(
            &HSTRING::from(path),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut info),
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_SMALLICON,
        )
    };
    if found == 0 {
        return None;
    }
    let icon = windows_icon::read(info.hIcon);
    // SAFETY: the call above handed this icon to us to destroy.
    let _ = unsafe { DestroyIcon(info.hIcon) };
    icon
}

#[cfg(windows)]
mod windows_icon {
    use std::ffi::c_void;

    use windows::Win32::Graphics::Gdi::{
        BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetDC,
        GetDIBits, GetObjectW, HBITMAP, ReleaseDC,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetIconInfo, HICON, ICONINFO};

    use super::Icon;

    pub fn read(icon: HICON) -> Option<Icon> {
        let mut info = ICONINFO::default();
        // SAFETY: `info` is a valid place for the call to fill.
        unsafe { GetIconInfo(icon, &mut info) }.ok()?;
        let pixels = pixels(info.hbmColor);
        // SAFETY: `GetIconInfo` gave us both bitmaps to delete. A mask-only icon has no colour one.
        unsafe {
            let _ = DeleteObject(info.hbmColor.into());
            let _ = DeleteObject(info.hbmMask.into());
        }
        pixels
    }

    fn pixels(bitmap: HBITMAP) -> Option<Icon> {
        let mut size = BITMAP::default();
        // SAFETY: `size` is the size passed with it.
        let found = unsafe {
            GetObjectW(
                bitmap.into(),
                size_of::<BITMAP>() as i32,
                Some(&raw mut size as *mut c_void),
            )
        };
        if found == 0 {
            return None;
        }
        let (width, height) = (size.bmWidth, size.bmHeight);
        let mut frame = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                // Negative: rows run top to bottom.
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut rgba = vec![0; width as usize * height as usize * 4];
        // SAFETY: `rgba` holds the `height` rows of `width` pixels the call writes.
        let rows = unsafe {
            let screen = GetDC(None);
            let rows = GetDIBits(
                screen,
                bitmap,
                0,
                height as u32,
                Some(rgba.as_mut_ptr().cast()),
                &mut frame,
                DIB_RGB_COLORS,
            );
            ReleaseDC(None, screen);
            rows
        };
        if rows == 0 {
            return None;
        }
        let (pixels, _) = rgba.as_chunks_mut::<4>();
        for pixel in pixels.iter_mut() {
            pixel.swap(0, 2);
        }
        // An icon drawn before alpha existed has none; its mask is not read.
        if pixels.iter().all(|pixel| pixel[3] == 0) {
            pixels.iter_mut().for_each(|pixel| pixel[3] = 255);
        }
        Some(Icon { width: width as u32, height: height as u32, rgba })
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::of;

    #[test]
    fn a_system_executable_has_an_icon_of_the_size_it_states() {
        let icon = of(r"C:\Windows\System32\notepad.exe").expect("notepad has an icon");
        assert!(icon.width > 0 && icon.height > 0);
        assert_eq!(icon.rgba.len(), icon.width as usize * icon.height as usize * 4);
        assert!(icon.rgba.as_chunks::<4>().0.iter().any(|pixel| pixel[3] != 0));
    }

    #[test]
    fn a_missing_file_has_no_icon() {
        assert!(of(r"C:\no\such\file.exe").is_none());
    }
}
