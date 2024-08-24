{
  lib,
  rustPlatform,
  protobuf,
}:
rustPlatform.buildRustPackage {
  pname = "task-scheduler";
  version = "0.1.0";
  src = ./.;
  cargoHash = "sha256-KNw6P0yYnvACwvT6o2CShXpAbJ1hawIgUHsaS3x2XT4=";
  meta = {
    description = "A task scheduler in Rust";
    license = lib.licenses.mit;
  };
  nativeBuildInputs = [protobuf];
}
