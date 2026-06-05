//! Integration tests for the yrs-backed [`YrsBackend`].
//!
//! These tests assert the two properties expected by v009 :
//!
//! 1. Two backends concurrently editing **different** elements converge
//!    after exchanging snapshots, retaining both edits.
//! 2. A snapshot of a backend can be applied to a fresh backend to
//!    recover the original state byte-for-byte (at the element level).

use bse_crdt::{CrdtBackend, YrsBackend};
use bse_model::{Element, ElementKind, Style};
use bse_types::{ElementId, PeerId, Transform};

/// Build a rectangle [`Element`] with a fresh id and the given Z order.
fn make_rect(z: i32) -> Element {
    Element {
        id: ElementId::new(),
        kind: ElementKind::Rectangle {
            width: 10.0,
            height: 20.0,
        },
        style: Style::shape(),
        transform: Transform::IDENTITY,
        z,
        created_by: PeerId::new(),
        created_at: 0,
    }
}

#[test]
fn two_yrs_backends_converge_on_concurrent_edits() {
    // 1. Create two YrsBackend instances (Alice, Bob).
    //    Use deterministic client ids so any conflict resolution is reproducible.
    let mut alice = YrsBackend::with_client_id(1);
    let mut bob = YrsBackend::with_client_id(2);

    // 2. Both insert different elements locally.
    let alice_a = make_rect(1);
    let alice_b = make_rect(2);
    let bob_c = make_rect(3);
    let bob_d = make_rect(4);

    let id_a = alice_a.id;
    let id_b = alice_b.id;
    let id_c = bob_c.id;
    let id_d = bob_d.id;

    alice.upsert_element(alice_a.clone()).expect("alice a");
    alice.upsert_element(alice_b.clone()).expect("alice b");
    bob.upsert_element(bob_c.clone()).expect("bob c");
    bob.upsert_element(bob_d.clone()).expect("bob d");

    // 3. Exchange snapshots both ways (apply_remote_update).
    let snap_alice = alice.encode_snapshot().expect("encode alice");
    let snap_bob = bob.encode_snapshot().expect("encode bob");

    bob.apply_remote_update(&snap_alice)
        .expect("apply alice -> bob");
    alice
        .apply_remote_update(&snap_bob)
        .expect("apply bob -> alice");

    // 4. Assert both have the same element count.
    assert_eq!(alice.element_count(), 4, "alice should see 4 elements");
    assert_eq!(bob.element_count(), 4, "bob should see 4 elements");

    // 5. Assert both can fetch all elements by id, and the bodies match
    //    the originals.
    for (id, original) in [
        (id_a, &alice_a),
        (id_b, &alice_b),
        (id_c, &bob_c),
        (id_d, &bob_d),
    ] {
        let from_alice = alice.get_element(id).expect("alice has element");
        let from_bob = bob.get_element(id).expect("bob has element");
        assert_eq!(
            &from_alice, original,
            "alice's copy must match the original"
        );
        assert_eq!(&from_bob, original, "bob's copy must match the original");
        assert_eq!(from_alice, from_bob, "alice and bob must agree on contents");
    }
}

#[test]
fn snapshot_roundtrip_preserves_elements() {
    // Insert N elements into a YrsBackend.
    let mut source = YrsBackend::new();
    let mut originals = Vec::new();
    for z in 0..8 {
        let e = make_rect(z);
        originals.push(e.clone());
        source.upsert_element(e).expect("upsert");
    }
    assert_eq!(source.element_count(), originals.len());

    // Encode snapshot.
    let snapshot = source.encode_snapshot().expect("encode snapshot");
    assert!(!snapshot.is_empty(), "snapshot must contain bytes");

    // Create a fresh YrsBackend, apply the snapshot.
    let mut restored = YrsBackend::new();
    restored
        .apply_remote_update(&snapshot)
        .expect("apply snapshot");

    // Assert element counts and content match.
    assert_eq!(restored.element_count(), originals.len());
    for original in &originals {
        let fetched = restored
            .get_element(original.id)
            .expect("element survived the roundtrip");
        assert_eq!(&fetched, original);
    }
}

#[test]
fn concurrent_remove_after_sync_propagates() {
    let mut alice = YrsBackend::with_client_id(10);
    let mut bob = YrsBackend::with_client_id(20);

    let elem = make_rect(0);
    let id = elem.id;
    alice.upsert_element(elem).expect("upsert");

    // Sync alice's element to bob.
    let snap = alice.encode_snapshot().expect("snapshot");
    bob.apply_remote_update(&snap).expect("apply");
    assert_eq!(bob.element_count(), 1);

    // Bob removes the element ; alice catches up.
    bob.remove_element(id).expect("remove");
    let bob_snap = bob.encode_snapshot().expect("snapshot 2");
    alice.apply_remote_update(&bob_snap).expect("apply 2");

    assert_eq!(alice.element_count(), 0);
    assert_eq!(bob.element_count(), 0);
}
