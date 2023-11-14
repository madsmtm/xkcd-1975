use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
struct Data {
    root: Root,
    graph: Graph,
}

fn main() -> std::io::Result<()> {
    let data: Data = serde_json::from_str(&std::fs::read_to_string("./data.json")?)?;
    // println!("{data:?}");
    Ok(())
}
