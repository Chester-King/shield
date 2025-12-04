use anyhow::Result;
use std::path::PathBuf;

/// Downloads Zcash Sapling proving parameters if not already present
///
/// Parameters will be downloaded to platform-specific directory:
/// - macOS: ~/.local/share/ZcashParams/
/// - Linux: ~/.local/share/ZcashParams/
/// - Windows: %LOCALAPPDATA%\ZcashParams\
///
/// Files:
/// - sapling-spend.params (~46MB)
/// - sapling-output.params (~3.4MB)
///
/// This only needs to be done once and files are cached permanently
pub fn ensure_params() -> Result<PathBuf> {
    println!("Checking Zcash proving parameters...");

    // Use zcash_proofs to download/verify parameters
    // It handles all the logic including checking if files already exist
    let paths = zcash_proofs::download_sapling_parameters(None)?;

    // Extract the parent directory from the returned paths
    let params_dir = paths.spend.parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid params path"))?
        .to_path_buf();

    println!("✓ Proving parameters ready!");
    println!("  Location: {}", params_dir.display());
    println!("  - Spend: {} ({:.1} MB)",
        paths.spend.display(),
        std::fs::metadata(&paths.spend)?.len() as f64 / 1_000_000.0
    );
    println!("  - Output: {} ({:.1} MB)",
        paths.output.display(),
        std::fs::metadata(&paths.output)?.len() as f64 / 1_000_000.0
    );

    Ok(params_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_params() {
        let result = ensure_params();
        assert!(result.is_ok(), "Failed to download parameters: {:?}", result);

        let params_dir = result.unwrap();

        // Verify the directory exists
        assert!(params_dir.exists(), "Params directory doesn't exist: {}", params_dir.display());

        // Verify the files exist
        let spend_params = params_dir.join("sapling-spend.params");
        let output_params = params_dir.join("sapling-output.params");

        assert!(spend_params.exists(), "Spend params not found: {}", spend_params.display());
        assert!(output_params.exists(), "Output params not found: {}", output_params.display());

        // Verify file sizes are reasonable (spend ~46MB, output ~3.4MB)
        let spend_size = std::fs::metadata(&spend_params).unwrap().len();
        let output_size = std::fs::metadata(&output_params).unwrap().len();

        assert!(spend_size > 40_000_000, "Spend params too small: {} bytes", spend_size);
        assert!(output_size > 3_000_000, "Output params too small: {} bytes", output_size);
    }
}
