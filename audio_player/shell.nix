{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  packages = with pkgs; [
    rustc
    cargo
    gcc
    pkg-config
    alsa-lib
    pulseaudio
  ];
}
