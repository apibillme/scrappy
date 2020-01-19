use std::io;

use scrappy_codec::{BytesCodec, Framed};
use scrappy_rt::net::TcpStream;
use scrappy_service::{fn_service, Service, ServiceFactory};
use scrappy_testing::TestServer;
use bytes::Bytes;
use futures::SinkExt;

use scrappy_connect::resolver::{ResolverConfig, ResolverOpts};
use scrappy_connect::Connect;

#[cfg(feature = "openssl")]
#[scrappy_rt::test]
async fn test_string() {
    let srv = TestServer::with(|| {
        fn_service(|io: TcpStream| {
            async {
                let mut framed = Framed::new(io, BytesCodec);
                framed.send(Bytes::from_static(b"test")).await?;
                Ok::<_, io::Error>(())
            }
        })
    });

    let mut conn = scrappy_connect::default_connector();
    let addr = format!("localhost:{}", srv.port());
    let con = conn.call(addr.into()).await.unwrap();
    assert_eq!(con.peer_addr().unwrap(), srv.addr());
}

#[cfg(feature = "rustls")]
#[scrappy_rt::test]
async fn test_rustls_string() {
    let srv = TestServer::with(|| {
        fn_service(|io: TcpStream| {
            async {
                let mut framed = Framed::new(io, BytesCodec);
                framed.send(Bytes::from_static(b"test")).await?;
                Ok::<_, io::Error>(())
            }
        })
    });

    let mut conn = scrappy_connect::default_connector();
    let addr = format!("localhost:{}", srv.port());
    let con = conn.call(addr.into()).await.unwrap();
    assert_eq!(con.peer_addr().unwrap(), srv.addr());
}

#[scrappy_rt::test]
async fn test_static_str() {
    let srv = TestServer::with(|| {
        fn_service(|io: TcpStream| {
            async {
                let mut framed = Framed::new(io, BytesCodec);
                framed.send(Bytes::from_static(b"test")).await?;
                Ok::<_, io::Error>(())
            }
        })
    });

    let resolver = scrappy_connect::start_default_resolver();
    let mut conn = scrappy_connect::new_connector(resolver.clone());

    let con = conn.call(Connect::with("10", srv.addr())).await.unwrap();
    assert_eq!(con.peer_addr().unwrap(), srv.addr());

    let connect = Connect::new(srv.host().to_owned());
    let mut conn = scrappy_connect::new_connector(resolver);
    let con = conn.call(connect).await;
    assert!(con.is_err());
}

#[scrappy_rt::test]
async fn test_new_service() {
    let srv = TestServer::with(|| {
        fn_service(|io: TcpStream| {
            async {
                let mut framed = Framed::new(io, BytesCodec);
                framed.send(Bytes::from_static(b"test")).await?;
                Ok::<_, io::Error>(())
            }
        })
    });

    let resolver =
        scrappy_connect::start_resolver(ResolverConfig::default(), ResolverOpts::default());

    let factory = scrappy_connect::new_connector_factory(resolver);

    let mut conn = factory.new_service(()).await.unwrap();
    let con = conn.call(Connect::with("10", srv.addr())).await.unwrap();
    assert_eq!(con.peer_addr().unwrap(), srv.addr());
}

#[cfg(feature = "openssl")]
#[scrappy_rt::test]
async fn test_uri() {
    use std::convert::TryFrom;

    let srv = TestServer::with(|| {
        fn_service(|io: TcpStream| {
            async {
                let mut framed = Framed::new(io, BytesCodec);
                framed.send(Bytes::from_static(b"test")).await?;
                Ok::<_, io::Error>(())
            }
        })
    });

    let mut conn = scrappy_connect::default_connector();
    let addr = http::Uri::try_from(format!("https://localhost:{}", srv.port())).unwrap();
    let con = conn.call(addr.into()).await.unwrap();
    assert_eq!(con.peer_addr().unwrap(), srv.addr());
}

#[cfg(feature = "rustls")]
#[scrappy_rt::test]
async fn test_rustls_uri() {
    use std::convert::TryFrom;

    let srv = TestServer::with(|| {
        fn_service(|io: TcpStream| {
            async {
                let mut framed = Framed::new(io, BytesCodec);
                framed.send(Bytes::from_static(b"test")).await?;
                Ok::<_, io::Error>(())
            }
        })
    });

    let mut conn = scrappy_connect::default_connector();
    let addr = http::Uri::try_from(format!("https://localhost:{}", srv.port())).unwrap();
    let con = conn.call(addr.into()).await.unwrap();
    assert_eq!(con.peer_addr().unwrap(), srv.addr());
}
