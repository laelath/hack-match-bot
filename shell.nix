with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "rust-env";
  nativeBuildInputs = [
    rustPackages.rustc
    rustPackages.cargo
    rustPackages.rustfmt
    rustPackages.clippy
    rustPackages.rls
    rustracer
    pkgconfig
  ];
  buildInputs = [
    xlibs.libX11
    xlibs.libXtst
  ];

  # Set Environment Variables
  RUST_BACKTRACE = 1;
}
