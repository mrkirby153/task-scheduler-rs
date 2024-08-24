{
  lib,
  rustPlatform,
  protobuf,
}:
rustPlatform.buildRustPackage {
  pname = "task-scheduler";
  version = "0.1.0";
  src = ./.;
  cargoHash = "sha256-HhDIqzso7kqI9Fm3UBFz48vejgl3T0qLXXOJkh6y8wI=";
  meta = {
    description = "A task scheduler in Rust";
    license = lib.licenses.mit;
  };
  nativeBuildInputs = [protobuf];
}
