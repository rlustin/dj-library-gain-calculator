use std::assert;
use walkdir::WalkDir;
use dj_library_gain_calculator::analysis::scan_loudness;

#[test]
fn loudness_measurement_smoketest() -> Result<(), Box<dyn std::error::Error>> {
        for entry in WalkDir::new(".")
            .into_iter()
            .filter_map(|e| e.ok()) {
        let path = entry.path().to_string_lossy();
        let name = entry.file_name().to_string_lossy();

        if name.starts_with("sine-440") {
            match scan_loudness(&path) {
                Ok(computed_loudness) => {
                    let integrated = computed_loudness.integrated_loudness;
                    let peak = computed_loudness.true_peak;
                    // Account for variations in decoder output.
                    assert!(integrated < -3.5 && integrated > -4.2);
                    assert!(peak > 0.67 && peak < 0.73);
                }
                Err(e) => {
                    eprintln!("{}", e);
                    assert!(false);
                }
            }
        }
    }

    Ok(())
}
