use std::io::Result;

fn discover_protos() -> Vec<String> {
    let mut protos = Vec::new();
    for entry in std::fs::read_dir("proto").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            protos.push(path.to_str().unwrap().to_string());
        }
    }
    protos
}

fn main() -> Result<()> {
    let protos = discover_protos();
    protos.iter().for_each(|proto| {
        tonic_build::compile_protos(proto).unwrap();
    });
    Ok(())
}
