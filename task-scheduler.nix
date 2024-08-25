{
  lib,
  rustPlatform,
  protobuf,
}:
rustPlatform.buildRustPackage {
  pname = "task-scheduler";
  version = "0.1.0";
  src = ./.;
  cargoHash = "sha256-tKv5EJKp+IeZh1Wb51DWHpzPL4Lkm9KLaTkPGmoiuMY=";
  meta = {
    description = "A task scheduler in Rust";
    license = lib.licenses.mit;
  };
  nativeBuildInputs = [protobuf];
}
