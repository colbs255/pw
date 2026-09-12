# pw

Encrypted password storage CLI.

## Install / run with Nix

Run without installing:

```sh
nix run github:colbs255/pw -- <args>
```

Install into a profile:

```sh
nix profile install github:colbs255/pw
```

Or add it as a flake input in your own flake:

```nix
{
  inputs.pw.url = "github:colbs255/pw";

  outputs = { self, nixpkgs, pw, ... }: {
    # e.g. in home-manager: home.packages = [ pw.packages.${system}.default ];
  };
}
```

An overlay is also exposed (`pw.overlays.default`) if you'd rather pull it into
`pkgs` via `nixpkgs.overlays`.

## Development

```sh
nix develop
just check   # fmt-check + lint + test
```
