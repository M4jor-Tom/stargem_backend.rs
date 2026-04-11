use stargem_server::domain::*;

#[test]
fn test_game_mode_is_pvp() {
    assert!(GameMode::TeamDeathmatch.is_pvp());
    assert!(GameMode::FreeForAll.is_pvp());
    assert!(!GameMode::WavesSurvival.is_pvp());
    assert!(!GameMode::OpenWorld.is_pvp());
}

#[test]
fn test_game_mode_is_pve() {
    assert!(!GameMode::TeamDeathmatch.is_pve());
    assert!(GameMode::WavesSurvival.is_pve());
    assert!(GameMode::OperationScenario.is_pve());
    assert!(!GameMode::OpenWorld.is_pve());
}

#[test]
fn test_game_mode_allows_respawn() {
    assert!(GameMode::TeamDeathmatch.allows_respawn());
    assert!(GameMode::FreeForAll.allows_respawn());
    assert!(GameMode::WavesSurvival.allows_respawn());
    assert!(!GameMode::OperationScenario.allows_respawn());
    assert!(!GameMode::OpenWorld.allows_respawn());
}

#[test]
fn test_game_instance_create() {
    let instance = GameInstance::new(
        "Test Game".into(),
        GameMode::TeamDeathmatch,
        8,
    );
    
    assert_eq!(instance.name, "Test Game");
    assert_eq!(instance.mode, GameMode::TeamDeathmatch);
    assert_eq!(instance.max_players, 8);
    assert_eq!(instance.state, GameState::Lobby);
    assert!(instance.player_ids.is_empty());
}

#[test]
fn test_game_instance_add_player() {
    let mut instance = GameInstance::new("Test".into(), GameMode::TeamDeathmatch, 4);
    let player_id = uuid::Uuid::new_v4();
    
    assert!(instance.add_player(player_id).is_ok());
    assert_eq!(instance.player_ids.len(), 1);
}

#[test]
fn test_game_instance_add_player_full() {
    let mut instance = GameInstance::new("Test".into(), GameMode::TeamDeathmatch, 2);
    
    instance.add_player(uuid::Uuid::new_v4()).unwrap();
    instance.add_player(uuid::Uuid::new_v4()).unwrap();
    
    assert!(instance.add_player(uuid::Uuid::new_v4()).is_err());
}

#[test]
fn test_game_instance_add_player_twice() {
    let mut instance = GameInstance::new("Test".into(), GameMode::TeamDeathmatch, 4);
    let player_id = uuid::Uuid::new_v4();
    
    instance.add_player(player_id).unwrap();
    assert!(instance.add_player(player_id).is_err());
}

#[test]
fn test_game_instance_start() {
    let mut instance = GameInstance::new("Test".into(), GameMode::TeamDeathmatch, 2);
    
    instance.add_player(uuid::Uuid::new_v4()).unwrap();
    instance.add_player(uuid::Uuid::new_v4()).unwrap();
    
    assert!(instance.start().is_ok());
    assert_eq!(instance.state, GameState::Starting);
    assert!(instance.started_at.is_some());
}

#[test]
fn test_game_instance_start_not_enough_players() {
    let mut instance = GameInstance::new("Test".into(), GameMode::TeamDeathmatch, 4);
    
    instance.add_player(uuid::Uuid::new_v4()).unwrap();
    assert!(instance.start().is_err());
}

#[test]
fn test_game_instance_team_scores() {
    let mut instance = GameInstance::new("Test".into(), GameMode::TeamDeathmatch, 4);
    
    instance.add_player(uuid::Uuid::new_v4()).unwrap();
    instance.add_player(uuid::Uuid::new_v4()).unwrap();
    instance.start().unwrap();
    
    assert!(instance.team_scores.is_some());
    let scores = instance.team_scores.unwrap();
    assert_eq!(scores.len(), 2);
    assert_eq!(scores[0], 0);
    assert_eq!(scores[1], 0);
}

#[test]
fn test_game_instance_update_score() {
    let mut instance = GameInstance::new("Test".into(), GameMode::TeamDeathmatch, 4);
    
    instance.add_player(uuid::Uuid::new_v4()).unwrap();
    instance.add_player(uuid::Uuid::new_v4()).unwrap();
    instance.start().unwrap();
    
    instance.update_score(0, 10).unwrap();
    
    assert_eq!(instance.team_scores.unwrap()[0], 10);
}

#[test]
fn test_game_instance_wave_number() {
    let mut instance = GameInstance::new("Test".into(), GameMode::WavesSurvival, 4);
    
    instance.add_player(uuid::Uuid::new_v4()).unwrap();
    instance.add_player(uuid::Uuid::new_v4()).unwrap();
    instance.start().unwrap();
    
    assert!(instance.wave_number.is_some());
    assert_eq!(instance.wave_number.unwrap(), 0);
    
    instance.advance_wave().unwrap();
    assert_eq!(instance.wave_number.unwrap(), 1);
}

#[test]
fn test_game_instance_end() {
    let mut instance = GameInstance::new("Test".into(), GameMode::TeamDeathmatch, 2);
    
    instance.add_player(uuid::Uuid::new_v4()).unwrap();
    instance.add_player(uuid::Uuid::new_v4()).unwrap();
    instance.start().unwrap();
    
    assert!(instance.end().is_ok());
    assert_eq!(instance.state, GameState::Ended);
    assert!(instance.ended_at.is_some());
}
