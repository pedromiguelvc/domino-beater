{
  pkgs,
  ...
}:

{
  languages.rust = {
    enable = true;
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
    ];
  };

  packages = [
    pkgs.cargo-watch
    pkgs.cargo-nextest
    pkgs.claude-code
  ];

  scripts = {
    t.exec = "cargo test";
    play.exec = "cargo run --release";
  };

  git-hooks.hook = {
    rustfmt.enable = true;
    clippy.enable = true;
  };

  enterShell = ''
    echo "domino beater env ready"
    rustc --version
  '';
}
