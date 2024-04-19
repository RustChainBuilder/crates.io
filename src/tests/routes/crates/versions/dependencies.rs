use crate::builders::{CrateBuilder, VersionBuilder};
use crate::util::{RequestHelper, TestApp};
use crates_io::views::EncodableDependency;
use http::StatusCode;

#[derive(Deserialize)]
pub struct Deps {
    pub dependencies: Vec<EncodableDependency>,
}

#[tokio::test(flavor = "multi_thread")]
async fn dependencies() {
    let (app, anon, user) = TestApp::init().with_user();
    let user = user.as_model();

    app.db(|conn| {
        let c1 = CrateBuilder::new("foo_deps", user.id).expect_build(conn);
        let c2 = CrateBuilder::new("bar_deps", user.id).expect_build(conn);
        VersionBuilder::new("1.0.0")
            .dependency(&c2, None)
            .expect_build(c1.id, user.id, conn);
    });

    let deps: Deps = anon
        .async_get("/api/v1/crates/foo_deps/1.0.0/dependencies")
        .await
        .good();
    assert_eq!(deps.dependencies[0].crate_id, "bar_deps");

    let response = anon
        .async_get::<()>("/api/v1/crates/missing-crate/1.0.0/dependencies")
        .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        response.json(),
        json!({ "errors": [{ "detail": "crate `missing-crate` does not exist" }] })
    );

    let response = anon
        .async_get::<()>("/api/v1/crates/foo_deps/1.0.2/dependencies")
        .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        response.json(),
        json!({ "errors": [{ "detail": "crate `foo_deps` does not have a version `1.0.2`" }] })
    );
}
