{ lib, rustPlatform }:

rustPlatform.buildRustPackage {
  pname = "pw";
  version = (lib.importTOML ../Cargo.toml).package.version;

  src = lib.cleanSource ./..;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  meta = {
    description = "Encrypted password storage";
    mainProgram = "pw";
  };
}
