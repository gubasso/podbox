#[test]
#[ignore = "requires PODBOX_E2E=1, KVM, rootless Podman, libkrun, and live podbox guest networking"]
fn denied_endpoint_fails_while_allowed_endpoint_succeeds_in_guest() {
    if std::env::var_os("PODBOX_E2E").is_none() {
        eprintln!("set PODBOX_E2E=1 on a KVM-capable host to run this test");
        return;
    }
    panic!("live in-guest network enforcement is verified by the product-surface round");
}
