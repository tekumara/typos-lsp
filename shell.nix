let
  inherit (builtins) fromJSON readFile;
  flake-compat = (fromJSON (readFile ./flake.lock)).nodes.flake-compat.locked;
  url = "https://github.com/edolstra/flake-compat/archive/${flake-compat.rev}.tar.gz";
  tarball = fetchTarball {
    url = flake-compat.url or url;
    sha256 = flake-compat.narHash;
  };
in
(import tarball { src = ./.; }).shellNix
