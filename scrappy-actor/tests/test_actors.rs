use scrappy_actor::actors::resolver;
use scrappy_actor::prelude::*;

#[scrappy_rt::test]
async fn test_resolver() {
    Arbiter::spawn(async {
        let resolver = resolver::Resolver::from_registry();
        let _ = resolver.send(resolver::Resolve::host("localhost")).await;
        System::current().stop();
    });

    Arbiter::spawn(async {
        let resolver = resolver::Resolver::from_registry();
        let _ = resolver
            .send(resolver::Connect::host("localhost:5000"))
            .await;
    });
}
