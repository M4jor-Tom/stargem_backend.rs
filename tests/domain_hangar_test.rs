use stargem_server::domain::*;

#[test]
fn test_hangar_add_ship() {
    let mut hangar = Hangar::new(uuid::Uuid::new_v4());
    let ship_id = uuid::Uuid::new_v4();

    assert!(hangar.add_ship(ship_id).is_ok());
    assert_eq!(hangar.ship_ids.len(), 1);
}

#[test]
fn test_hangar_add_ship_already_exists() {
    let mut hangar = Hangar::new(uuid::Uuid::new_v4());
    let ship_id = uuid::Uuid::new_v4();

    hangar.add_ship(ship_id).unwrap();
    let result = hangar.add_ship(ship_id);

    assert!(result.is_err());
    assert_eq!(hangar.ship_ids.len(), 1);
}

#[test]
fn test_hangar_max_capacity() {
    let mut hangar = Hangar::new(uuid::Uuid::new_v4());

    for _ in 0..MAX_HANGAR_SIZE {
        let ship_id = uuid::Uuid::new_v4();
        assert!(hangar.add_ship(ship_id).is_ok());
    }

    let extra_ship = uuid::Uuid::new_v4();
    assert!(hangar.add_ship(extra_ship).is_err());
}

#[test]
fn test_hangar_remove_ship() {
    let mut hangar = Hangar::new(uuid::Uuid::new_v4());
    let ship_id = uuid::Uuid::new_v4();

    hangar.add_ship(ship_id).unwrap();
    assert!(hangar.remove_ship(ship_id).is_ok());
    assert!(hangar.ship_ids.is_empty());
}

#[test]
fn test_hangar_remove_ship_not_found() {
    let mut hangar = Hangar::new(uuid::Uuid::new_v4());
    let ship_id = uuid::Uuid::new_v4();

    assert!(hangar.remove_ship(ship_id).is_err());
}

#[test]
fn test_hangar_select_ship() {
    let mut hangar = Hangar::new(uuid::Uuid::new_v4());
    let ship_id1 = uuid::Uuid::new_v4();
    let ship_id2 = uuid::Uuid::new_v4();

    hangar.add_ship(ship_id1).unwrap();
    hangar.add_ship(ship_id2).unwrap();

    assert_eq!(hangar.select_ship(0).unwrap(), ship_id1);
    assert_eq!(hangar.select_ship(1).unwrap(), ship_id2);
    assert_eq!(hangar.selected_ship(), Some(ship_id2));
}

#[test]
fn test_hangar_select_invalid_index() {
    let mut hangar = Hangar::new(uuid::Uuid::new_v4());

    assert!(hangar.select_ship(0).is_err());
}

#[test]
fn test_hangar_select_ship_updates_on_remove() {
    let mut hangar = Hangar::new(uuid::Uuid::new_v4());
    let ship_id1 = uuid::Uuid::new_v4();
    let ship_id2 = uuid::Uuid::new_v4();

    hangar.add_ship(ship_id1).unwrap();
    hangar.add_ship(ship_id2).unwrap();
    hangar.select_ship(1).unwrap();

    hangar.remove_ship(ship_id2).unwrap();
    assert!(hangar.selected_ship_index.is_none());
}
