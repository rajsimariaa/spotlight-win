use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;
use std::sync::OnceLock;

use super::{SearchResult, SearchResultCategory};

type EverythingSetSearchW = unsafe extern "system" fn(*const u16);
type EverythingQueryW = unsafe extern "system" fn(i32) -> i32;
type EverythingGetNumResults = unsafe extern "system" fn() -> u32;
type EverythingGetResultFilePathW = unsafe extern "system" fn(u32) -> *const u16;
type EverythingGetResultFileNameW = unsafe extern "system" fn(u32) -> *const u16;
type EverythingGetResultExtensionW = unsafe extern "system" fn(u32) -> *const u16;

struct EverythingFns {
    set_search_w: EverythingSetSearchW,
    query_w: EverythingQueryW,
    get_num_results: EverythingGetNumResults,
    get_result_file_path_w: EverythingGetResultFilePathW,
    get_result_file_name_w: EverythingGetResultFileNameW,
    get_result_extension_w: EverythingGetResultExtensionW,
}

static EVERYTHING_FNS: OnceLock<Option<EverythingFns>> = OnceLock::new();

fn load_everything_dll() -> Option<EverythingFns> {
    unsafe {
        let dll = windows::Win32::System::LibraryLoader::LoadLibraryW(
            windows::core::w!("Everything64.dll")
        ).ok()?;

        let set_search_w: EverythingSetSearchW = std::mem::transmute(
            windows::Win32::System::LibraryLoader::GetProcAddress(dll, windows::core::s!("Everything_SetSearchW"))?
        );
        let query_w: EverythingQueryW = std::mem::transmute(
            windows::Win32::System::LibraryLoader::GetProcAddress(dll, windows::core::s!("Everything_QueryW"))?
        );
        let get_num_results: EverythingGetNumResults = std::mem::transmute(
            windows::Win32::System::LibraryLoader::GetProcAddress(dll, windows::core::s!("Everything_GetNumResults"))?
        );
        let get_result_file_path_w: EverythingGetResultFilePathW = std::mem::transmute(
            windows::Win32::System::LibraryLoader::GetProcAddress(dll, windows::core::s!("Everything_GetResultFilePathW"))?
        );
        let get_result_file_name_w: EverythingGetResultFileNameW = std::mem::transmute(
            windows::Win32::System::LibraryLoader::GetProcAddress(dll, windows::core::s!("Everything_GetResultFileNameW"))?
        );
        let get_result_extension_w: EverythingGetResultExtensionW = std::mem::transmute(
            windows::Win32::System::LibraryLoader::GetProcAddress(dll, windows::core::s!("Everything_GetResultExtensionW"))?
        );

        Some(EverythingFns {
            set_search_w,
            query_w,
            get_num_results,
            get_result_file_path_w,
            get_result_file_name_w,
            get_result_extension_w,
        })
    }
}

fn get_everything_fns() -> Option<&'static EverythingFns> {
    let opt = EVERYTHING_FNS.get_or_init(|| load_everything_dll());
    opt.as_ref()
}

pub fn is_everything_available() -> bool {
    get_everything_fns().is_some()
}

pub fn query_files(search_term: &str, max_results: u32) -> Vec<SearchResult> {
    let fns = match get_everything_fns() {
        Some(f) => f,
        None => return fallback_file_search(search_term, max_results),
    };

    unsafe {
        // Set search to everything (all drives)
        let search_wide: Vec<u16> = OsStr::new(search_term)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        (fns.set_search_w)(search_wide.as_ptr());

        // query_w(1) = search all indexed volumes
        (fns.query_w)(1);

        let num_results = (fns.get_num_results)();
        let count = num_results.min(max_results);
        let mut results = Vec::with_capacity(count as usize);

        for i in 0..count {
            let path_ptr = (fns.get_result_file_path_w)(i);
            let name_ptr = (fns.get_result_file_name_w)(i);
            let ext_ptr = (fns.get_result_extension_w)(i);

            let path = wide_ptr_to_string(path_ptr);
            let name = wide_ptr_to_string(name_ptr);
            let ext = wide_ptr_to_string(ext_ptr);

            if !path.is_empty() {
                results.push(SearchResult {
                    id: path.clone(),
                    name: if name.is_empty() {
                        path.split('\\').last().unwrap_or(&path).to_string()
                    } else {
                        name
                    },
                    path: path.clone(),
                    category: SearchResultCategory::File,
                    icon: Some(ext),
                    score: 1.0 - (i as f64 * 0.005),
                    metadata: None,
                });
            }
        }

        results
    }
}

fn fallback_file_search(query: &str, max_results: u32) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    // Get all fixed drives
    let drives = get_all_drive_letters();

    // Common searchable directories on each drive
    let scan_dirs = ["", "Users", "Program Files", "Program Files (x86)"];

    for drive in &drives {
        for dir in &scan_dirs {
            let base = if dir.is_empty() {
                format!("{}\\", drive)
            } else {
                format!("{}\\{}", drive, dir)
            };

            if let Ok(entries) = std::fs::read_dir(&base) {
                for entry in entries.flatten().take(200) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let name_lower = name.to_lowercase();

                    if name_lower.contains(&query_lower) {
                        let path = entry.path().to_string_lossy().to_string();
                        let ext = entry.path()
                            .extension()
                            .map(|e| e.to_string_lossy().to_string())
                            .unwrap_or_default();

                        results.push(SearchResult {
                            id: path.clone(),
                            name: name,
                            path,
                            category: SearchResultCategory::File,
                            icon: Some(ext),
                            score: 0.5,
                            metadata: None,
                        });

                        if results.len() >= max_results as usize {
                            return results;
                        }
                    }
                }
            }
        }
    }

    results
}

fn get_all_drive_letters() -> Vec<String> {
    let mut drives = Vec::new();
    for i in 0..26 {
        let letter = (b'A' + i as u8) as char;
        let path = format!("{}:\\", letter);
        if std::path::Path::new(&path).exists() {
            drives.push(path);
        }
    }
    drives
}

unsafe fn wide_ptr_to_string(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }

    let mut len = 0;
    while *ptr.add(len) != 0 {
        len += 1;
    }

    let slice = std::slice::from_raw_parts(ptr, len);
    OsString::from_wide(slice)
        .to_string_lossy()
        .to_string()
}
