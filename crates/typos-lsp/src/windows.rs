#[cfg(windows)]
extern "system" {
    // https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getlogicaldrives
    pub fn GetLogicalDrives() -> u32;
}

#[cfg(windows)]
pub fn get_drives() -> Vec<String> {
    let mut drives = Vec::new();
    let mut bitmask = unsafe { GetLogicalDrives() };
    let mut letter = b'A';
    while bitmask > 0 {
        if bitmask & 1 != 0 {
            drives.push((letter as char).into());
        }
        bitmask >>= 1;
        letter += 1;
    }
    drives
}
