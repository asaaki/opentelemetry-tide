use tide::Request;

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let mut app = tide::new();
    app.at("/").get(|req: Request<()>| async move {
        // for unknown reasons this always returns None
        assert_eq!(req.version(), Some(http_types::Version::Http1_1));
        Ok("")
    });
    app.listen("127.0.0.1:8080").await?;

    Ok(())
}
