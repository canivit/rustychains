FROM nixos/nix
RUN nix-channel --update

# install interpreters/compilers

# Python
RUN nix-env -iA nixpkgs.python3

# Java
RUN nix-env -iA nixpkgs.openjdk
