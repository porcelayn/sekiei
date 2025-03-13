use crate::build;
use std::path::Path;
use colored::Colorize;

pub async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let dist = Path::new("dist");
    build::build().unwrap();
    let routes = warp::fs::dir(dist);
    println!("{}", "Starting server at 8000".on_blue());
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
    Ok(())
}
