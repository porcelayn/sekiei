use std::path::Path;

pub async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let dist = Path::new("dist");
    if !dist.exists() {
        return Err("dist directory does not exist. Please run 'sekiei build' first.".into());
    }

    let routes = warp::fs::dir(dist);
    println!("Serving on http://localhost:8000");
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
    Ok(())
}