use cucumber::World;
use futures::FutureExt;
use std::sync::Arc;
use tokio::sync::Mutex;

mod state;
mod steps;
mod utils;

use zero2prod::server::State;

// This runs before everything else, so you can setup things here.
#[tokio::main]
async fn main() {
    state::TestWorld::cucumber()
        .before(move |_feature, _rule, _scenario, world| 
                async {
                    let exec = world.pool.begin().await.expect("Unable to begin transaction");
                    let exec = Arc::new(Mutex::new(State { exec }));
                    world.exec = Some(exec);
                }.boxed()
               )
        .after(move |_feature, _rule, _scenario, _event, world|
               async {
                   let exec = world.unwrap().exec.take().expect("take transaction");
                   println!("Arc count: {}", Arc::strong_count(&exec));
                   let exec = Arc::into_inner(exec).expect("try unwrap").into_inner().exec;
                   exec.rollback().await.expect("Unable to rollback transaction");
               }
               .boxed()
               )
        .run("tests/features")
        .await;
}
