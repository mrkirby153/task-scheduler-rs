{
  lib,
  rustPlatform,
  protobuf,
}:
rustPlatform.buildRustPackage {
  pname = "task-scheduler";
  version = "0.1.0";
  src = ./.;
  cargoHash = "sha256-ftsntSo78Om9y2czOGPdJh9+nPF0DaFxfnE4bjcva+w=";
  meta = {
    description = "A task scheduler in Rust";
    license = lib.licenses.mit;
  };
  nativeBuildInputs = [protobuf];
}
