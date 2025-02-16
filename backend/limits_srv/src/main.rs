use std::sync::atomic::{Ordering, AtomicU64};
use std::sync::{RwLock, Arc};

use warp::Filter;

use limits_core::json::Base;

static VISIT_COUNT: AtomicU64 = AtomicU64::new(0);

fn get_limits(base: Base) -> impl warp::Reply {
    VISIT_COUNT.fetch_add(1, Ordering::AcqRel);
    println!("Limits got");
    warp::reply::json(&base)
}

fn get_visits() -> impl warp::Reply {
    let count = VISIT_COUNT.load(Ordering::Relaxed);
    println!("Count got");
    warp::reply::json(&count)
}

fn routes(base: Arc<RwLock<Base>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get().and(
        warp::path!("powertools" / "v1")
            .map(move || {
                let base = base.read().expect("Failed to acquire base limits read lock").clone();
                get_limits(base)
            })
        .or(
            warp::path!("powertools" / "count")
                .map(get_visits)
        )
    ).recover(recovery)
}

pub async fn recovery(reject: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    if reject.is_not_found() {
        Ok(warp::hyper::StatusCode::NOT_FOUND)
    } else {
        Err(reject)
    }
}

#[tokio::main]
async fn main() {
    let file = std::fs::File::open("./pt_limits.json").expect("Failed to read limits file");
    let limits: Base = serde_json::from_reader(file).expect("Failed to parse limits file");
    assert!(limits.refresh.is_some(), "`refresh` cannot be null, since it will brick future refreshes");

    warp::serve(routes(Arc::new(RwLock::new(limits))))
        .run(([0, 0, 0, 0], 8080))
        .await;
}
