use hamsrs::ProbeManual;
use hamsrs::probes::ProbeKick;
use std::thread;
use std::time::Duration;

#[test]
fn test_manual_probe_send_sync() {
    let probe = ProbeManual::new("test_manual_send_sync", true).unwrap();

    // Check if we can move it to another thread
    let probe_clone = probe.clone();
    let handle = thread::spawn(move || {
        probe_clone.toggle().unwrap();
    });

    handle.join().unwrap();

    // Check if toggle worked (it should be false now)
    assert_eq!(probe.check().unwrap(), false);
}

#[test]
fn test_kick_probe_send_sync() {
    let probe = ProbeKick::new("test_kick_send_sync", Duration::from_secs(1)).unwrap();

    // Check if we can move it to another thread
    let probe_clone = probe.clone();
    let handle = thread::spawn(move || {
        probe_clone.kick().unwrap();
    });

    handle.join().unwrap();

    // Check if it is still healthy
    // assert_eq!(probe.check().unwrap(), true);
}
