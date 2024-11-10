// Take a look at the license at the top of the repository in the LICENSE file.

#[test]
#[cfg(all(feature = "system", feature = "disk"))]
fn test_disks() {
    if sysinfo::IS_SUPPORTED_SYSTEM {
        let s = sysinfo::System::new_all();
        // If we don't have any physical core present, it's very likely that we're inside a VM...
        if s.physical_core_count().unwrap_or_default() > 0 {
            let mut disks = sysinfo::Disks::new();
            assert!(disks.list().is_empty());
            disks.refresh_list();
            assert!(!disks.list().is_empty());
        }
    }
}

#[test]
#[cfg(feature = "disk")]
fn test_disks_usage() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let s = sysinfo::System::new_all();

    // Skip the tests on unsupported platforms and on systems with no physical cores (likely a VM)
    if !sysinfo::IS_SUPPORTED_SYSTEM || s.physical_core_count().unwrap_or_default() == 0 {
        return;
    }

    // The test always fails in CI on Linux. For some unknown reason, /proc/diskstats just doesn't update, regardless
    // of how long we wait. Until the root cause is discovered, skip the test in CI
    if cfg!(target_os = "linux") && std::env::var("CI").is_ok() {
        return;
    }

    let mut disks = sysinfo::Disks::new_with_refreshed_list();

    let mut file = NamedTempFile::new().unwrap();

    // Write 10mb worth of data to the temp file.
    let data = vec![1u8; 10 * 1024 * 1024];
    file.write_all(&data).unwrap();
    // The sync_all call is important to ensure all the data is persisted to disk. Without
    // the call, this test is flaky.
    file.as_file().sync_all().unwrap();

    // Wait a bit just in case
    std::thread::sleep(std::time::Duration::from_millis(100));
    disks.refresh();

    // Depending on the OS and how disks are configured, the disk usage may be the exact same
    // across multiple disks. To account for this, collect the disk usages and dedup
    let mut disk_usages = disks.list().iter().map(|d| d.usage()).collect::<Vec<_>>();
    disk_usages.dedup();

    let mut written_bytes = 0;
    for disk_usage in disk_usages {
        written_bytes += disk_usage.written_bytes;
    }

    // written_bytes should have increased by about 10mb, but this is not fully reliable in CI Linux. For now,
    // just verify the number is non-zero.
    #[cfg(not(target_os = "freebsd"))]
    assert!(written_bytes > 0);
    // Disk usage is not yet supported on freebsd
    #[cfg(target_os = "freebsd")]
    assert_eq!(written_bytes, 0);
}