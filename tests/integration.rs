use cucumber::World;
// use cucumber::WriterExt;


mod state;
mod steps;
mod utils;

use steps::service::spawn_service;

// This runs before everything else, so you can setup things here.
#[tokio::main]
async fn main() {
    spawn_service().await;
    // Use the following if we need some debug output
    // state::TestWorld::cucumber()
    //     .max_concurrent_scenarios(1)
    //     .with_writer(
    //         cucumber::writer::Basic::raw(std::io::stdout(), cucumber::writer::Coloring::Never, 0)
    //             .summarized()
    //             .assert_normalized(),
    //     )
    //     .run_and_exit("tests/features")
    //     .await;
    //
    state::TestWorld::cucumber()
        .init_tracing()
        .run("tests/features")
        .await;
 
    // state::TestWorld::cucumber()
    //     .run("tests/features")
    //     .await;
}
