#[macro_use]
extern crate servex;

#[import("example.model")]
struct App;

#[tokio::main()]
async fn main() {
    let x = Model::Foo {
        id: 9,
        bla: Some("bloo".into()),
    };
    dbg!(&x);

    async fn injector() -> impl axum::response::IntoResponse {
        (axum::http::StatusCode::OK, "injected")
    }

    let mut app = App::init(Opt {
        ..Default::default()
    }).await;
    app.http = app.http.map(|mut http| {
        http.router = http.router.route("/injector", axum::routing::get(injector));
        http
    });
    app.boot().await;
}
