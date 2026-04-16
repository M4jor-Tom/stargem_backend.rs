use stargem_server::domain::GameMode;
use stargem_server::game::{Matchmaker, MatchmakingTicket};
use uuid::Uuid;

#[test]
fn test_matchmaker_empty_queue() {
    let mut matchmaker = Matchmaker::new();
    assert!(matchmaker.try_match().is_none());
    assert_eq!(matchmaker.queue_size(), 0);
}

#[test]
fn test_matchmaker_add_to_queue() {
    let mut matchmaker = Matchmaker::new();
    let player_id = Uuid::new_v4();

    matchmaker.enter_queue(player_id, GameMode::TeamDeathmatch);

    assert_eq!(matchmaker.queue_size(), 1);
    assert!(matchmaker.is_in_queue(player_id));
}

#[test]
fn test_matchmaker_leave_queue() {
    let mut matchmaker = Matchmaker::new();
    let player_id = Uuid::new_v4();

    matchmaker.enter_queue(player_id, GameMode::TeamDeathmatch);
    matchmaker.leave_queue(player_id);

    assert_eq!(matchmaker.queue_size(), 0);
    assert!(!matchmaker.is_in_queue(player_id));
}

#[test]
fn test_matchmaker_no_duplicate() {
    let mut matchmaker = Matchmaker::new();
    let player_id = Uuid::new_v4();

    matchmaker.enter_queue(player_id, GameMode::TeamDeathmatch);
    matchmaker.enter_queue(player_id, GameMode::TeamDeathmatch);

    assert_eq!(matchmaker.queue_size(), 1);
}

#[test]
fn test_matchmaker_insufficient_players() {
    let mut matchmaker = Matchmaker::new();

    matchmaker.enter_queue(Uuid::new_v4(), GameMode::TeamDeathmatch);

    assert!(matchmaker.try_match().is_none());
    assert_eq!(matchmaker.queue_size(), 1);
}

#[test]
fn test_matchmaker_successful_match() {
    let mut matchmaker = Matchmaker::new();

    for _ in 0..4 {
        matchmaker.enter_queue(Uuid::new_v4(), GameMode::TeamDeathmatch);
    }

    let matched = matchmaker.try_match();
    assert!(matched.is_some());
    assert!(matched.unwrap().len() >= 2);
    assert!(matchmaker.queue_size() <= 2);
}

#[test]
fn test_matchmaker_max_players() {
    let mut matchmaker = Matchmaker::new();

    for _ in 0..20 {
        matchmaker.enter_queue(Uuid::new_v4(), GameMode::TeamDeathmatch);
    }

    let matched = matchmaker.try_match();
    assert!(matched.is_some());
    assert!(matched.unwrap().len() <= 16);
}
