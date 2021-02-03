// most of this code is taken wholesale from this:
// https://github.com/rust-lang/rustup/blob/master/src/cli/self_update/windows.rs
use crate::{Installer, InstallerError};

use std::io;

/// Adds the downloaded binary in Installer to a Windows PATH
pub fn add_binary_to_path(installer: &Installer) -> Result<(), InstallerError> {
    let windows_path = get_windows_path_var()?;
    let bin_path = installer
        .get_bin_dir_path()?
        .to_str()
        .ok_or(InstallerError::PathNotUnicode)?
        .to_string();
    if let Some(old_path) = windows_path {
        if let Some(new_path) = add_to_path(&old_path, &bin_path) {
            apply_new_path(&new_path)?;
        }
    }

    Ok(())
}

// ---------------------------------------------
// https://en.wikipedia.org/wiki/Here_be_dragons
// ---------------------------------------------
// nothing is sacred beyond this point.

// Get the windows PATH variable out of the registry as a String. If
// this returns None then the PATH variable is not unicode and we
// should not mess with it.
fn get_windows_path_var() -> Result<Option<String>, InstallerError> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::RegKey;

    let root = RegKey::predef(HKEY_CURRENT_USER);
    let environment = root.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;

    let reg_value = environment.get_raw_value("PATH");
    match reg_value {
        Ok(val) => {
            if let Some(s) = string_from_winreg_value(&val) {
                Ok(Some(s))
            } else {
                log::warn!("the registry key HKEY_CURRENT_USER\\Environment\\PATH does not contain valid Unicode. \
                       Not modifying the PATH variable");
                Ok(None)
            }
        }
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(Some(String::new())),
        Err(e) => Err(e)?,
    }
}

// This is used to decode the value of HKCU\Environment\PATH. If that
// key is not unicode (or not REG_SZ | REG_EXPAND_SZ) then this
// returns None.  The winreg library itself does a lossy unicode
// conversion.
fn string_from_winreg_value(val: &winreg::RegValue) -> Option<String> {
    use std::slice;
    use winreg::enums::RegType;

    match val.vtype {
        RegType::REG_SZ | RegType::REG_EXPAND_SZ => {
            // Copied from winreg
            let words = unsafe {
                #[allow(clippy::cast_ptr_alignment)]
                slice::from_raw_parts(val.bytes.as_ptr().cast::<u16>(), val.bytes.len() / 2)
            };
            String::from_utf16(words).ok().map(|mut s| {
                while s.ends_with('\u{0}') {
                    s.pop();
                }
                s
            })
        }
        _ => None,
    }
}

// Returns None if the existing old_path does not need changing, otherwise
// prepends the path_str to old_path, handling empty old_path appropriately.
fn add_to_path(old_path: &str, path_str: &str) -> Option<String> {
    if old_path.is_empty() {
        Some(path_str.to_string())
    } else if old_path.contains(path_str) {
        None
    } else {
        let mut new_path = path_str.to_string();
        new_path.push_str(";");
        new_path.push_str(&old_path);
        Some(new_path)
    }
}

fn apply_new_path(new_path: &str) -> Result<(), InstallerError> {
    use std::ptr;
    use winapi::shared::minwindef::*;
    use winapi::um::winuser::{
        SendMessageTimeoutA, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
    };
    use winreg::enums::{RegType, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::{RegKey, RegValue};

    let root = RegKey::predef(HKEY_CURRENT_USER);
    let environment = root.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;

    if new_path.is_empty() {
        environment.delete_value("PATH")?;
    } else {
        let reg_value = RegValue {
            bytes: string_to_winreg_bytes(new_path),
            vtype: RegType::REG_EXPAND_SZ,
        };
        environment.set_raw_value("PATH", &reg_value)?;
    }

    // Tell other processes to update their environment
    unsafe {
        SendMessageTimeoutA(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0 as WPARAM,
            "Environment\0".as_ptr() as LPARAM,
            SMTO_ABORTIFHUNG,
            5000,
            ptr::null_mut(),
        );
    }

    Ok(())
}

fn string_to_winreg_bytes(s: &str) -> Vec<u8> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    let v: Vec<u16> = OsStr::new(s).encode_wide().chain(Some(0)).collect();
    unsafe { std::slice::from_raw_parts(v.as_ptr().cast::<u8>(), v.len() * 2).to_vec() }
}

#[cfg(test)]
mod tests {
    #[test]
    fn windows_install_does_not_add_path_twice() {
        assert_eq!(
            None,
            super::add_to_path(
                r"c:\users\example\.mybinary\bin;foo",
                r"c:\users\example\.mybinary\bin"
            )
        );
    }

    #[test]
    fn windows_install_does_add_path() {
        assert_eq!(
            Some(r"c:\users\example\.mybinary\bin;foo".to_string()),
            super::add_to_path("foo", r"c:\users\example\.mybinary\bin")
        );
    }

    #[test]
    fn windows_install_does_add_path_no_double_semicolon() {
        assert_eq!(
            Some(r"c:\users\example\.mybinary\bin;foo;bar;".to_string()),
            super::add_to_path("foo;bar;", r"c:\users\example\.mybinary\bin")
        );
    }
}
