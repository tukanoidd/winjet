{...}: {
  perSystem = {
    pkgs,
    config,
    ...
  }: let
    crateName = "winjet";
  in {
    nci = {
      projects."winjet".path = ./.;
      crates.${crateName} = {
        runtimeLibs = with pkgs;
        with xorg; [
          vulkan-loader
          libGL

          wayland
          libX11
          libxkbcommon
        ];
      };
    };
  };
}
