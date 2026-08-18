{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
    sqlx-cli
  ];


  buildInputs = with pkgs; [
    glib
    dbus
    gtk3
    webkitgtk_4_1
    libsoup_3
    openssl
  ];

  shellHook = ''
    export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath [
      pkgs.glib
      pkgs.dbus
      pkgs.gtk3
      pkgs.webkitgtk_4_1
      pkgs.libsoup_3
      pkgs.openssl
    ]}:$LD_LIBRARY_PATH
  '';
}
