#[test]
#[ignore = "requires PODBOX_E2E=1 and a running microVM guest agent"]
fn host_guest_channel_exec_round_trip_placeholder() {
    let _enabled = std::env::var("PODBOX_E2E").as_deref() == Ok("1");
}
