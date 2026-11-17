//! `PortalForcer.findClosestPortalPosition` may read a portal from a saved
//! chunk, but `getLargestRectangleAround` reads the level's blocks: a portal
//! whose chunk has not loaded must wait for it, not be read as air.

use super::*;
use crate::game::game_tests::tests::server_game;
use horizon_client::{
    chunk::Chunk,
    save::{LevelShape, SaveDir},
};
use std::sync::Arc;

#[test]
#[ignore = "daily"]
fn an_exit_portal_in_an_unloaded_chunk_waits_for_it() {
    let mut resources = None;
    let mut game = server_game(&mut resources);
    let dir = std::env::temp_dir().join(format!(
        "horizon-exit-search-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    game.save = Arc::new(SaveDir::new(dir.clone(), LevelShape::OVERWORLD));
    let portal = game
        .pack
        .assets
        .registry
        .default_state("minecraft:nether_portal")
        .unwrap();
    // A 2 by 3 portal in chunk (2, 0), saved, then unloaded.
    game.streaming
        .insert_remote(&mut game.world, Chunk::empty(2, 0));
    for width in 0..2 {
        for height in 0..3 {
            game.world.set(32 + width, 64 + height, 0, portal).unwrap();
        }
    }
    game.save_level(true).unwrap();
    let saved = game.world.chunks.remove(&(2, 0)).unwrap();
    assert!(!game.world.chunks.contains_key(&(2, 0)));
    let plan = Plan {
        dim: Dim::Overworld,
        approximate: BlockPos::new(32, 64, 0),
        entry_x_axis: true,
        offset: Vec3::new(0.5, 0.0, 0.0),
        width: 0.6,
        height: 1.8,
        end: false,
    };
    // The search reads the portal from the saved chunk, so it cannot read
    // its rectangle from the level yet.
    assert_eq!(
        game.find_exit_portal(&plan),
        ExitSearch::Unloaded(BlockPos::new(32, 64, 0))
    );
    // The exit waits for that chunk, with its portal ticket, and builds
    // nothing (`ensureLoadedAndValid`).
    assert_eq!(game.nether_exit_portal(&plan), Exit::Waiting);
    assert!(
        game.tickets.portal_tickets().iter().any(|t| t.chunk == (2, 0)),
        "the found portal's chunk has a ticket"
    );
    // Once its chunk is in the level the rectangle is the portal's, not a
    // sheet of air.
    game.world.chunks.insert((2, 0), saved);
    let ExitSearch::Found(found, rectangle) = game.find_exit_portal(&plan) else {
        panic!("the loaded portal is read");
    };
    assert_eq!(found, BlockPos::new(32, 64, 0));
    assert_eq!((rectangle.axis1_size, rectangle.axis2_size), (2, 3));
    let _ = std::fs::remove_dir_all(&dir);
}
