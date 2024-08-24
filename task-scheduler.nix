{
  lib,
  rustPlatform,
}:
rustPlatform.buildRustPackage {
  pname = "task-scheduler";
  version = "0.1.0";
  src = ./.;
  cargoHash = "sha256-NFWLL1hgYs88KP1cMHrimlnwbZFjdrzKBtXzLKd/iRY=";
  meta = {
    description = "A task scheduler in Rust";
    license = lib.licenses.mit;
  };
}
