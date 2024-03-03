with import <nixpkgs> {};
mkShell {
  buildInputs = [
    pkg-config

    # dependencies you want available in your shell
  ];
}
