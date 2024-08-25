{
  lib,
  rustPlatform,
  protobuf,
}:
rustPlatform.buildRustPackage {
  pname = "task-scheduler";
  version = "0.1.0";
  src = ./.;
  cargoHash = "sha256-lOOEeYHxmpUbkaSwL6mycnp8fv+qaD+secsUXMJQu4s=";
  meta = {
    description = "A task scheduler in Rust";
    license = lib.licenses.mit;
  };
  nativeBuildInputs = [protobuf];
}
