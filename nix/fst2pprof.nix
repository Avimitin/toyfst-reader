{ lib
, rustPlatform
, protobuf

  # dev deps
, rust-bin
, rust-analyzer-unwrapped
}:
let
  self = rustPlatform.buildRustPackage
    {
      pname = "fst2pprof";
      version = "v0.1.0";

      src = lib.cleanSource ../.;

      cargoLock = {
        lockFile = ../Cargo.lock;
        allowBuiltinFetchGit = true;
      };

      nativeBuildInputs = [ protobuf ];

      passthru =
        let
          rust-toolchain = rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" ];
          };
        in
        {
          dev = self.overrideAttrs (old: {
            nativeBuildInputs = [
              # add rust-analyzer support for dev shell
              rust-analyzer-unwrapped
              rust-toolchain
            ] ++ old.nativeBuildInputs;
            env = {
              # To make rust-analyzer work correctly (The path prefix issue)
              RUST_SRC_PATH = "${rust-toolchain}/lib/rustlib/src/rust/library";
            };
          });
        };

      meta = with lib; {
        description = "A converter for trasforming gtkwave fst file to pprof data file.";
        license = licenses.mit;
      };
    };
in
self
