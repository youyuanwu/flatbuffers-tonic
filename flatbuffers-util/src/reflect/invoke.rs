use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::ownedfb::OwnedFB;

fn get_temp_subdir_name(prefix: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let pid = std::process::id();
    format!("{prefix}_{timestamp}_{pid}")
}

fn get_flatbuffers_util_temp_dir() -> PathBuf {
    let temp_dir = std::env::temp_dir().join("flatbuffers_util_bfbs");
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
    temp_dir
}

/// Use temp dir $TEMP/flatbuffers_util_bfbs/<file_stem>_<timestamp>_<pid>/<file_stem>.bfbs
/// to store the generated bfbs file, then read it back into an OwnedFB<Schema>
pub fn compile_reflection_schema(
    fbs_path: &Path,
) -> crate::ownedfb::OwnedFB<flatbuffers_reflection::reflection::Schema<'static>> {
    let flatc_path = ensure_flatc();
    // out file has a different extension
    let fbs_file_name = fbs_path
        .file_stem()
        .expect("Failed to get file stem")
        .to_str()
        .expect("Failed to convert OsStr to str")
        .to_owned();
    let temp_subdir = get_temp_subdir_name(&fbs_file_name);
    let temp_full_dir = get_flatbuffers_util_temp_dir().join(temp_subdir);
    fs::create_dir_all(&temp_full_dir).expect("Failed to create temp dir");

    let status = std::process::Command::new(flatc_path)
        .args([
            "--binary",
            "--schema",
            "-o",
            temp_full_dir.to_str().unwrap(),
            fbs_path.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to execute flatc for reflection schema generation");
    assert!(
        status.success(),
        "flatc command for reflection schema generation failed"
    );

    let schema_file_path = temp_full_dir.join(format!("{fbs_file_name}.bfbs"));
    assert!(schema_file_path.exists());

    // Read the temp file
    let schema_data = fs::read(&schema_file_path).expect("Failed to read schema file");
    let schema =
        OwnedFB::new_boxed(schema_data.into_boxed_slice()).expect("Failed to create OwnedFB");

    // remove temp file
    let _ = fs::remove_dir_all(temp_full_dir);
    schema
}

/// Returns the flatc that can be executed
pub fn ensure_flatc() -> String {
    // execute `flatc --version` to ensure flatc is available
    let status = std::process::Command::new("flatc")
        .arg("--version")
        .status();
    if status.is_ok() && status.unwrap().success() {
        return "flatc".to_string();
    }

    // read the env var FLATC_PATH
    let flatc_path =
        std::env::var("FLATC_PATH").expect("FLATC_PATH environment variable is not set");
    let flatc_path = PathBuf::from(flatc_path);
    assert!(flatc_path.exists(), "FLATC_PATH does not exist");
    // execute the flatc at the path
    let status = std::process::Command::new(&flatc_path)
        .arg("--version")
        .status();
    assert!(status.is_ok(), "Failed to execute flatc at FLATC_PATH");
    flatc_path.to_str().unwrap().to_string()
}
